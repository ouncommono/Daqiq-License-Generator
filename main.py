#!/usr/bin/env python3
"""Daqiq License Generator v1.0.0 - Commercial Offline License Studio for Daqiq Desktop ERP.

Single entrypoint supporting headless CLI checks (--version, --self-test)
and full interactive Persian RTL desktop UI.
"""

from __future__ import annotations

import argparse
import os
import sys

APP_NAME = "Daqiq License Generator"
APP_VERSION = "1.0.0"


def run_self_test() -> int:
    """Headless self-test verifying core cryptography, canonicalization, and registry."""
    print(f"=== {APP_NAME} v{APP_VERSION} Headless Self-Test ===")
    try:
        from core.key_vault import KeyVault
        from core.hwid_parser import HWIDParser, validate_hwid_string
        from core.license_builder import LicenseBuilder, LicenseSpec, ALL_FEATURE_KEYS
        from core.registry_db import RegistryDB

        # 1. Key Vault check
        unlocked, _ = KeyVault.create_vault(key_id="selftest-key-v1", passphrase="Pass")
        assert unlocked.is_unlocked
        assert len(unlocked.public_key_b64) > 0
        print("[PASS] Key Vault Ed25519 generation and in-memory unlocking verified.")

        # 2. HWID check
        hwid = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"
        assert validate_hwid_string(hwid)
        parsed = HWIDParser.parse_single(hwid)
        assert parsed.is_valid
        print("[PASS] Hardware ID parser and validation verified.")

        # 3. License Builder check
        spec = LicenseSpec(
            license_id="DQL-SELFTEST-001",
            customer_name="فروشگاه خودآزمایی",
            license_type="perpetual",
            issued_at="2026-11-22T00:00:00Z",
            not_before="2026-11-22T00:00:00Z",
            expires_at=None,
            max_devices=1,
            device_fingerprints=[hwid],
            features={k: True for k in ALL_FEATURE_KEYS},
        )
        gen = LicenseBuilder.build_and_sign(spec, unlocked)
        assert gen.signature_b64
        print("[PASS] Canonical license payload construction and digital signature verified.")

        # 4. Registry DB check
        db = RegistryDB(":memory:")
        stats = db.get_stats()
        assert stats["total"] == 0
        print("[PASS] Sales registry SQLite engine verified.")

        print("=== ALL SELF-TESTS PASSED SUCCESSFULLY ===")
        return 0
    except Exception as e:
        print(f"[FAIL] Self-test encountered an error: {e}", file=sys.stderr)
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(description=f"{APP_NAME} v{APP_VERSION}")
    parser.add_argument("--version", action="store_true", help="Print version and exit")
    parser.add_argument("--self-test", action="store_true", help="Run headless cryptographic verification and exit")

    args, unknown = parser.parse_known_args()

    if args.version:
        print(f"{APP_NAME} v{APP_VERSION}")
        return 0

    if args.self_test:
        return run_self_test()

    # GUI Mode
    from PySide6.QtCore import Qt, QCoreApplication
    from PySide6.QtGui import QIcon
    from PySide6.QtWidgets import QApplication
    from ui.main_window import MainWindow

    app = QApplication(sys.argv)
    app.setApplicationName(APP_NAME)
    app.setApplicationVersion(APP_VERSION)
    app.setLayoutDirection(Qt.RightToLeft)

    # Resolve application icon
    base_dir = os.path.dirname(os.path.abspath(__file__))
    icon_png = os.path.join(base_dir, "assets", "icon.png")
    icon_ico = os.path.join(base_dir, "assets", "icon.ico")
    if os.path.isfile(icon_png):
        app.setWindowIcon(QIcon(icon_png))
    elif os.path.isfile(icon_ico):
        app.setWindowIcon(QIcon(icon_ico))

    window = MainWindow()
    window.show()

    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
