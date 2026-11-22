"""Unit tests for Offline Commercial Sales & License Registry (core/registry_db.py)."""

import os
import pytest
from core.registry_db import RegistryDB, LicenseRecord


@pytest.fixture
def test_db(tmp_path):
    db_file = str(tmp_path / "test_registry.sqlite")
    return RegistryDB(db_file)


def make_sample_record(lic_id="DQL-1405-0001", cust_name="فروشگاه آزمایشی"):
    return LicenseRecord(
        license_id=lic_id,
        customer_name=cust_name,
        customer_organization="سازمان نمونه",
        customer_phone="+989120000000",
        edition="Pro",
        license_type="subscription",
        issued_at_utc="2026-11-22T00:00:00Z",
        not_before_utc="2026-11-22T00:00:00Z",
        expires_at_utc="2027-11-22T00:00:00Z",
        max_devices=3,
        device_count=1,
        device_fingerprints=["fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"],
        device_labels={"fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e": "صندوق اصلی"},
        features={"dashboard": True, "products": True},
        key_id="daqiq-prod-v1",
        key_fingerprint="abcdef123456",
        signed_daqlic_content="{}",
        notes="مشتری وفادار",
        status="active",
    )


def test_insert_and_get_record(test_db):
    """Verify storing and retrieving a license record."""
    rec = make_sample_record()
    row_id = test_db.record_license(rec)
    assert row_id > 0

    fetched = test_db.get_license("DQL-1405-0001")
    assert fetched is not None
    assert fetched.customer_name == "فروشگاه آزمایشی"
    assert fetched.edition == "Pro"
    assert fetched.device_fingerprints == rec.device_fingerprints
    assert fetched.device_labels == rec.device_labels


def test_search_by_query_and_filter(test_db):
    """Verify search across customer name, phone, license ID, and HWID."""
    test_db.record_license(make_sample_record(lic_id="DQL-001", cust_name="فروشگاه تهران"))
    test_db.record_license(make_sample_record(lic_id="DQL-002", cust_name="فروشگاه شیراز"))

    # Search by name
    res = test_db.search_licenses(query="شیراز")
    assert len(res) == 1
    assert res[0].license_id == "DQL-002"

    # Search by phone
    res_phone = test_db.search_licenses(query="+98912")
    assert len(res_phone) == 2


def test_statistics_summary(test_db):
    """Verify statistics calculation for metric cards."""
    # Active subscription
    test_db.record_license(make_sample_record(lic_id="DQL-ACT"))

    # Perpetual license
    perp = make_sample_record(lic_id="DQL-PERP")
    perp.license_type = "perpetual"
    perp.expires_at_utc = None
    test_db.record_license(perp)

    # Expired subscription
    exp = make_sample_record(lic_id="DQL-EXP")
    exp.expires_at_utc = "2025-01-01T00:00:00Z"
    test_db.record_license(exp)

    stats = test_db.get_stats()
    assert stats["total"] == 3
    assert stats["active"] == 2  # DQL-ACT and DQL-PERP
    assert stats["expired"] == 1
    assert stats["perpetual"] == 1


def test_export_to_csv(test_db, tmp_path):
    """Verify CSV export functionality."""
    test_db.record_license(make_sample_record())
    csv_file = str(tmp_path / "export.csv")
    test_db.export_to_csv(csv_file)

    assert os.path.isfile(csv_file)
    with open(csv_file, "r", encoding="utf-8-sig") as f:
        content = f.read()
    assert "شناسه لایسنس" in content
    assert "DQL-1405-0001" in content
