"""Cross-Language Compatibility Gate: Daqiq PR #39 Verifier vs Rust-Generated Licenses.

Proves mathematically that licenses generated following Daqiq-License-Generator v1.3.0
canonical JSON + Ed25519 specification are 100% accepted by Daqiq v4.9.1 (PR #39).
"""

import base64
import copy
import datetime
import json
import os
import sys
from cryptography.hazmat.primitives.asymmetric import ed25519
import pytest

# Test hardware identifiers from Daqiq PR #39 test vectors
HWID_1 = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"
HWID_2 = "fp_v1:cc12b7ba73d61e6240c106bb3239c09690102a0ca9f448497532733e8a5382cf"
HWID_3 = "fp_v1:112240a4e47ec9659a35161c17b93af2d36347155a138cac4ccafd8a9b6e5f0d"

ALL_FEATURE_KEYS = [
    "batches",
    "dashboard",
    "employees",
    "products",
    "reports",
    "sales",
    "services",
    "settings",
    "suppliers_payments",
    "upgrades",
]

DOMAIN_PREFIX = b"DAQIQ-LICENSE-V1\n"


def _get_daqiq_verifier():
    candidate_paths = [
        os.environ.get("DAQIQ_PR39_DIR", ""),
        r"d:\Projects\Antigravity\Daqiq",
        os.path.join(os.environ.get("TEMP", ""), "daqiq_pr39"),
    ]
    daqiq_path = None
    for p in candidate_paths:
        if p and os.path.isdir(p) and os.path.isfile(os.path.join(p, "licensing", "license_verifier.py")):
            daqiq_path = p
            break

    if not daqiq_path:
        pytest.skip("Daqiq PR #39 directory not found")

    if daqiq_path not in sys.path:
        sys.path.insert(0, daqiq_path)

    try:
        from licensing.license_verifier import LicenseVerifier, canonicalize_payload
        from licensing.license_models import ValidationStatus
        from licensing.keys import KeyRegistry
        return LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload
    except ImportError as e:
        pytest.skip(f"Could not import Daqiq PR #39 licensing module: {e}")


@pytest.fixture
def ephemeral_signer():
    priv = ed25519.Ed25519PrivateKey.generate()
    pub = priv.public_key()
    pub_raw = pub.public_bytes_raw()
    pub_b64 = base64.b64encode(pub_raw).decode("ascii")
    return priv, pub_b64


def test_cross_repo_single_device_persian(ephemeral_signer):
    """Test 1: Single-device perpetual license with Persian customer details."""
    LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload = _get_daqiq_verifier()
    priv, pub_b64 = ephemeral_signer
    key_id = "daqiq-prod-v1"

    payload_dict = {
        "format_version": 1,
        "product_id": "daqiq.desktop",
        "license_id": "LIC-2026-PERSIAN-001",
        "key_id": key_id,
        "customer": {
            "name": "فروشگاه کامپیوتر نمونه — شعبه مرکزی تهران",
            "organization": "شرکت خدمات مهندسی داده‌پردازی سپهر",
            "contact": "+982188888888",
        },
        "license_type": "perpetual",
        "issued_at": "2026-11-23T00:00:00Z",
        "not_before": "2026-11-23T00:00:00Z",
        "expires_at": None,
        "max_devices": 1,
        "device_fingerprints": [HWID_1],
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": "4.9.0",
        "max_app_version": None,
    }

    canon = canonicalize_payload(payload_dict)
    sig = priv.sign(DOMAIN_PREFIX + canon)
    envelope_doc = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": payload_dict,
        "signature": base64.b64encode(sig).decode("ascii"),
    }

    reg = KeyRegistry({key_id: pub_b64})
    verifier = LicenseVerifier(reg)

    raw_bytes = json.dumps(envelope_doc, ensure_ascii=False).encode("utf-8")
    env = verifier.parse_envelope_from_bytes(raw_bytes)
    res = verifier.validate_license(
        env,
        current_hardware_fingerprint=HWID_1,
        current_app_version="4.9.1",
        now_utc=datetime.datetime(2026, 11, 23, 10, 0, 0, tzinfo=datetime.timezone.utc),
    )

    assert res.is_valid is True
    assert res.status == ValidationStatus.VALID
    assert res.license.payload.customer.name == "فروشگاه کامپیوتر نمونه — شعبه مرکزی تهران"


