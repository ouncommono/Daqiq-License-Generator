"""Tab 3: License Inspector & Simulator (بررسی و شبیه‌ساز لایسنس .daqlic).

Inspects, verifies signatures, decodes license contents with Jalali dates,
and simulates Daqiq v4.9.0 verification outcomes across test devices and dates.
"""

from __future__ import annotations

import base64
import datetime
import json
import os
from typing import Optional, Dict, Any, List

from PySide6.QtCore import Qt
from PySide6.QtGui import QFont, QColor
from PySide6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QGridLayout,
    QGroupBox,
    QLabel,
    QLineEdit,
    QPushButton,
    QTextEdit,
    QFileDialog,
    QMessageBox,
    QScrollArea,
    QFrame,
    QTableWidget,
    QTableWidgetItem,
    QHeaderView,
)

import jdatetime

from core.key_vault import UnlockedVault
from core.hwid_parser import clean_hwid_string, validate_hwid_string
from core.license_builder import (
    canonicalize_payload,
    DOMAIN_PREFIX,
    ALL_FEATURE_KEYS,
)
from .jalali_picker import JalaliDateWidget
from .tab_generator import FEATURE_TRANSLATIONS


class TabInspector(QWidget):
    """Tab 3: License Inspector & Simulator."""

    def __init__(
        self,
        get_active_vault_callback,
        parent: Optional[QWidget] = None,
    ):
        super().__init__(parent)
        self.get_active_vault = get_active_vault_callback
        self.current_envelope: Optional[Dict[str, Any]] = None
        self.setLayoutDirection(Qt.RightToLeft)
        self._build_ui()

    def _build_ui(self) -> None:
        scroll = QScrollArea(self)
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.NoFrame)

        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(14)

        # 1. File Loader Card
        layout.addWidget(self._build_loader_card())

        # 2. Decoded Details & Verification Card
        layout.addWidget(self._build_details_card())

        # 3. Simulator Card
        layout.addWidget(self._build_simulator_card())

        scroll.setWidget(container)
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(scroll)

    def _build_loader_card(self) -> QGroupBox:
        box = QGroupBox("بارگذاری و اعتبارسنجی فایل لایسنس (.daqlic)")
        layout = QVBoxLayout(box)
        layout.setSpacing(10)

        row = QHBoxLayout()
        self.btn_open_file = QPushButton("📂 انتخاب فایل .daqlic از دیسک...")
        self.btn_open_file.clicked.connect(self._on_open_file)
        row.addWidget(self.btn_open_file)

        self.btn_paste_json = QPushButton("📋 چسباندن متن JSON یا توکن لایسنس")
        self.btn_paste_json.setObjectName("BtnSecondary")
        self.btn_paste_json.clicked.connect(self._on_paste_text)
        row.addWidget(self.btn_paste_json)

        row.addStretch()
        layout.addLayout(row)

        # Public key input (defaults to active vault, editable if testing other keys)
        key_row = QHBoxLayout()
        key_row.addWidget(QLabel("کلید عمومی جهت بررسی امضا (Base64):"))
        self.ed_pub_key = QLineEdit()
        self.ed_pub_key.setPlaceholderText("پیش‌فرض: کلید گاوصندوق فعال جاری")
        self.ed_pub_key.setStyleSheet("font-family: Consolas, monospace;")
        key_row.addWidget(self.ed_pub_key, 1)

        self.btn_use_vault_key = QPushButton("دریافت از گاوصندوق")
        self.btn_use_vault_key.setObjectName("BtnSecondary")
        self.btn_use_vault_key.clicked.connect(self._sync_vault_public_key)
        key_row.addWidget(self.btn_use_vault_key)

        layout.addLayout(key_row)
        return box

    def _build_details_card(self) -> QGroupBox:
        self.box_details = QGroupBox("نتیجه بررسی رمزنگاری و مشخصات لایسنس")
        layout = QVBoxLayout(self.box_details)
        layout.setSpacing(10)

        # Signature Status Banner
        self.lbl_sig_banner = QLabel("هیچ فایلی برای بررسی بارگذاری نشده است.")
        self.lbl_sig_banner.setAlignment(Qt.AlignCenter)
        self.lbl_sig_banner.setStyleSheet(
            "background-color: #e2e8f0; color: #475569; font-size: 13px; font-weight: bold; padding: 10px; border-radius: 6px;"
        )
        layout.addWidget(self.lbl_sig_banner)

        # Grid of fields
        self.grid_fields = QGridLayout()
        self.grid_fields.setSpacing(8)

        self.lbl_val_lic_id = QLabel("-")
        self.lbl_val_cust = QLabel("-")
        self.lbl_val_type = QLabel("-")
        self.lbl_val_issued = QLabel("-")
        self.lbl_val_not_before = QLabel("-")
        self.lbl_val_expires = QLabel("-")
        self.lbl_val_devices = QLabel("-")
        self.lbl_val_versions = QLabel("-")

        self.grid_fields.addWidget(QLabel("شماره لایسنس:"), 0, 0)
        self.grid_fields.addWidget(self.lbl_val_lic_id, 0, 1)
        self.grid_fields.addWidget(QLabel("نام مشتری:"), 0, 2)
        self.grid_fields.addWidget(self.lbl_val_cust, 0, 3)

        self.grid_fields.addWidget(QLabel("نوع لایسنس:"), 1, 0)
        self.grid_fields.addWidget(self.lbl_val_type, 1, 1)
        self.grid_fields.addWidget(QLabel("سقف دستگاه:"), 1, 2)
        self.grid_fields.addWidget(self.lbl_val_devices, 1, 3)

        self.grid_fields.addWidget(QLabel("تاریخ شروع (شمسی):"), 2, 0)
        self.grid_fields.addWidget(self.lbl_val_not_before, 2, 1)
        self.grid_fields.addWidget(QLabel("تاریخ انقضا (شمسی):"), 2, 2)
        self.grid_fields.addWidget(self.lbl_val_expires, 2, 3)

        self.grid_fields.addWidget(QLabel("تاریخ صدور (شمسی):"), 3, 0)
        self.grid_fields.addWidget(self.lbl_val_issued, 3, 1)
        self.grid_fields.addWidget(QLabel("نسخه‌های مجاز:"), 3, 2)
        self.grid_fields.addWidget(self.lbl_val_versions, 3, 3)

        layout.addLayout(self.grid_fields)

        # Features grid
        layout.addWidget(QLabel("ماژول‌ها و دسترسی‌های مجاز در لایسنس:"))
        self.features_layout = QGridLayout()
        self.features_layout.setSpacing(6)
        self._feature_labels: Dict[str, QLabel] = {}

        for i, k in enumerate(ALL_FEATURE_KEYS):
            row = i // 2
            col = i % 2
            lbl = QLabel(f"• {FEATURE_TRANSLATIONS.get(k, k)}: -")
            lbl.setStyleSheet("font-size: 11px;")
            self._feature_labels[k] = lbl
            self.features_layout.addWidget(lbl, row, col)

        layout.addLayout(self.features_layout)

        # Authorized Devices
        layout.addWidget(QLabel("فهرست شناسه‌های سخت‌افزاری مجاز (Node-Locked HWIDs):"))
        self.txt_devices = QTextEdit()
        self.txt_devices.setReadOnly(True)
        self.txt_devices.setFixedHeight(70)
        self.txt_devices.setFont(QFont("Consolas", 10))
        layout.addWidget(self.txt_devices)

        return self.box_details

    def _build_simulator_card(self) -> QGroupBox:
        box = QGroupBox("شبیه‌ساز رفتار و انطباق در Daqiq v4.9.0 (Verification Simulator)")
        layout = QVBoxLayout(box)
        layout.setSpacing(10)

        hint = QLabel(
            "شناسه سخت‌افزاری دستگاه مشتری و یک تاریخ فرضی را وارد نمایید تا پاسخ دقیق نرم‌افزار Daqiq v4.9.0 "
            "را در زمان اجرای لایسنس بدون نیاز به نصب برنامه شبیه‌سازی کنید:"
        )
        hint.setStyleSheet("font-size: 11px; color: #475569;")
        layout.addWidget(hint)

        sim_row = QHBoxLayout()
        sim_row.addWidget(QLabel("شناسه سخت‌افزاری تست:"))
        self.ed_sim_hwid = QLineEdit()
        self.ed_sim_hwid.setPlaceholderText("fp_v1:...")
        self.ed_sim_hwid.setStyleSheet("font-family: Consolas, monospace;")
        sim_row.addWidget(self.ed_sim_hwid, 2)

        sim_row.addWidget(QLabel("نسخه برنامه:"))
        self.ed_sim_app_ver = QLineEdit("4.9.0")
        self.ed_sim_app_ver.setFixedWidth(75)
        sim_row.addWidget(self.ed_sim_app_ver)

        layout.addLayout(sim_row)

        date_row = QHBoxLayout()
        self.sim_picker = JalaliDateWidget(label_text="تاریخ اجرای آزمایشی (فرضی):")
        date_row.addWidget(self.sim_picker, 1)

        self.btn_run_sim = QPushButton("🔍 اجرای شبیه‌ساز انطباق Daqiq")
        self.btn_run_sim.setObjectName("BtnSuccess")
        self.btn_run_sim.clicked.connect(self._on_run_simulation)
        date_row.addWidget(self.btn_run_sim)

        layout.addLayout(date_row)

        # Simulation Result Display
        self.frame_sim_result = QFrame()
        self.frame_sim_result.setObjectName("MetricCard")
        f_layout = QVBoxLayout(self.frame_sim_result)
        f_layout.setContentsMargins(12, 10, 12, 10)

        self.lbl_sim_status = QLabel("جهت مشاهده نتیجه شبیه‌سازی، روی دکمه اجرای شبیه‌ساز کلیک کنید.")
        self.lbl_sim_status.setStyleSheet("font-size: 13px; font-weight: bold; color: #475569;")
        f_layout.addWidget(self.lbl_sim_status)

        self.lbl_sim_detail = QLabel("")
        self.lbl_sim_detail.setWordWrap(True)
        self.lbl_sim_detail.setStyleSheet("font-size: 11px; color: #64748b; margin-top: 4px;")
        f_layout.addWidget(self.lbl_sim_detail)

        layout.addWidget(self.frame_sim_result)
        return box

    def _sync_vault_public_key(self) -> None:
        vault = self.get_active_vault()
        if vault and vault.public_key_b64:
            self.ed_pub_key.setText(vault.public_key_b64)
        else:
            QMessageBox.information(
                self,
                "گاوصندوق فعال نیست",
                "ابتدا گاوصندوق کلید را در تب مدیریت کلیدها باز کنید یا کلید عمومی را به صورت دستی وارد نمایید.",
            )

    def _on_open_file(self) -> None:
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "انتخاب فایل لایسنس دقیق",
            "",
            "فایل‌های لایسنس دقیق (*.daqlic);;تمام فایل‌ها (*.*)",
        )
        if not file_path:
            return

        try:
            with open(file_path, "r", encoding="utf-8") as f:
                content = f.read()
            self._load_and_inspect_json(content)
        except Exception as e:
            QMessageBox.critical(self, "خطا در خواندن فایل", str(e))

    def _on_paste_text(self) -> None:
        text, ok = QMessageBox.getText(self, "چسباندن لایسنس", "متن JSON یا توکن لایسنس را وارد کنید:")
        if ok and text:
            self._load_and_inspect_json(text)

    def _load_and_inspect_json(self, raw_text: str) -> None:
        clean = raw_text.strip()
        # Handle compact token DAQIQ1.<payload_b64>.<sig_b64>
        if clean.startswith("DAQIQ1."):
            parts = clean.split(".")
            if len(parts) == 3:
                try:
                    payload_bytes = base64.b64decode(parts[1])
                    payload_dict = json.loads(payload_bytes.decode("utf-8"))
                    doc = {
                        "envelope_version": 1,
                        "algorithm": "Ed25519",
                        "key_id": payload_dict.get("key_id", "unknown"),
                        "payload": payload_dict,
                        "signature": parts[2],
                    }
                except Exception as e:
                    QMessageBox.warning(self, "خطا", f"عدم توانایی در رمزگشایی توکن لایسنس: {e}")
                    return
            else:
                QMessageBox.warning(self, "خطا", "ساختار توکن متنی معتبر نیست.")
                return
        else:
            try:
                doc = json.loads(clean)
            except Exception as e:
                QMessageBox.warning(self, "خطا در فرمت JSON", f"متن وارد شده JSON معتبر نیست: {e}")
                return

        if not isinstance(doc, dict) or "payload" not in doc:
            QMessageBox.warning(self, "خطا", "فایل بارگذاری‌شده فاقد ساختار پاکت لایسنس معتبر است.")
            return

        self.current_envelope = doc
        self._display_envelope(doc)

    def _display_envelope(self, doc: Dict[str, Any]) -> None:
        p = doc.get("payload", {})
        cust = p.get("customer", {})

        # Basic fields
        self.lbl_val_lic_id.setText(p.get("license_id", "-"))
        cust_name = cust.get("name", "-")
        if cust.get("organization"):
            cust_name += f" ({cust.get('organization')})"
        self.lbl_val_cust.setText(cust_name)

        lic_type = "دائمی (نامحدود)" if p.get("license_type") == "perpetual" else "اشتراکی (زمان‌دار)"
        self.lbl_val_type.setText(lic_type)

        fps = p.get("device_fingerprints", [])
        self.lbl_val_devices.setText(f"{len(fps)} از {p.get('max_devices', 1)}")

        # Jalali dates
        from .tab_ledger import format_iso_to_jalali
        self.lbl_val_issued.setText(format_iso_to_jalali(p.get("issued_at")))
        self.lbl_val_not_before.setText(format_iso_to_jalali(p.get("not_before")))
        self.lbl_val_expires.setText(format_iso_to_jalali(p.get("expires_at")))

        min_v = p.get("min_app_version") or "ندارد"
        max_v = p.get("max_app_version") or "نامحدود"
        self.lbl_val_versions.setText(f"حداقل: {min_v} | حداکثر: {max_v}")

        # Features
        features = p.get("features", {})
        for k, lbl in self._feature_labels.items():
            en = features.get(k, False)
            icon = "✅ فعال" if en else "❌ غیرفعال"
            color = "#0f766e" if en else "#be123c"
            lbl.setText(f"• {FEATURE_TRANSLATIONS.get(k, k)}: {icon}")
            lbl.setStyleSheet(f"color: {color}; font-size: 11px;")

        # Authorized devices
        self.txt_devices.setPlainText("\n".join(fps))
        if fps:
            self.ed_sim_hwid.setText(fps[0])

        # Cryptographic Signature Verification
        self._verify_signature(doc)

    def _verify_signature(self, doc: Dict[str, Any]) -> None:
        pub_b64 = self.ed_pub_key.text().strip()
        if not pub_b64:
            vault = self.get_active_vault()
            if vault and vault.public_key_b64:
                pub_b64 = vault.public_key_b64
                self.ed_pub_key.setText(pub_b64)

        if not pub_b64:
            self.lbl_sig_banner.setText("⚠️ امضا قابل بررسی نیست: کلید عمومی وارد نشده است.")
            self.lbl_sig_banner.setStyleSheet(
                "background-color: #fef3c7; color: #b45309; font-size: 13px; font-weight: bold; padding: 10px; border-radius: 6px;"
            )
            return

        try:
            from cryptography.hazmat.primitives.asymmetric import ed25519
            from cryptography.exceptions import InvalidSignature

            pub_bytes = base64.b64decode(pub_b64)
            if len(pub_bytes) != 32:
                raise ValueError("طول کلید عمومی باید ۳۲ بایت باشد.")
            pub_key = ed25519.Ed25519PublicKey.from_public_bytes(pub_bytes)

            sig_b64 = doc.get("signature", "")
            sig_bytes = base64.b64decode(sig_b64)
            canonical_bytes = canonicalize_payload(doc.get("payload", {}))
            data_to_verify = DOMAIN_PREFIX + canonical_bytes

            pub_key.verify(sig_bytes, data_to_verify)

            self.lbl_sig_banner.setText("✅ امضای دیجیتال معتبر است (تطابق کامل با کلید عمومی و پروتکل Ed25519)")
            self.lbl_sig_banner.setStyleSheet(
                "background-color: #d1fae5; color: #065f46; font-size: 13px; font-weight: bold; padding: 10px; border-radius: 6px;"
            )
        except InvalidSignature:
            self.lbl_sig_banner.setText("❌ امضای دیجیتال نامعتبر است (محتوای لایسنس دستکاری شده یا کلید تطابق ندارد)")
            self.lbl_sig_banner.setStyleSheet(
                "background-color: #ffe4e6; color: #9f1239; font-size: 13px; font-weight: bold; padding: 10px; border-radius: 6px;"
            )
        except Exception as e:
            self.lbl_sig_banner.setText(f"❌ خطا در اعتبارسنجی امضا: {e}")
            self.lbl_sig_banner.setStyleSheet(
                "background-color: #ffe4e6; color: #9f1239; font-size: 13px; font-weight: bold; padding: 10px; border-radius: 6px;"
            )

    def _on_run_simulation(self) -> None:
        """Run simulated evaluation identical to Daqiq Desktop v4.9.0."""
        if not self.current_envelope:
            QMessageBox.warning(self, "خطا", "ابتدا یک فایل لایسنس را بارگذاری کنید.")
            return

        test_hwid = clean_hwid_string(self.ed_sim_hwid.text())
        if not test_hwid:
            QMessageBox.warning(self, "خطا", "لطفاً شناسه سخت‌افزاری دستگاه را برای شبیه‌سازی وارد کنید.")
            return

        test_iso = self.sim_picker.get_utc_iso_string()
        from core.license_builder import parse_iso8601_utc
        test_now_utc = parse_iso8601_utc(test_iso)

        app_ver = self.ed_sim_app_ver.text().strip() or "4.9.0"

        p = self.current_envelope.get("payload", {})

        # 1. Product check
        if p.get("product_id") != "daqiq.desktop":
            self._set_sim_result("WRONG_PRODUCT", "شناسه محصول با daqiq.desktop همخوانی ندارد.", "#be123c")
            return

        # 2. Hardware check
        authorized_fps = [clean_hwid_string(fp) for fp in p.get("device_fingerprints", [])]
        if test_hwid not in authorized_fps:
            self._set_sim_result(
                "UNAUTHORIZED_DEVICE",
                f"شناسه سخت‌افزاری دستگاه ({test_hwid[:16]}...) در فهرست دستگاه‌های مجاز این لایسنس نیست.",
                "#be123c",
            )
            return

        # 3. Version bounds check
        from packaging import version as pkg_version
        try:
            curr_v = pkg_version.parse(app_ver)
            if p.get("min_app_version"):
                min_v = pkg_version.parse(p.get("min_app_version"))
                if curr_v < min_v:
                    self._set_sim_result(
                        "UNSUPPORTED_APP_VERSION",
                        f"نسخه برنامه ({app_ver}) کمتر از حداقل نسخه مجاز ({p.get('min_app_version')}) است.",
                        "#be123c",
                    )
                    return
            if p.get("max_app_version"):
                max_v = pkg_version.parse(p.get("max_app_version"))
                if curr_v > max_v:
                    self._set_sim_result(
                        "UNSUPPORTED_APP_VERSION",
                        f"نسخه برنامه ({app_ver}) بیشتر از حداکثر نسخه مجاز ({p.get('max_app_version')}) است.",
                        "#be123c",
                    )
                    return
        except Exception as e:
            self._set_sim_result("CORRUPTED_LICENSE", f"خطا در بررسی نسخه: {e}", "#be123c")
            return

        # 4. Date bounds check
        dt_start = parse_iso8601_utc(p.get("not_before"))
        if test_now_utc < dt_start:
            self._set_sim_result(
                "NOT_YET_VALID",
                f"تاریخ تست قبل از تاریخ شروع اعتبار لایسنس ({p.get('not_before')}) است.",
                "#d97706",
            )
            return

        if p.get("license_type") == "subscription":
            exp_str = p.get("expires_at")
            if not exp_str:
                self._set_sim_result("CORRUPTED_LICENSE", "لایسنس اشتراکی فاقد تاریخ انقضا است.", "#be123c")
                return
            dt_exp = parse_iso8601_utc(exp_str)
            if test_now_utc >= dt_exp:
                self._set_sim_result(
                    "EXPIRED",
                    f"تاریخ تست پس از تاریخ انقضای لایسنس ({exp_str}) است. اشتراک منقضی شده تلقی می‌شود.",
                    "#be123c",
                )
                return

        # Passed all checks!
        self._set_sim_result(
            "VALID",
            "نرم‌افزار Daqiq v4.9.0 این لایسنس را تأیید کرده و دسترسی به تمام ۱۰ ماژول بر اساس دسترسی‌ها باز خواهد شد.",
            "#0f766e",
        )

    def _set_sim_result(self, code: str, detail: str, color_hex: str) -> None:
        self.lbl_sim_status.setText(f"نتیجه شبیه‌سازی: {code}")
        self.lbl_sim_status.setStyleSheet(f"font-size: 14px; font-weight: bold; color: {color_hex};")
        self.lbl_sim_detail.setText(detail)
