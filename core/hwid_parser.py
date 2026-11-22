"""Hardware ID Parser and Multi-Device Manager for Daqiq fp_v1 node-locking.

Matches Daqiq PR #38 SMBIOS fp_v1 specifications with 100% precision:
- Sanitizes Persian/Arabic BiDi control characters, invisible zero-width chars, quotes.
- Validates fp_v1 format and 64-character lowercase hex digest.
- Supports single paste, bulk paste (multi-line, comma-separated), and .daqreq file import.
- Handles duplicate detection, per-device friendly labels, and hardware tier metadata.
"""

from __future__ import annotations

import json
import os
import re
import unicodedata
from dataclasses import dataclass, field
from typing import List, Optional, Tuple, Set, Dict, Any

# Strict regex matching Daqiq fp_v1 specification
FP_V1_REGEX = re.compile(r"^fp_v1:[0-9a-fA-F]{64}$")

# Persian/Arabic BiDi, invisible zero-width, and formatting characters to strip
BIDI_AND_INVISIBLE_CHARS = {
    "\u200e",  # Left-to-Right Mark (LRM)
    "\u200f",  # Right-to-Left Mark (RLM)
    "\u202a",  # Left-to-Right Embedding (LRE)
    "\u202b",  # Right-to-Left Embedding (RLE)
    "\u202c",  # Pop Directional Formatting (PDF)
    "\u202d",  # Left-to-Right Override (LRO)
    "\u202e",  # Right-to-Left Override (RLO)
    "\u2066",  # Left-to-Right Isolate (LRI)
    "\u2067",  # Right-to-Left Isolate (RLI)
    "\u2068",  # First Strong Isolate (FSI)
    "\u2069",  # Pop Directional Isolate (PDI)
    "\u200c",  # Zero-Width Non-Joiner (ZWNJ)
    "\u200d",  # Zero-Width Joiner (ZWJ)
    "\u200b",  # Zero-Width Space
    "\ufeff",  # Zero-Width No-Break Space / BOM
}


def clean_hwid_string(raw: Optional[str]) -> str:
    """Clean and sanitize a Hardware ID string.

    Strips whitespace, surrounding quotes, brackets, and invisible BiDi formatting characters.
    Normalizes result to lowercase Unicode NFC.
    """
    if not raw:
        return ""

    s = unicodedata.normalize("NFC", str(raw))
    # Strip BiDi and invisible formatting characters
    for ch in BIDI_AND_INVISIBLE_CHARS:
        s = s.replace(ch, "")

    # Strip surrounding quotes, brackets, and whitespace
    s = s.strip()
    s = s.strip("\"'«»“”‘’[]()")
    s = s.strip()

    # If prefixed with common prefixes like "HWID:", "شناسه:", strip them
    s = re.sub(r"^(hwid|device_id|شناسه|شناسه دستگاه)\s*[:=]\s*", "", s, flags=re.IGNORECASE)
    return s.strip().lower()


def validate_hwid_string(hwid: str) -> bool:
    """Check if cleaned Hardware ID strictly matches the fp_v1:<64-hex> specification."""
    if not hwid:
        return False
    return bool(FP_V1_REGEX.match(hwid))