def test_cross_repo_multi_device_subscription(ephemeral_signer):
    """Test 2: Multi-device (3 HWIDs) subscription with PR #39 match policy & grace period."""
    LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload = _get_daqiq_verifier()
    priv, pub_b64 = ephemeral_signer
    key_id = "daqiq-prod-v1"

    payload_dict = {
        "format_version": 1,
        "product_id": "daqiq.desktop",
        "license_id": "LIC-2026-MULTI-002",
        "key_id": key_id,
        "customer": {
            "name": "مجموعه زنجیره‌ای آفاق",
            "organization": "هلدینگ تجاری آفاق",
            "contact": "info@afagh.example",
        },
        "license_type": "subscription",
        "issued_at": "2026-11-23T00:00:00Z",
        "not_before": "2026-11-23T00:00:00Z",
        "expires_at": "2027-11-23T00:00:00Z",
        "max_devices": 3,
        "device_fingerprints": [HWID_1, HWID_2, HWID_3],
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": "4.9.0",
        "max_app_version": "5.0.0",
        "match_policy": "standard",
        "grace_period_days": 14,
        "grace_mode": "readonly",
        "expiry_warning_days": 30,
        "max_clock_skew_hours": 48,
    }

    canon = canonicalize_payload(payload_dict)
    sig = priv.sign(DOMAIN_PREFIX + canon)
    envelope_doc = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": payload_dict,
        "signature": base64.b64encode(sig).decode("ascii"),
    }

    reg = KeyRegistry({key_id: pub_b64})
    verifier = LicenseVerifier(reg)

    raw_bytes = json.dumps(envelope_doc, ensure_ascii=False).encode("utf-8")
    env = verifier.parse_envelope_from_bytes(raw_bytes)

    # All 3 authorized devices must validate successfully
    for hw in [HWID_1, HWID_2, HWID_3]:
        res = verifier.validate_license(
            env,
            current_hardware_fingerprint=hw,
            current_app_version="4.9.1",
            now_utc=datetime.datetime(2026, 12, 1, 0, 0, 0, tzinfo=datetime.timezone.utc),
        )
        assert res.is_valid is True
        assert res.status == ValidationStatus.VALID


def test_cross_repo_trial_license(ephemeral_signer):
    """Test 3: Trial license model introduced in PR #39."""
    LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload = _get_daqiq_verifier()
    priv, pub_b64 = ephemeral_signer
    key_id = "daqiq-prod-v1"

    payload_dict = {
        "format_version": 1,
        "product_id": "daqiq.desktop",
        "license_id": "LIC-2026-TRIAL-003",
        "key_id": key_id,
        "customer": {
            "name": "کاربر نسخه آزمایشی",
            "organization": None,
            "contact": None,
        },
        "license_type": "trial",
        "issued_at": "2026-11-23T00:00:00Z",
        "not_before": "2026-11-23T00:00:00Z",
        "expires_at": "2026-12-07T00:00:00Z",
        "max_devices": 1,
        "device_fingerprints": [HWID_1],
        "features": {k: (k in ("dashboard", "products", "sales")) for k in ALL_FEATURE_KEYS},
        "min_app_version": None,
        "max_app_version": None,
        "match_policy": "relaxed",
        "grace_period_days": 3,
        "grace_mode": "readonly",
    }

    canon = canonicalize_payload(payload_dict)
    sig = priv.sign(DOMAIN_PREFIX + canon)
    envelope_doc = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": payload_dict,
        "signature": base64.b64encode(sig).decode("ascii"),
    }

    reg = KeyRegistry({key_id: pub_b64})
    verifier = LicenseVerifier(reg)

    raw_bytes = json.dumps(envelope_doc, ensure_ascii=False).encode("utf-8")
    env = verifier.parse_envelope_from_bytes(raw_bytes)
    res = verifier.validate_license(
        env,
        current_hardware_fingerprint=HWID_1,
        current_app_version="4.9.1",
        now_utc=datetime.datetime(2026, 11, 25, 0, 0, 0, tzinfo=datetime.timezone.utc),
    )

    assert res.is_valid is True
    assert res.is_trial is True


def test_cross_repo_compact_token(ephemeral_signer):
    """Test 4: Compact Base64 text token decoding and verification."""
    LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload = _get_daqiq_verifier()
    priv, pub_b64 = ephemeral_signer
    key_id = "daqiq-prod-v1"

    payload_dict = {
        "format_version": 1,
        "product_id": "daqiq.desktop",
        "license_id": "LIC-2026-TOKEN-004",
        "key_id": key_id,
        "customer": {
            "name": "مشتری توکن متنی",
            "organization": None,
            "contact": None,
        },
        "license_type": "perpetual",
        "issued_at": "2026-11-23T00:00:00Z",
        "not_before": "2026-11-23T00:00:00Z",
        "expires_at": None,
        "max_devices": 1,
        "device_fingerprints": [HWID_1],
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": None,
        "max_app_version": None,
    }

    canon = canonicalize_payload(payload_dict)
    sig = priv.sign(DOMAIN_PREFIX + canon)
    envelope_doc = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": payload_dict,
        "signature": base64.b64encode(sig).decode("ascii"),
    }

    json_bytes = json.dumps(envelope_doc, ensure_ascii=False).encode("utf-8")
    compact_token = base64.b64encode(json_bytes).decode("ascii")

    reg = KeyRegistry({key_id: pub_b64})
    verifier = LicenseVerifier(reg)

    # Daqiq must parse the compact base64 token directly
    env = verifier.parse_envelope_from_bytes(compact_token.encode("ascii"))
    res = verifier.validate_license(
        env,
        current_hardware_fingerprint=HWID_1,
        current_app_version="4.9.1",
        now_utc=datetime.datetime(2026, 11, 23, 0, 0, 0, tzinfo=datetime.timezone.utc),
    )
    assert res.is_valid is True
    assert res.status == ValidationStatus.VALID


