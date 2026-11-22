"""Unit tests for Canonical License Builder (core/license_builder.py)."""

import json
import pytest
from core.key_vault import KeyVault
from core.license_builder import (
    LicenseBuilder,
    LicenseSpec,
    LicenseValidationError,
    canonicalize_payload,
    PLAN_PRESETS,
    ALL_FEATURE_KEYS,
)

SAMPLE_HWID = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"


@pytest.fixture
def test_vault():
    unlocked, _ = KeyVault.create_vault(key_id="daqiq-test-v1", passphrase="Pass")
    return unlocked


def test_build_valid_subscription_license(test_vault):
    """Verify generating valid subscription license with all 14 fields."""
    spec = LicenseSpec(
        license_id="LIC-2026-0001",
        customer_name="فروشگاه آزمایشی دقیق",
        customer_organization="مجموعه پارس",
        customer_contact="+989123456789",
        license_type="subscription",
        issued_at="2026-11-22T00:00:00Z",
        not_before="2026-11-22T00:00:00Z",
        expires_at="2027-11-22T00:00:00Z",
        max_devices=2,
        device_fingerprints=[SAMPLE_HWID],
        features={k: True for k in ALL_FEATURE_KEYS},
        min_app_version="4.9.0",
        max_app_version=None,
    )

    gen = LicenseBuilder.build_and_sign(spec, test_vault)
    assert gen.license_id == "LIC-2026-0001"
    assert gen.key_id == test_vault.key_id
    assert gen.signature_b64 is not None

    # Check envelope structure
    doc = json.loads(gen.json_text)
    assert doc["envelope_version"] == 1
    assert doc["algorithm"] == "Ed25519"
    assert doc["key_id"] == test_vault.key_id
    assert "payload" in doc
    assert "signature" in doc

    p = doc["payload"]
    assert p["product_id"] == "daqiq.desktop"
    assert p["format_version"] == 1
    assert p["customer"]["name"] == "فروشگاه آزمایشی دقیق"
    assert p["customer"]["organization"] == "مجموعه پارس"
    assert p["customer"]["contact"] == "+989123456789"
    assert p["license_type"] == "subscription"
    assert p["issued_at"] == "2026-11-22T00:00:00Z"
    assert p["not_before"] == "2026-11-22T00:00:00Z"
    assert p["expires_at"] == "2027-11-22T00:00:00Z"
    assert p["max_devices"] == 2
    assert p["device_fingerprints"] == [SAMPLE_HWID]
    assert len(p["features"]) == 10
    assert all(p["features"].values())
    assert p["min_app_version"] == "4.9.0"
    assert p["max_app_version"] is None


def test_build_valid_perpetual_license(test_vault):
    """Verify perpetual license has expires_at == None."""
    spec = LicenseSpec(
        license_id="LIC-PERP-001",
        customer_name="شرکت ابزار صنعت",
        license_type="perpetual",
        issued_at="2026-11-22T00:00:00Z",
        not_before="2026-11-22T00:00:00Z",
        expires_at=None,
        max_devices=5,
        device_fingerprints=[SAMPLE_HWID],
        features={k: True for k in ALL_FEATURE_KEYS},
    )

    gen = LicenseBuilder.build_and_sign(spec, test_vault)
    doc = json.loads(gen.json_text)
    assert doc["payload"]["expires_at"] is None
    assert doc["payload"]["license_type"] == "perpetual"


def test_perpetual_license_rejects_expires_at(test_vault):
    """Verify perpetual license with non-null expires_at raises error."""
    spec = LicenseSpec(
        license_id="LIC-PERP-ERR",
        customer_name="تست",
        license_type="perpetual",
        issued_at="2026-11-22T00:00:00Z",
        not_before="2026-11-22T00:00:00Z",
        expires_at="2027-11-22T00:00:00Z",  # Not allowed for perpetual
        device_fingerprints=[SAMPLE_HWID],
    )
    with pytest.raises(LicenseValidationError, match="دائمی"):
        LicenseBuilder.build_and_sign(spec, test_vault)


def test_subscription_requires_valid_expiry_greater_than_not_before(test_vault):
    """Verify subscription rejects expires_at <= not_before."""
    spec = LicenseSpec(
        license_id="LIC-SUB-ERR",
        customer_name="تست",
        license_type="subscription",
        issued_at="2026-11-22T00:00:00Z",
        not_before="2026-11-22T00:00:00Z",
        expires_at="2026-11-20T00:00:00Z",  # Before not_before!
        device_fingerprints=[SAMPLE_HWID],
    )
    with pytest.raises(LicenseValidationError, match="انقضا"):
        LicenseBuilder.build_and_sign(spec, test_vault)


def test_reject_more_devices_than_max_devices(test_vault):
    """Verify len(device_fingerprints) > max_devices raises error."""
    spec = LicenseSpec(
        license_id="LIC-DEV-ERR",
        customer_name="تست",
        license_type="perpetual",
        issued_at="2026-11-22T00:00:00Z",
        not_before="2026-11-22T00:00:00Z",
        expires_at=None,
        max_devices=1,
        device_fingerprints=[
            SAMPLE_HWID,
            "fp_v1:cc12b7ba73d61e6240c106bb3239c09690102a0ca9f448497532733e8a5382cf",
        ],
    )
    with pytest.raises(LicenseValidationError, match="سقف مجاز"):
        LicenseBuilder.build_and_sign(spec, test_vault)


def test_canonicalize_payload_rejects_floats():
    """Verify floats are rejected during canonicalization."""
    with pytest.raises(LicenseValidationError, match="float"):
        canonicalize_payload({"max_devices": 1.0})