@dataclass
class DeviceEntry:
    """Represents a customer hardware device bound to a license."""
    fingerprint: str
    label: str = ""
    hardware_tier: str = "tier1"
    is_valid: bool = True
    validation_error: str = ""
    match_policy: str = "strict"  # "strict" or "tier_fallback"
    app_version: Optional[str] = None
    source: str = "manual"  # "manual", "bulk", "daqreq"

    def to_dict(self) -> Dict[str, Any]:
        return {
            "fingerprint": self.fingerprint,
            "label": self.label,
            "hardware_tier": self.hardware_tier,
            "match_policy": self.match_policy,
            "app_version": self.app_version,
            "source": self.source,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> DeviceEntry:
        fp = clean_hwid_string(data.get("fingerprint", ""))
        valid = validate_hwid_string(fp)
        return cls(
            fingerprint=fp,
            label=data.get("label", ""),
            hardware_tier=data.get("hardware_tier", "tier1"),
            is_valid=valid,
            validation_error="" if valid else "شناسه سخت‌افزاری با فرمت fp_v1 همخوانی ندارد.",
            match_policy=data.get("match_policy", "strict"),
            app_version=data.get("app_version"),
            source=data.get("source", "manual"),
        )


@dataclass
class ParsedHWIDBatch:
    """Result of parsing single or bulk Hardware ID inputs."""
    devices: List[DeviceEntry] = field(default_factory=list)
    valid_count: int = 0
    invalid_count: int = 0
    duplicate_count: int = 0
    duplicates: List[str] = field(default_factory=list)

    @property
    def valid_fingerprints(self) -> List[str]:
        return [d.fingerprint for d in self.devices if d.is_valid]


class HWIDParser:
    """Parser and validator for Daqiq v4.9.0 Hardware IDs and .daqreq request files."""

    @classmethod
    def parse_single(cls, raw: str, label: str = "") -> DeviceEntry:
        """Parse a single Hardware ID string with optional label."""
        cleaned = clean_hwid_string(raw)
        valid = validate_hwid_string(cleaned)
        err = "" if valid else "فرمت شناسه نامعتبر است. ساختار صحیح: fp_v1:<۶۴ نویسه هگزادسیمال>"
        return DeviceEntry(
            fingerprint=cleaned,
            label=label.strip(),
            hardware_tier="tier1",
            is_valid=valid,
            validation_error=err,
            source="manual",
        )

    @classmethod
    def parse_bulk_text(cls, bulk_text: str, default_label_prefix: str = "دستگاه") -> ParsedHWIDBatch:
        """Parse multi-line or delimited text containing one or multiple Hardware IDs.

        Accepts inputs formatted as:
        - One fp_v1 per line
        - Comma or semicolon separated
        - 'Label: fp_v1:...' or 'fp_v1:... (Label)'
        """
        result = ParsedHWIDBatch()
        if not bulk_text or not bulk_text.strip():
            return result

        # Replace semicolons with newlines, but preserve lines
        raw_lines = re.split(r"[\r\n;]+", bulk_text)
        seen_fps: Set[str] = set()
        device_index = 1

        for raw_line in raw_lines:
            line = raw_line.strip()
            if not line or line.startswith("#") or line.startswith("//"):
                continue

            # Split on commas if line contains multiple comma-separated IDs
            tokens = [t.strip() for t in line.split(",") if t.strip()]
            for token in tokens:
                # Check for label syntax: "Label: fp_v1:..." or "fp_v1:... - Label" or "fp_v1:... (Label)"
                extracted_label = ""
                candidate_fp = token

                # Pattern: "Label: fp_v1:..."
                colon_match = re.match(r"^([^:]+):\s*(fp_v1:[0-9a-fA-F]{64})\s*$", token, flags=re.IGNORECASE)
                if colon_match:
                    extracted_label = colon_match.group(1).strip()
                    candidate_fp = colon_match.group(2).strip()
                else:
                    # Pattern: "fp_v1:... (Label)"
                    paren_match = re.match(r"^(fp_v1:[0-9a-fA-F]{64})\s*\(([^)]+)\)\s*$", token, flags=re.IGNORECASE)
                    if paren_match:
                        candidate_fp = paren_match.group(1).strip()
                        extracted_label = paren_match.group(2).strip()
                    else:
                        # Pattern: "fp_v1:... - Label"
                        dash_match = re.match(r"^(fp_v1:[0-9a-fA-F]{64})\s*[-–—]\s*(.+)$", token, flags=re.IGNORECASE)
                        if dash_match:
                            candidate_fp = dash_match.group(1).strip()
                            extracted_label = dash_match.group(2).strip()

                cleaned_fp = clean_hwid_string(candidate_fp)
                if not cleaned_fp:
                    continue

                valid = validate_hwid_string(cleaned_fp)
                label = extracted_label or f"{default_label_prefix} {device_index}"

                if valid:
                    if cleaned_fp in seen_fps:
                        result.duplicate_count += 1
                        result.duplicates.append(cleaned_fp)
                        # Add as invalid duplicate entry to give transparent user feedback
                        entry = DeviceEntry(
                            fingerprint=cleaned_fp,
                            label=label,
                            hardware_tier="tier1",
                            is_valid=False,
                            validation_error="شناسه سخت‌افزاری تکراری در این لیست وجود دارد.",
                            source="bulk",
                        )
                        result.devices.append(entry)
                        result.invalid_count += 1
                    else:
                        seen_fps.add(cleaned_fp)
                        entry = DeviceEntry(
                            fingerprint=cleaned_fp,
                            label=label,
                            hardware_tier="tier1",
                            is_valid=True,
                            validation_error="",
                            source="bulk",
                        )
                        result.devices.append(entry)
                        result.valid_count += 1
                        device_index += 1
                else:
                    err = "ساختار شناسه سخت‌افزاری معتبر نیست (باید با fp_v1 شروع شده و ۶۴ رقم هگزادسیمال باشد)."
                    entry = DeviceEntry(
                        fingerprint=cleaned_fp,
                        label=label,
                        hardware_tier="unknown",
                        is_valid=False,
                        validation_error=err,
                        source="bulk",
                    )
                    result.devices.append(entry)
                    result.invalid_count += 1

        return result

    @classmethod
    def parse_daqreq_content(cls, raw_text: str) -> DeviceEntry:
        """Parse JSON text conforming to Companion App A (.daqreq) specification."""
        try:
            doc = json.loads(raw_text)
        except Exception as e:
            raise ValueError(f"خطا در خواندن ساختار JSON فایل درخواست: {e}") from e

        if not isinstance(doc, dict):
            raise ValueError("فایل درخواست باید یک شیء JSON معتبر باشد.")

        product_id = doc.get("product_id")
        if product_id != "daqiq.desktop":
            raise ValueError(f"شناسه محصول در فایل درخواست نامعتبر است: {product_id}")

        req_ver = doc.get("request_version")
        if req_ver != 1:
            raise ValueError(f"نسخه فایل درخواست ({req_ver}) پشتیبانی نمی‌شود.")

        raw_fp = doc.get("device_fingerprint")
        cleaned_fp = clean_hwid_string(raw_fp)
        if not validate_hwid_string(cleaned_fp):
            raise ValueError(f"شناسه سخت‌افزاری داخل فایل درخواست نامعتبر است: {raw_fp}")

        label = doc.get("device_label", "")
        tier = doc.get("hardware_tier", "tier1")
        app_ver = doc.get("app_version")

        return DeviceEntry(
            fingerprint=cleaned_fp,
            label=label or "سیستم متقاضی",
            hardware_tier=tier,
            is_valid=True,
            validation_error="",
            match_policy="strict",
            app_version=app_ver,
            source="daqreq",
        )

    @classmethod
    def parse_daqreq_file(cls, file_path: str) -> DeviceEntry:
        """Read and parse a .daqreq file from disk."""
        if not os.path.isfile(file_path):
            raise FileNotFoundError(f"فایل درخواست یافت نشد: {file_path}")

        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        return cls.parse_daqreq_content(content)