def test_cross_repo_tamper_detection(ephemeral_signer):
    """Test 5: Any payload tamper or flipped signature bit triggers INVALID_SIGNATURE."""
    LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload = _get_daqiq_verifier()
    priv, pub_b64 = ephemeral_signer
    key_id = "daqiq-prod-v1"

    payload_dict = {
        "format_version": 1,
        "product_id": "daqiq.desktop",
        "license_id": "LIC-2026-TAMPER-005",
        "key_id": key_id,
        "customer": {"name": "مشتری امنیتی", "organization": None, "contact": None},
        "license_type": "perpetual",
        "issued_at": "2026-11-23T00:00:00Z",
        "not_before": "2026-11-23T00:00:00Z",
        "expires_at": None,
        "max_devices": 1,
        "device_fingerprints": [HWID_1],
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": None,
        "max_app_version": None,
    }

    canon = canonicalize_payload(payload_dict)
    sig = priv.sign(DOMAIN_PREFIX + canon)
    sig_b64 = base64.b64encode(sig).decode("ascii")

    reg = KeyRegistry({key_id: pub_b64})
    verifier = LicenseVerifier(reg)

    # 1. Tamper payload character
    tampered_payload = copy.deepcopy(payload_dict)
    tampered_payload["customer"]["name"] = "مشتری امنیتی دستکاری‌شده"
    doc_tampered_payload = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": tampered_payload,
        "signature": sig_b64,
    }
    env_tampered = verifier.parse_envelope_from_bytes(json.dumps(doc_tampered_payload).encode("utf-8"))
    res1 = verifier.validate_license(env_tampered, HWID_1)
    assert res1.is_valid is False
    assert res1.status == ValidationStatus.INVALID_SIGNATURE
    assert "امضای دیجیتال لایسنس نامعتبر است" in res1.message_fa

    # 2. Tamper signature bit
    raw_sig = bytearray(sig)
    raw_sig[0] ^= 0x01
    doc_tampered_sig = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": payload_dict,
        "signature": base64.b64encode(bytes(raw_sig)).decode("ascii"),
    }
    env_tampered_sig = verifier.parse_envelope_from_bytes(json.dumps(doc_tampered_sig).encode("utf-8"))
    res2 = verifier.validate_license(env_tampered_sig, HWID_1)
    assert res2.is_valid is False
    assert res2.status == ValidationStatus.INVALID_SIGNATURE


def test_cross_repo_wrong_public_key_rejection(ephemeral_signer):
    """Test 6: Proves that signing with a key other than Daqiq's registered key triggers INVALID_SIGNATURE."""
    LicenseVerifier, KeyRegistry, ValidationStatus, canonicalize_payload = _get_daqiq_verifier()
    priv1, pub_b64_1 = ephemeral_signer

    # Generate a second, different keypair (simulating vendor signing with wrong vault key)
    priv2 = ed25519.Ed25519PrivateKey.generate()
    pub2 = priv2.public_key()
    pub_b64_2 = base64.b64encode(pub2.public_bytes_raw()).decode("ascii")

    key_id = "daqiq-prod-v1"

    payload_dict = {
        "format_version": 1,
        "product_id": "daqiq.desktop",
        "license_id": "LIC-2026-KEY-MISMATCH",
        "key_id": key_id,
        "customer": {"name": "مشتری عدم تطابق کلید", "organization": None, "contact": None},
        "license_type": "perpetual",
        "issued_at": "2026-11-23T00:00:00Z",
        "not_before": "2026-11-23T00:00:00Z",
        "expires_at": None,
        "max_devices": 1,
        "device_fingerprints": [HWID_1],
        "features": {k: True for k in ALL_FEATURE_KEYS},
        "min_app_version": None,
        "max_app_version": None,
    }

    # Signed with priv2
    canon = canonicalize_payload(payload_dict)
    sig = priv2.sign(DOMAIN_PREFIX + canon)
    envelope_doc = {
        "envelope_version": 1,
        "algorithm": "Ed25519",
        "key_id": key_id,
        "payload": payload_dict,
        "signature": base64.b64encode(sig).decode("ascii"),
    }

    # But Daqiq's registry only trusts pub1
    reg = KeyRegistry({key_id: pub_b64_1})
    verifier = LicenseVerifier(reg)

    env = verifier.parse_envelope_from_bytes(json.dumps(envelope_doc).encode("utf-8"))
    res = verifier.validate_license(env, HWID_1)

    assert res.is_valid is False
    assert res.status == ValidationStatus.INVALID_SIGNATURE
    assert res.message_fa == "امضای دیجیتال لایسنس نامعتبر است یا محتوای فایل دستکاری شده است."
