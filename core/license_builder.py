"""Canonical License Builder, Signer, and Self-Verifier for Daqiq v4.9.0 (.daqlic).

Enforces 100% byte-for-byte compatibility with Daqiq PR #38:
- Constructs exact 14-field payload schema.
- NFC Unicode normalization and deterministic JSON serialization.
- Ed25519 digital signature with domain separation prefix 'DAQIQ-LICENSE-V1\\n'.
- Mandatory post-sign self-verification pass against the public key before writing.
- Atomic file persistence (tmp + os.replace).
- Compact text activation token generator.
"""

from __future__ import annotations

import base64
import datetime
import json
import os
import re
import tempfile
import unicodedata
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric import ed25519
from packaging import version as pkg_version

from .key_vault import UnlockedVault


MAX_LICENSE_FILE_BYTES = 65536  # 64 KiB
DOMAIN_PREFIX = b"DAQIQ-LICENSE-V1\n"

# Canonical list of 10 Daqiq application tab modules
ALL_FEATURE_KEYS = [
    "dashboard",
    "products",
    "batches",
    "sales",
    "services",
    "upgrades",
    "suppliers_payments",
    "reports",
    "employees",
    "settings",
]

# Preset plan configurations
PLAN_PRESETS = {
    "trial": {
        "label_fa": "آزمایشی (Trial)",
        "license_type": "subscription",
        "duration_days": 14,
        "max_devices": 1,
        "features": {
            "dashboard": True,
            "products": True,
            "batches": False,
            "sales": True,
            "services": False,
            "upgrades": False,
            "suppliers_payments": False,
            "reports": True,
            "employees": False,
            "settings": True,
        },
        "min_app_version": "4.9.0",
        "max_app_version": None,
    },
    "standard": {
        "label_fa": "استاندارد (Standard)",
        "license_type": "subscription",
        "duration_days": 365,
        "max_devices": 1,
        "features": {
            "dashboard": True,
            "products": True,
            "batches": True,
            "sales": True,
            "services": True,
            "upgrades": False,
            "suppliers_payments": False,
            "reports": True,
            "employees": True,
            "settings": True,
        },
        "min_app_version": "4.9.0",
        "max_app_version": None,
    },
    "pro": {
        "label_fa": "حرفه‌ای (Pro)",
        "license_type": "subscription",
        "duration_days": 365,
        "max_devices": 3,
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": "4.9.0",
        "max_app_version": None,
    },
    "enterprise": {
        "label_fa": "سازمانی دائمی (Enterprise Perpetual)",
        "license_type": "perpetual",
        "duration_days": 0,
        "max_devices": 10,
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": "4.9.0",
        "max_app_version": None,
    },
    "custom": {
        "label_fa": "سفارشی (Custom)",
        "license_type": "subscription",
        "duration_days": 365,
        "max_devices": 1,
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": "4.9.0",
        "max_app_version": None,
    },
}


class LicenseValidationError(Exception):
    """Raised when license specification violates PR #38 validation rules."""
    pass


def _assert_no_floats(obj: Any, path: str = "payload") -> None:
    """Strictly forbid float values inside license payload."""
    if isinstance(obj, float):
        raise LicenseValidationError(f"مقدار اعشاری (float) در ساختار لایسنس مجاز نیست: {path}")
    if isinstance(obj, dict):
        for k, v in obj.items():
            _assert_no_floats(v, f"{path}.{k}")
    elif isinstance(obj, list):
        for i, item in enumerate(obj):
            _assert_no_floats(item, f"{path}[{i}]")


def normalize_value_nfc(val: Any) -> Any:
    """Recursively normalize strings and dictionary keys to Unicode NFC."""
    if isinstance(val, str):
        return unicodedata.normalize("NFC", val)
    if isinstance(val, dict):
        return {
            unicodedata.normalize("NFC", str(k)): normalize_value_nfc(v)
            for k, v in val.items()
        }
    if isinstance(val, list):
        return [normalize_value_nfc(item) for item in val]
    return val


def canonicalize_payload(payload_dict: Dict[str, Any]) -> bytes:
    """Serialize normalized payload dictionary to canonical UTF-8 bytes matching PR #38."""
    _assert_no_floats(payload_dict)
    normalized = normalize_value_nfc(payload_dict)
    return json.dumps(
        normalized,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")


def parse_iso8601_utc(dt_str: str) -> datetime.datetime:
    """Parse UTC ISO-8601 string to timezone-aware UTC datetime."""
    clean = dt_str.strip()
    if clean.endswith("Z"):
        clean = clean[:-1] + "+00:00"
    dt = datetime.datetime.fromisoformat(clean)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=datetime.timezone.utc)
    else:
        dt = dt.astimezone(datetime.timezone.utc)
    return dt


