"""Unit tests for HWID Parser and Multi-Device Manager (core/hwid_parser.py)."""

import json
import pytest
from core.hwid_parser import (
    clean_hwid_string,
    validate_hwid_string,
    HWIDParser,
    DeviceEntry,
)

SAMPLE_VALID_HWID = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"
SAMPLE_VALID_HWID_2 = "fp_v1:cc12b7ba73d61e6240c106bb3239c09690102a0ca9f448497532733e8a5382cf"


def test_clean_hwid_persian_bidi_and_whitespace():
    """Verify stripping of Persian BiDi marks, quotes, and whitespace."""
    # Embedded with LRM, RLM, ZWNJ, quotes, whitespace
    dirty = f"\u200e \u200f«  {SAMPLE_VALID_HWID}  »\u200c"
    cleaned = clean_hwid_string(dirty)
    assert cleaned == SAMPLE_VALID_HWID
    assert validate_hwid_string(cleaned) is True


def test_clean_hwid_with_prefix():
    """Verify common prefixes like 'HWID:' or 'شناسه:' are stripped."""
    with_prefix = f"شناسه: {SAMPLE_VALID_HWID}"
    assert clean_hwid_string(with_prefix) == SAMPLE_VALID_HWID

    with_prefix_en = f"HWID: {SAMPLE_VALID_HWID}"
    assert clean_hwid_string(with_prefix_en) == SAMPLE_VALID_HWID


def test_invalid_hwid_rejection():
    """Verify corrupted, truncated, or non-hex HWIDs are rejected."""
    assert validate_hwid_string("") is False
    assert validate_hwid_string("fp_v1:") is False
    assert validate_hwid_string("fp_v1:12345") is False  # too short
    assert validate_hwid_string(f"{SAMPLE_VALID_HWID}0") is False  # too long
    assert validate_hwid_string(SAMPLE_VALID_HWID.replace("b", "z")) is False  # non-hex
    assert validate_hwid_string("some-random-uuid-here") is False


def test_bulk_parse_multi_line():
    """Verify parsing multi-line inputs with labels and duplicate detection."""
    bulk_input = f"""
    صندوق اصلی: {SAMPLE_VALID_HWID}
    {SAMPLE_VALID_HWID_2} (لپتاپ مدیریت)
    
    # Duplicate line below
    {SAMPLE_VALID_HWID}
    # Invalid line
    fp_v1:invalid-hex-here
    """

    batch = HWIDParser.parse_bulk_text(bulk_input)
    assert batch.valid_count == 2
    assert batch.duplicate_count == 1
    assert batch.invalid_count == 2  # 1 duplicate + 1 invalid
    assert len(batch.devices) == 4

    # Check labels extracted properly
    assert batch.devices[0].label == "صندوق اصلی"
    assert batch.devices[1].label == "لپتاپ مدیریت"
    assert batch.valid_fingerprints == [SAMPLE_VALID_HWID, SAMPLE_VALID_HWID_2]


def test_parse_daqreq_file():
    """Verify parsing Daqiq PR #38 .daqreq JSON request format."""
    daqreq_json = json.dumps({
        "request_version": 1,
        "product_id": "daqiq.desktop",
        "generated_at": "2026-11-22T00:00:00Z",
        "device_fingerprint": SAMPLE_VALID_HWID,
        "device_label": "صندوق فروشگاهی ۱",
        "app_version": "4.9.0",
        "hardware_tier": "tier1",
    })

    entry = HWIDParser.parse_daqreq_content(daqreq_json)
    assert entry.is_valid is True
    assert entry.fingerprint == SAMPLE_VALID_HWID
    assert entry.label == "صندوق فروشگاهی ۱"
    assert entry.hardware_tier == "tier1"
    assert entry.app_version == "4.9.0"
