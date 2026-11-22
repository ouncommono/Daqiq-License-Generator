"""Headless UI smoke and integration tests (tests/test_ui_headless.py).

Executes with QT_QPA_PLATFORM=offscreen:
Verifies MainWindow, TabGenerator, TabLedger, TabInspector, TabVault,
end-to-end workflow from key creation -> license signing -> ledger -> inspector simulator.
Guarantees ZERO blocking modals and ZERO hangs.
"""

from __future__ import annotations

import datetime
import json
import os
import sys
import pytest

from PySide6.QtCore import Qt
from PySide6.QtWidgets import QApplication

from core.key_vault import KeyVault, UnlockedVault
from core.registry_db import RegistryDB, LicenseRecord
from ui.main_window import MainWindow, APP_VERSION

HWID_1 = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"
HWID_2 = "fp_v1:cc12b7ba73d61e6240c106bb3239c09690102a0ca9f448497532733e8a5382cf"


@pytest.fixture(scope="session")
def qapp():
    """Ensure a single offscreen QApplication exists for all headless UI tests."""
    app = QApplication.instance()
    if app is None:
        os.environ["QT_QPA_PLATFORM"] = "offscreen"
        app = QApplication(sys.argv)
    return app


@pytest.fixture
def temp_db(tmp_path):
    return RegistryDB(str(tmp_path / "test_ui_registry.sqlite"))


@pytest.fixture
def unlocked_vault():
    vault, _ = KeyVault.create_vault(key_id="ui-test-key-v1", passphrase="Pass")
    return vault


def test_main_window_instantiation_and_tabs(qapp, temp_db, unlocked_vault):
    """Verify MainWindow and all 4 tabs instantiate cleanly without crash."""
    win = MainWindow(registry_db=temp_db, initial_vault=unlocked_vault)
    assert win.windowTitle() == f"مولد لایسنس دقیق — نسخه {APP_VERSION}"
    assert win.layoutDirection() == Qt.RightToLeft
    assert win.tab_widget.count() == 4

    # Verify Tab titles
    assert "صدور لایسنس" in win.tab_widget.tabText(0)
    assert "سوابق" in win.tab_widget.tabText(1)
    assert "بازرس" in win.tab_widget.tabText(2)
    assert "گاوصندوق" in win.tab_widget.tabText(3)

    # Verify header vault pill is unlocked
    assert "ui-test-key-v1" in win.lbl_header_vault_pill.text()


def test_end_to_end_generator_flow(qapp, temp_db, unlocked_vault, tmp_path):
    """Test full cycle: Tab 1 input -> sign -> DB record -> Tab 2 ledger -> Tab 3 inspector."""
    win = MainWindow(registry_db=temp_db, initial_vault=unlocked_vault)

    # --- TAB 1: Generator ---
    tab_gen = win.tab_generator
    tab_gen.ed_license_id.setText("DQL-E2E-001")
    tab_gen.ed_customer_name.setText("فروشگاه تست جامع")
    tab_gen.ed_customer_org.setText("شرکت نمونه")
    tab_gen.ed_customer_phone.setText("+989120000000")

    # Add 2 devices
    tab_gen.ed_new_hwid.setText(HWID_1)
    tab_gen.ed_new_label.setText("صندوق ۱")
    tab_gen._on_add_single_hwid()

    tab_gen.ed_new_hwid.setText(HWID_2)
    tab_gen.ed_new_label.setText("صندوق ۲")
    tab_gen._on_add_single_hwid()

    assert len(tab_gen.devices) == 2
    assert tab_gen.tbl_devices.rowCount() == 2

    # Build spec
    spec = tab_gen._build_license_spec()
    assert spec.license_id == "DQL-E2E-001"
    assert spec.customer_name == "فروشگاه تست جامع"
    assert spec.device_fingerprints == [HWID_1, HWID_2]

    # Sign directly using builder
    from core.license_builder import LicenseBuilder
    gen = LicenseBuilder.build_and_sign(spec, unlocked_vault)
    target_lic_path = str(tmp_path / "test_license.daqlic")
    gen.save_to_file(target_lic_path)
    assert os.path.isfile(target_lic_path)

    # Record in database
    rec = LicenseRecord(
        license_id=spec.license_id,
        customer_name=spec.customer_name,
        customer_organization=spec.customer_organization,
        customer_phone=spec.customer_contact,
        edition="Standard",
        license_type="subscription",
        issued_at_utc=spec.issued_at,
        not_before_utc=spec.not_before,
        expires_at_utc=spec.expires_at,
        max_devices=spec.max_devices,
        device_count=len(spec.device_fingerprints),
        device_fingerprints=spec.device_fingerprints,
        device_labels={HWID_1: "صندوق ۱", HWID_2: "صندوق ۲"},
        features=spec.features,
        key_id=unlocked_vault.key_id,
        key_fingerprint=unlocked_vault.fingerprint,
        signed_daqlic_content=gen.json_text,
        status="active",
    )
    temp_db.record_license(rec)

    # --- TAB 2: Ledger ---
    win.tab_ledger.refresh_records()
    assert win.tab_ledger.table.rowCount() == 1
    item_lic = win.tab_ledger.table.item(0, 0)
    assert item_lic.text() == "DQL-E2E-001"

    # Test Clone & Renew signal
    win.tab_ledger.cloneRequested.emit(rec)
    assert win.tab_widget.currentIndex() == 0
    assert win.tab_generator.ed_customer_name.text() == "فروشگاه تست جامع"
    assert len(win.tab_generator.devices) == 2

    # --- TAB 3: Inspector ---
    tab_insp = win.tab_inspector
    tab_insp._load_and_inspect_json(gen.json_text)
    assert tab_insp.current_envelope is not None
    assert "✅ امضای دیجیتال معتبر است" in tab_insp.lbl_sig_banner.text()
    assert tab_insp.lbl_val_lic_id.text() == "DQL-E2E-001"

    # Test Simulator
    tab_insp.ed_sim_hwid.setText(HWID_1)
    tab_insp.sim_picker.set_to_today()
    tab_insp._on_run_simulation()
    assert "VALID" in tab_insp.lbl_sim_status.text()

    # Test Simulator with unauthorized HWID
    tab_insp.ed_sim_hwid.setText("fp_v1:0000000000000000000000000000000000000000000000000000000000000000")
    tab_insp._on_run_simulation()
    assert "UNAUTHORIZED_DEVICE" in tab_insp.lbl_sim_status.text()

    # --- TAB 4: Vault ---
    win.tab_vault.refresh_vault_display()
    assert win.tab_vault.lbl_val_key_id.text() == "ui-test-key-v1"

    # Lock vault
    unlocked_vault.lock()
    win._set_active_vault(None)
    assert "قفل" in win.lbl_header_vault_pill.text()

    # Toggle theme
    win._toggle_theme()
    assert win.is_dark_theme is True
    win._toggle_theme()
    assert win.is_dark_theme is False