@dataclass
class LicenseSpec:
    """Complete specification parameters for constructing a .daqlic license."""
    license_id: str
    customer_name: str
    customer_organization: Optional[str] = None
    customer_contact: Optional[str] = None
    license_type: str = "subscription"  # "perpetual" or "subscription"
    issued_at: Optional[str] = None     # ISO-8601 UTC
    not_before: Optional[str] = None    # ISO-8601 UTC
    expires_at: Optional[str] = None    # ISO-8601 UTC or None for perpetual
    max_devices: int = 1
    device_fingerprints: List[str] = field(default_factory=list)
    features: Dict[str, bool] = field(default_factory=dict)
    min_app_version: Optional[str] = "4.9.0"
    max_app_version: Optional[str] = None
    edition_label: str = "Standard"
    notes: str = ""

    def validate(self) -> None:
        """Validate specification according to Daqiq PR #38 invariants."""
        if not self.license_id or not self.license_id.strip():
            raise LicenseValidationError("شماره لایسنس (license_id) الزامی است.")

        if not self.customer_name or not self.customer_name.strip():
            raise LicenseValidationError("نام مشتری یا فروشگاه الزامی است.")

        if self.license_type not in ("perpetual", "subscription"):
            raise LicenseValidationError(f"نوع لایسنس نامعتبر است: {self.license_type}")

        if not self.issued_at:
            raise LicenseValidationError("تاریخ صدور لایسنس الزامی است.")
        if not self.not_before:
            raise LicenseValidationError("تاریخ شروع اعتبار لایسنس الزامی است.")

        dt_nb = parse_iso8601_utc(self.not_before)

        if self.license_type == "perpetual":
            if self.expires_at is not None:
                raise LicenseValidationError("لایسنس دائمی نباید دارای تاریخ انقضا باشد.")
        else:
            if not self.expires_at:
                raise LicenseValidationError("لایسنس اشتراکی باید دارای تاریخ انقضا باشد.")
            dt_exp = parse_iso8601_utc(self.expires_at)
            if dt_exp <= dt_nb:
                raise LicenseValidationError("تاریخ انقضای لایسنس باید پس از تاریخ شروع اعتبار آن باشد.")

        if self.max_devices < 1:
            raise LicenseValidationError("سقف مجاز تعداد دستگاه‌ها (max_devices) باید حداقل ۱ باشد.")

        if not self.device_fingerprints:
            raise LicenseValidationError("حداقل یک شناسه سخت‌افزاری مجاز (fp_v1) باید برای لایسنس تعیین شود.")

        unique_fps = set(self.device_fingerprints)
        if len(unique_fps) != len(self.device_fingerprints):
            raise LicenseValidationError("شناسه‌های سخت‌افزاری تکراری در فهرست دستگاه‌ها مجاز نیست.")

        from .hwid_parser import validate_hwid_string
        for fp in self.device_fingerprints:
            if not validate_hwid_string(fp):
                raise LicenseValidationError(f"شناسه سخت‌افزاری نامعتبر است: {fp}")

        if len(self.device_fingerprints) > self.max_devices:
            raise LicenseValidationError(
                f"تعداد شناسه‌های سخت‌افزاری واردشده ({len(self.device_fingerprints)}) "
                f"بیشتر از سقف مجاز دستگاه‌ها ({self.max_devices}) است."
            )

        # Validate features
        for k in ALL_FEATURE_KEYS:
            if k not in self.features:
                self.features[k] = False
            elif not isinstance(self.features[k], bool):
                raise LicenseValidationError(f"مقدار ماژول {k} باید بولی (True/False) باشد.")

        # Validate app versions
        if self.min_app_version:
            try:
                pkg_version.parse(self.min_app_version)
            except Exception as e:
                raise LicenseValidationError(f"فرمت حداقل نسخه برنامه نامعتبر است: {e}") from e

        if self.max_app_version:
            try:
                max_v = pkg_version.parse(self.max_app_version)
                if self.min_app_version:
                    min_v = pkg_version.parse(self.min_app_version)
                    if max_v < min_v:
                        raise LicenseValidationError("حداکثر نسخه برنامه نمی‌تواند کمتر از حداقل نسخه باشد.")
            except Exception as e:
                raise LicenseValidationError(f"فرمت حداکثر نسخه برنامه نامعتبر است: {e}") from e


@dataclass
class GeneratedLicense:
    """Result of constructing, signing, and self-verifying a .daqlic file."""
    envelope_dict: Dict[str, Any]
    canonical_payload_bytes: bytes
    json_text: str
    signature_b64: str
    spec: LicenseSpec
    key_id: str
    key_fingerprint: str
    compact_token: str

    @property
    def license_id(self) -> str:
        return self.spec.license_id

    @property
    def customer_name(self) -> str:
        return self.spec.customer_name

    def save_to_file(self, file_path: str) -> str:
        """Atomically persist signed .daqlic content to disk."""
        target_dir = os.path.dirname(os.path.abspath(file_path))
        os.makedirs(target_dir, exist_ok=True)

        fd, tmp_path = tempfile.mkstemp(dir=target_dir, prefix=".daqlic_tmp_")
        try:
            with open(fd, "w", encoding="utf-8") as f:
                f.write(self.json_text)
                f.flush()
                os.fsync(f.fileno())
            os.replace(tmp_path, file_path)
            return file_path
        except Exception as e:
            if os.path.exists(tmp_path):
                try:
                    os.remove(tmp_path)
                except Exception:
                    pass
            raise IOError(f"خطا در ذخیره‌سازی فایل لایسنس: {e}") from e


class LicenseBuilder:
    """Engine for building, signing, and self-verifying .daqlic licenses."""

    @classmethod
    def generate_license_id(cls, year: Optional[int] = None) -> str:
        """Generate a commercial license ID e.g. DQL-1405-A7B2C9."""
        if year is None:
            now = datetime.datetime.now(datetime.timezone.utc)
            year = now.year
        import secrets
        rand_token = secrets.token_hex(3).upper()
        return f"DQL-{year}-{rand_token}"

    @classmethod
    def build_and_sign(
        cls,
        spec: LicenseSpec,
        vault: UnlockedVault,
    ) -> GeneratedLicense:
        """Construct canonical payload, sign with vault secret key, and self-verify."""
        if not vault.is_unlocked:
            raise LicenseValidationError("گاوصندوق در وضعیت قفل قرار دارد. ابتدا گاوصندوق را باز کنید.")

        # 1. Enforce spec validation
        spec.validate()

        # 2. Build 14-field payload dictionary strictly conforming to Daqiq PR #38
        payload_dict: Dict[str, Any] = {
            "customer": {
                "name": spec.customer_name.strip(),
                "organization": spec.customer_organization.strip() if spec.customer_organization else None,
                "contact": spec.customer_contact.strip() if spec.customer_contact else None,
            },
            "device_fingerprints": list(spec.device_fingerprints),
            "expires_at": spec.expires_at,
            "features": {k: bool(spec.features.get(k, False)) for k in ALL_FEATURE_KEYS},
            "format_version": 1,
            "issued_at": spec.issued_at,
            "key_id": vault.key_id,
            "license_id": spec.license_id.strip(),
            "license_type": spec.license_type,
            "max_app_version": spec.max_app_version,
            "max_devices": spec.max_devices,
            "min_app_version": spec.min_app_version,
            "not_before": spec.not_before,
            "product_id": "daqiq.desktop",
        }

        # 3. Canonicalize payload to UTF-8 bytes
        canonical_bytes = canonicalize_payload(payload_dict)

        # 4. Sign with Ed25519 using domain separation prefix
        message_to_sign = DOMAIN_PREFIX + canonical_bytes
        signature_raw = vault.private_key.sign(message_to_sign)
        signature_b64 = base64.b64encode(signature_raw).decode("ascii")

        # 5. Build outer envelope
        envelope_dict: Dict[str, Any] = {
            "envelope_version": 1,
            "algorithm": "Ed25519",
            "key_id": vault.key_id,
            "payload": payload_dict,
            "signature": signature_b64,
        }

        json_text = json.dumps(envelope_dict, indent=2, ensure_ascii=False)

        # 6. Check size limit
        raw_encoded = json_text.encode("utf-8")
        if len(raw_encoded) > MAX_LICENSE_FILE_BYTES:
            raise LicenseValidationError(
                f"حجم فایل لایسنس ({len(raw_encoded)} بایت) بیش از سقف ۶۴ کیلوبایت است."
            )

        # 7. MANDATORY POST-SIGN SELF-VERIFICATION
        cls._self_verify(envelope_dict, vault.public_key, canonical_bytes)

        # 8. Create compact text activation token
        compact_payload = base64.b64encode(canonical_bytes).decode("ascii")
        compact_token = f"DAQIQ1.{compact_payload}.{signature_b64}"

        return GeneratedLicense(
            envelope_dict=envelope_dict,
            canonical_payload_bytes=canonical_bytes,
            json_text=json_text,
            signature_b64=signature_b64,
            spec=spec,
            key_id=vault.key_id,
            key_fingerprint=vault.fingerprint,
            compact_token=compact_token,
        )

    @classmethod
    def _self_verify(
        cls,
        envelope_dict: Dict[str, Any],
        public_key: ed25519.Ed25519PublicKey,
        canonical_bytes: bytes,
    ) -> None:
        """Run an independent verification pass to guarantee 100% cryptographic validity."""
        sig_b64 = envelope_dict.get("signature", "")
        try:
            sig_raw = base64.b64decode(sig_b64)
            if len(sig_raw) != 64:
                raise ValueError("Signature length is not 64 bytes")
            data_to_verify = DOMAIN_PREFIX + canonical_bytes
            public_key.verify(sig_raw, data_to_verify)
        except InvalidSignature as e:
            raise LicenseValidationError("خطای اعتبارسنجی داخلی: امضای دیجیتال تطابق ندارد.") from e
        except Exception as e:
            raise LicenseValidationError(f"خطای خود-اعتبارسنجی لایسنس: {e}") from e

    @classmethod
    def generate_default_filename(cls, spec: LicenseSpec) -> str:
        """Generate a clean, sanitized default filename for saving the license."""
        # Sanitize customer name for Windows filename
        safe_cust = re.sub(r'[\\/*?:"<>| ]+', "_", spec.customer_name.strip()).strip("_")
        safe_lic = re.sub(r'[\\/*?:"<>| ]+', "-", spec.license_id.strip()).strip("-")
        return f"Daqiq_{safe_cust}_{safe_lic}.daqlic"
