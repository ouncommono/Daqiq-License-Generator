"""Tab 1: Main License Generation Studio (تولید لایسنس تجاری دقیق).

Offers complete PR #38 customization in clean Persian RTL layout:
- Customer & License ID generator
- Multi-Device Binding (single, bulk paste, .daqreq import)
- Jalali-first validity and quick duration chips
- 10 Modular feature entitlements and version bounds
- Live canonical preview and atomic .daqlic signing
"""

from __future__ import annotations

import datetime
import json
import os
import re
from typing import Optional, List, Dict, Any

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QIcon, QFont, QColor
from PySide6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QGridLayout,
    QGroupBox,
    QLabel,
    QLineEdit,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QHeaderView,
    QComboBox,
    QRadioButton,
    QButtonGroup,
    QSpinBox,
    QCheckBox,
    QTextEdit,
    QFileDialog,
    QMessageBox,
    QDialog,
    QScrollArea,
    QFrame,
    QSizePolicy,
)

import jdatetime
import pyperclip

from core.key_vault import UnlockedVault
from core.hwid_parser import (
    HWIDParser,
    DeviceEntry,
    validate_hwid_string,
    clean_hwid_string,
)
from core.license_builder import (
    LicenseBuilder,
    LicenseSpec,
    GeneratedLicense,
    LicenseValidationError,
    PLAN_PRESETS,
    ALL_FEATURE_KEYS,
)
from core.registry_db import RegistryDB, LicenseRecord
from .jalali_picker import JalaliDateWidget

FEATURE_TRANSLATIONS = {
    "dashboard": "📊 داشبورد و آمار مدیریتی",
    "products": "📦 مدیریت انبار و کالاها",
    "batches": "🚚 ثبت خرید و محموله‌ها",
    "sales": "🛒 ثبت فروش و صندوقداری",
    "services": "🛠️ میز خدمت و تعمیرات",
    "upgrades": "🔧 ارتقای کالا و خدمات فنی",
    "suppliers_payments": "💳 حسابداری تأمین‌کنندگان",
    "reports": "📈 گزارش‌های جامع و مالی",
    "employees": "👤 مدیریت کاربران و دسترسی‌ها",
    "settings": "⚙️ تنظیمات و پشتیبان‌گیری",
}


class BulkPasteDialog(QDialog):
    """Modal dialog for pasting multiple Hardware IDs at once."""

    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setWindowTitle("افزودن گروهی شناسه‌های سخت‌افزاری (Bulk Paste)")
        self.setLayoutDirection(Qt.RightToLeft)
        self.resize(580, 420)
        self._parsed_devices: List[DeviceEntry] = []
        self._build_ui()

    def _build_ui(self) -> None:
        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(10)

        hint = QLabel(
            "شناسه‌های سخت‌افزاری دریافت شده از مشتریان را در کادر زیر وارد کنید.\n"
            "می‌توانید در هر سطر یک شناسه وارد کنید یا از برچسب مانند «صندوق ۱: fp_v1:...» استفاده نمایید:"
        )
        hint.setStyleSheet("font-size: 11px; color: #475569;")
        layout.addWidget(hint)

        self.text_edit = QTextEdit()
        self.text_edit.setPlaceholderText(
            "صندوق اصلی: fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e\n"
            "لپتاپ مدیریت: fp_v1:cc12b7ba73d61e6240c106bb3239c09690102a0ca9f448497532733e8a5382cf"
        )
        self.text_edit.textChanged.connect(self._on_text_changed)
        layout.addWidget(self.text_edit, 1)

        self.lbl_stats = QLabel("تعداد شناسه‌های معتبر: ۰ | نامعتبر: ۰ | تکراری: ۰")
        self.lbl_stats.setStyleSheet("font-size: 11px; font-weight: bold; color: #0284c7;")
        layout.addWidget(self.lbl_stats)

        btn_row = QHBoxLayout()
        self.btn_import = QPushButton("افزودن شناسه‌های معتبر به لایسنس")
        self.btn_import.setObjectName("BtnSuccess")
        self.btn_import.clicked.connect(self.accept)
        btn_row.addWidget(self.btn_import)

        btn_cancel = QPushButton("انصراف")
        btn_cancel.setObjectName("BtnSecondary")
        btn_cancel.clicked.connect(self.reject)
        btn_row.addWidget(btn_cancel)

        layout.addLayout(btn_row)

    def _on_text_changed(self) -> None:
        text = self.text_edit.toPlainText()
        batch = HWIDParser.parse_bulk_text(text)
        self._parsed_devices = [d for d in batch.devices if d.is_valid]
        self.lbl_stats.setText(
            f"تعداد شناسه‌های معتبر: {batch.valid_count} | "
            f"نامعتبر: {batch.invalid_count} | "
            f"تکراری: {batch.duplicate_count}"
        )

    def get_valid_devices(self) -> List[DeviceEntry]:
        return self._parsed_devices


class TabGenerator(QWidget):
    """Main License Generator Tab."""

    licenseGenerated = Signal(str)  # Emits license_id on successful generation

    def __init__(
        self,
        get_active_vault_callback,
        registry_db: RegistryDB,
        parent: Optional[QWidget] = None,
    ):
        super().__init__(parent)
        self.get_active_vault = get_active_vault_callback
        self.registry_db = registry_db
        self.devices: List[DeviceEntry] = []
        self._feature_checkboxes: Dict[str, QCheckBox] = {}

        self.setLayoutDirection(Qt.RightToLeft)
        self._build_ui()
        self._apply_preset("standard")
        self._auto_generate_license_id()

    def _build_ui(self) -> None:
        scroll = QScrollArea(self)
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.NoFrame)

        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(14)

        # 1. Customer & Plan Preset Card
        layout.addWidget(self._build_customer_card())

        # 2. Multi-Device Binding Card
        layout.addWidget(self._build_device_card())

        # 3. Validity & Scheduling Card (Jalali-First)
        layout.addWidget(self._build_validity_card())

        # 4. Features & Entitlements Card
        layout.addWidget(self._build_features_card())

        # 5. Live Preview & Primary Action Footer
        layout.addWidget(self._build_footer_card())

        scroll.setWidget(container)
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(scroll)

    def _build_customer_card(self) -> QGroupBox:
        box = QGroupBox("مشخصات مشتری و قالب پلن (Plan Preset)")
        layout = QGridLayout(box)
        layout.setSpacing(10)

        # License ID
        layout.addWidget(QLabel("شماره لایسنس:"), 0, 0)
        lic_row = QHBoxLayout()
        self.ed_license_id = QLineEdit()
        self.ed_license_id.setPlaceholderText("مثال: DQL-1405-0001")
        lic_row.addWidget(self.ed_license_id, 1)

        self.btn_gen_id = QPushButton("🔄 تولید خودکار")
        self.btn_gen_id.setObjectName("BtnSecondary")
        self.btn_gen_id.setFixedWidth(110)
        self.btn_gen_id.clicked.connect(self._auto_generate_license_id)
        lic_row.addWidget(self.btn_gen_id)
        layout.addLayout(lic_row, 0, 1)

        # Preset Plan Dropdown
        layout.addWidget(QLabel("قالب آماده پلن:"), 0, 2)
        self.cb_preset = QComboBox()
        for k, v in PLAN_PRESETS.items():
            self.cb_preset.addItem(v["label_fa"], k)
        self.cb_preset.currentIndexChanged.connect(self._on_preset_changed)
        layout.addWidget(self.cb_preset, 0, 3)

        # Customer Name
        layout.addWidget(QLabel("نام مشتری / فروشگاه:*"), 1, 0)
        self.ed_customer_name = QLineEdit()
        self.ed_customer_name.setPlaceholderText("نام فروشگاه یا خریدار محترم")
        layout.addWidget(self.ed_customer_name, 1, 1)

        # Customer Organization
        layout.addWidget(QLabel("سازمان / مجموعه:"), 1, 2)
        self.ed_customer_org = QLineEdit()
        self.ed_customer_org.setPlaceholderText("نام شرکت یا برند تجاری (اختیاری)")
        layout.addWidget(self.ed_customer_org, 1, 3)

        # Customer Phone
        layout.addWidget(QLabel("شماره تماس:"), 2, 0)
        self.ed_customer_phone = QLineEdit()
        self.ed_customer_phone.setPlaceholderText("۰۹۱۲... یا شناسه ارتباطی")
        layout.addWidget(self.ed_customer_phone, 2, 1)

        # Notes
        layout.addWidget(QLabel("توضیحات و یادداشت:"), 2, 2)
        self.ed_notes = QLineEdit()
        self.ed_notes.setPlaceholderText("یادداشت داخلی برای ثبت در سوابق...")
        layout.addWidget(self.ed_notes, 2, 3)

        return box

    def _build_device_card(self) -> QGroupBox:
        box = QGroupBox("دستگاه‌ها و شناسه‌های سخت‌افزاری (Multi-Device Binding)")
        layout = QVBoxLayout(box)
        layout.setSpacing(10)

        # Input Row
        in_row = QHBoxLayout()
        in_row.addWidget(QLabel("شناسه دستگاه:"))

        self.ed_new_hwid = QLineEdit()
        self.ed_new_hwid.setPlaceholderText("fp_v1:...")
        self.ed_new_hwid.setStyleSheet("font-family: Consolas, monospace;")
        in_row.addWidget(self.ed_new_hwid, 1)

        self.ed_new_label = QLineEdit()
        self.ed_new_label.setPlaceholderText("برچسب دستگاه (مثلاً: صندوق ۱)")
        self.ed_new_label.setFixedWidth(170)
        in_row.addWidget(self.ed_new_label)

        self.btn_add_hwid = QPushButton("➕ افزودن")
        self.btn_add_hwid.clicked.connect(self._on_add_single_hwid)
        in_row.addWidget(self.btn_add_hwid)

        self.btn_bulk_hwid = QPushButton("📋 افزودن گروهی")
        self.btn_bulk_hwid.setObjectName("BtnSecondary")
        self.btn_bulk_hwid.clicked.connect(self._on_open_bulk_dialog)
        in_row.addWidget(self.btn_bulk_hwid)

        self.btn_import_daqreq = QPushButton("📂 فایل مشخصات (.daqreq)")
        self.btn_import_daqreq.setObjectName("BtnSecondary")
        self.btn_import_daqreq.clicked.connect(self._on_import_daqreq)
        in_row.addWidget(self.btn_import_daqreq)

        layout.addLayout(in_row)

        # Devices Table
        self.tbl_devices = QTableWidget(0, 5)
        self.tbl_devices.setHorizontalHeaderLabels([
            "ردیف",
            "برچسب دستگاه",
            "شناسه سخت‌افزاری (fp_v1)",
            "وضعیت فرمت",
            "عملیات",
        ])
        self.tbl_devices.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeToContents)
        self.tbl_devices.horizontalHeader().setSectionResizeMode(1, QHeaderView.Interactive)
        self.tbl_devices.horizontalHeader().setSectionResizeMode(2, QHeaderView.Stretch)
        self.tbl_devices.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeToContents)
        self.tbl_devices.horizontalHeader().setSectionResizeMode(4, QHeaderView.ResizeToContents)
        self.tbl_devices.setMinimumHeight(140)
        layout.addWidget(self.tbl_devices)

        # Quota Row
        quota_row = QHBoxLayout()
        quota_row.addWidget(QLabel("حداکثر تعداد دستگاه‌های مجاز (max_devices):"))
        self.sb_max_devices = QSpinBox()
        self.sb_max_devices.setRange(1, 100)
        self.sb_max_devices.setValue(1)
        self.sb_max_devices.setFixedWidth(80)
        quota_row.addWidget(self.sb_max_devices)

        self.lbl_devices_count = QLabel("تعداد فعلی: ۰ دستگاه")
        self.lbl_devices_count.setStyleSheet("font-weight: bold; color: #0284c7; margin-right: 12px;")
        quota_row.addWidget(self.lbl_devices_count)
        quota_row.addStretch()

        layout.addLayout(quota_row)
        return box

    def _build_validity_card(self) -> QGroupBox:
        box = QGroupBox("زمان‌بندی و اعتبار لایسنس (تقویم جلالی)")
        layout = QVBoxLayout(box)
        layout.setSpacing(12)

        # License Type selection
        type_row = QHBoxLayout()
        type_row.addWidget(QLabel("نوع لایسنس:"))

        self.rb_subscription = QRadioButton("اشتراکی (دارای تاریخ انقضا)")
        self.rb_subscription.setChecked(True)
        self.rb_subscription.toggled.connect(self._on_license_type_toggled)
        type_row.addWidget(self.rb_subscription)

        self.rb_perpetual = QRadioButton("دائمی و نامحدود (بدون انقضا)")
        type_row.addWidget(self.rb_perpetual)
        type_row.addStretch()
        layout.addLayout(type_row)

        # Quick Duration Chips
        chips_row = QHBoxLayout()
        chips_row.addWidget(QLabel("دوره‌های آماده:"))

        durations = [
            ("۷ روز", 7),
            ("۱ ماه", 30),
            ("۳ ماه", 90),
            ("۶ ماه", 180),
            ("۱ سال", 365),
            ("۲ سال", 730),
        ]
        for label, days in durations:
            btn = QPushButton(label)
            btn.setObjectName("BtnSecondary")
            btn.setFixedWidth(62)
            btn.clicked.connect(lambda _, d=days: self._apply_duration_days(d))
            chips_row.addWidget(btn)

        btn_perp = QPushButton("دائمی")
        btn_perp.setObjectName("BtnSecondary")
        btn_perp.setFixedWidth(62)
        btn_perp.clicked.connect(self._set_perpetual)
        chips_row.addWidget(btn_perp)
        chips_row.addStretch()
        layout.addLayout(chips_row)

        # Jalali Date Pickers
        dates_row = QHBoxLayout()
        self.picker_start = JalaliDateWidget(label_text="تاریخ شروع اعتبار (not_before):")
        dates_row.addWidget(self.picker_start)

        self.picker_expire = JalaliDateWidget(label_text="تاریخ پایان اعتبار (expires_at):")
        dates_row.addWidget(self.picker_expire)

        layout.addLayout(dates_row)
        return box

    def _build_features_card(self) -> QGroupBox:
        box = QGroupBox("امکانات، محدودیت‌ها و نسخه برنامه")
        layout = QVBoxLayout(box)
        layout.setSpacing(10)

        # Version bounds
        ver_row = QHBoxLayout()
        ver_row.addWidget(QLabel("حداقل نسخه مجاز برنامه:"))
        self.ed_min_version = QLineEdit("4.9.0")
        self.ed_min_version.setFixedWidth(90)
        ver_row.addWidget(self.ed_min_version)

        ver_row.addWidget(QLabel("حداکثر نسخه مجاز:"))
        self.ed_max_version = QLineEdit()
        self.ed_max_version.setPlaceholderText("خالی = نامحدود")
        self.ed_max_version.setFixedWidth(110)
        ver_row.addWidget(self.ed_max_version)
        ver_row.addStretch()

        # Select all / Deselect all buttons
        btn_sel_all = QPushButton("✅ انتخاب همه ماژول‌ها")
        btn_sel_all.setObjectName("BtnSecondary")
        btn_sel_all.clicked.connect(lambda: self._set_all_features(True))
        ver_row.addWidget(btn_sel_all)

        btn_desel_all = QPushButton("❌ لغو همه")
        btn_desel_all.setObjectName("BtnSecondary")
        btn_desel_all.clicked.connect(lambda: self._set_all_features(False))
        ver_row.addWidget(btn_desel_all)

        layout.addLayout(ver_row)

        # 10 Module checkboxes arranged in a 2x5 grid
        grid = QGridLayout()
        grid.setSpacing(10)
        for i, key in enumerate(ALL_FEATURE_KEYS):
            row = i // 2
            col = i % 2
            title = FEATURE_TRANSLATIONS.get(key, key)
            cb = QCheckBox(title)
            cb.setChecked(True)
            self._feature_checkboxes[key] = cb
            grid.addWidget(cb, row, col)

        layout.addLayout(grid)
        return box

    def _build_footer_card(self) -> QFrame:
        footer = QFrame()
        footer.setObjectName("MetricCard")
        layout = QVBoxLayout(footer)
        layout.setSpacing(10)

        # Action Buttons
        btn_row = QHBoxLayout()

        self.btn_generate = QPushButton("✨ تولید و ذخیره فایل لایسنس تجاری (.daqlic)")
        self.btn_generate.setStyleSheet(
            "background-color: #0f766e; color: white; font-size: 14px; padding: 10px 24px; font-weight: bold; border-radius: 8px;"
        )
        self.btn_generate.clicked.connect(self._on_generate_license)
        btn_row.addWidget(self.btn_generate, 2)

        self.btn_copy_token = QPushButton("📋 کپی کد متنی فعال‌سازی")
        self.btn_copy_token.setObjectName("BtnSecondary")
        self.btn_copy_token.clicked.connect(self._on_copy_compact_token)
        btn_row.addWidget(self.btn_copy_token, 1)

        layout.addLayout(btn_row)
        return footer

    def _auto_generate_license_id(self) -> None:
        self.ed_license_id.setText(LicenseBuilder.generate_license_id())

    def _on_preset_changed(self) -> None:
        preset_key = self.cb_preset.currentData()
        if preset_key:
            self._apply_preset(preset_key)

    def _apply_preset(self, preset_key: str) -> None:
        p = PLAN_PRESETS.get(preset_key)
        if not p:
            return

        is_sub = (p["license_type"] == "subscription")
        self.rb_subscription.setChecked(is_sub)
        self.rb_perpetual.setChecked(not is_sub)

        if is_sub and p["duration_days"] > 0:
            self._apply_duration_days(p["duration_days"])

        self.sb_max_devices.setValue(p["max_devices"])
        self.ed_min_version.setText(p.get("min_app_version") or "4.9.0")
        self.ed_max_version.setText(p.get("max_app_version") or "")

        features_dict = p.get("features", {})
        for k, cb in self._feature_checkboxes.items():
            cb.setChecked(features_dict.get(k, True))

    def _on_license_type_toggled(self) -> None:
        is_sub = self.rb_subscription.isChecked()
        self.picker_expire.setEnabled(is_sub)

    def _apply_duration_days(self, days: int) -> None:
        self.rb_subscription.setChecked(True)
        # Set start to today, expire to today + days
        self.picker_start.set_to_today()
        self.picker_expire.set_to_today()
        self.picker_expire.add_days(days)

    def _set_perpetual(self) -> None:
        self.rb_perpetual.setChecked(True)

    def _set_all_features(self, state: bool) -> None:
        for cb in self._feature_checkboxes.values():
            cb.setChecked(state)

    def _on_add_single_hwid(self) -> None:
        raw_hwid = self.ed_new_hwid.text().strip()
        raw_label = self.ed_new_label.text().strip()

        if not raw_hwid:
            QMessageBox.warning(self, "خطا", "لطفاً شناسه سخت‌افزاری را وارد کنید.")
            return

        entry = HWIDParser.parse_single(raw_hwid, raw_label)
        if not entry.is_valid:
            QMessageBox.warning(self, "خطای شناسه", entry.validation_error)
            return

        # Check for duplicates
        if any(d.fingerprint == entry.fingerprint for d in self.devices):
            QMessageBox.warning(self, "شناسه تکراری", "این شناسه سخت‌افزاری قبلاً به لیست اضافه شده است.")
            return

        if not entry.label:
            entry.label = f"دستگاه {len(self.devices) + 1}"

        self.devices.append(entry)
        self._refresh_devices_table()
        self.ed_new_hwid.clear()
        self.ed_new_label.clear()

    def _on_open_bulk_dialog(self) -> None:
        dlg = BulkPasteDialog(self)
        if dlg.exec() == QDialog.Accepted:
            new_entries = dlg.get_valid_devices()
            added_count = 0
            for entry in new_entries:
                if not any(d.fingerprint == entry.fingerprint for d in self.devices):
                    self.devices.append(entry)
                    added_count += 1
            self._refresh_devices_table()
            QMessageBox.information(
                self,
                "تکمیل افزودن گروهی",
                f"تعداد {added_count} شناسه سخت‌افزاری جدید با موفقیت اضافه شد.",
            )

    def _on_import_daqreq(self) -> None:
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "انتخاب فایل درخواست لایسنس دقیق",
            "",
            "فایل‌های درخواست دقیق (*.daqreq);;تمام فایل‌ها (*.*)",
        )
        if not file_path:
            return

        try:
            entry = HWIDParser.parse_daqreq_file(file_path)
            if any(d.fingerprint == entry.fingerprint for d in self.devices):
                QMessageBox.warning(self, "شناسه تکراری", "این شناسه قبلاً در لیست ثبت شده است.")
                return
            self.devices.append(entry)
            self._refresh_devices_table()
            QMessageBox.information(
                self,
                "ثبت موفق",
                f"مشخصات سیستم با شناسه:\n{entry.fingerprint}\nبا موفقیت بارگذاری شد.",
            )
        except Exception as e:
            QMessageBox.critical(self, "خطا در خواندن فایل", str(e))

    def _refresh_devices_table(self) -> None:
        self.tbl_devices.setRowCount(len(self.devices))
        for row, entry in enumerate(self.devices):
            # Row number
            it_num = QTableWidgetItem(str(row + 1))
            it_num.setTextAlignment(Qt.AlignCenter)
            self.tbl_devices.setItem(row, 0, it_num)

            # Label (editable)
            it_label = QTableWidgetItem(entry.label)
            self.tbl_devices.setItem(row, 1, it_label)

            # Fingerprint
            it_fp = QTableWidgetItem(entry.fingerprint)
            it_fp.setTextAlignment(Qt.AlignLeft | Qt.AlignVCenter)
            it_fp.setFont(QFont("Consolas", 10))
            self.tbl_devices.setItem(row, 2, it_fp)

            # Status badge
            badge = "معتبر (fp_v1)" if entry.is_valid else "نامعتبر"
            it_badge = QTableWidgetItem(badge)
            it_badge.setTextAlignment(Qt.AlignCenter)
            it_badge.setForeground(QColor("#0f766e" if entry.is_valid else "#be123c"))
            self.tbl_devices.setItem(row, 3, it_badge)

            # Delete button
            btn_del = QPushButton("حذف")
            btn_del.setObjectName("BtnDanger")
            btn_del.setFixedWidth(50)
            btn_del.clicked.connect(lambda _, r=row: self._remove_device(r))
            self.tbl_devices.setCellWidget(row, 4, btn_del)

        self.lbl_devices_count.setText(f"تعداد فعلی: {len(self.devices)} دستگاه")
        if self.sb_max_devices.value() < len(self.devices):
            self.sb_max_devices.setValue(len(self.devices))

    def _remove_device(self, row: int) -> None:
        if 0 <= row < len(self.devices):
            self.devices.pop(row)
            self._refresh_devices_table()

    def _build_license_spec(self) -> LicenseSpec:
        lic_id = self.ed_license_id.text().strip()
        cust_name = self.ed_customer_name.text().strip()
        cust_org = self.ed_customer_org.text().strip() or None
        cust_phone = self.ed_customer_phone.text().strip() or None
        notes = self.ed_notes.text().strip()

        is_sub = self.rb_subscription.isChecked()
        lic_type = "subscription" if is_sub else "perpetual"

        issued_at = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        not_before = self.picker_start.get_utc_iso_string()
        expires_at = self.picker_expire.get_utc_iso_string() if is_sub else None

        fps = [d.fingerprint for d in self.devices if d.is_valid]
        max_dev = max(self.sb_max_devices.value(), len(fps))

        features = {k: cb.isChecked() for k, cb in self._feature_checkboxes.items()}
        min_v = self.ed_min_version.text().strip() or None
        max_v = self.ed_max_version.text().strip() or None

        preset_key = self.cb_preset.currentData() or "Custom"
        edition = PLAN_PRESETS.get(preset_key, {}).get("label_fa", "سفارشی")

        return LicenseSpec(
            license_id=lic_id,
            customer_name=cust_name,
            customer_organization=cust_org,
            customer_contact=cust_phone,
            license_type=lic_type,
            issued_at=issued_at,
            not_before=not_before,
            expires_at=expires_at,
            max_devices=max_dev,
            device_fingerprints=fps,
            features=features,
            min_app_version=min_v,
            max_app_version=max_v,
            edition_label=edition,
            notes=notes,
        )

    def _on_generate_license(self) -> None:
        vault = self.get_active_vault()
        if not vault or not vault.is_unlocked:
            QMessageBox.critical(
                self,
                "گاوصندوق قفل است",
                "جهت امضای لایسنس تجاری، ابتدا گاوصندوق کلیدهای امضا را در تب «مدیریت کلیدها» باز کنید.",
            )
            return

        try:
            spec = self._build_license_spec()
            spec.validate()
        except LicenseValidationError as e:
            QMessageBox.warning(self, "خطا در اطلاعات لایسنس", str(e))
            return

        default_filename = LicenseBuilder.generate_default_filename(spec)
        save_path, _ = QFileDialog.getSaveFileName(
            self,
            "ذخیره فایل لایسنس تجاری (.daqlic)",
            default_filename,
            "فایل‌های لایسنس دقیق (*.daqlic);;تمام فایل‌ها (*.*)",
        )
        if not save_path:
            return

        try:
            gen = LicenseBuilder.build_and_sign(spec, vault)
            gen.save_to_file(save_path)

            # Record in sales registry
            labels_map = {d.fingerprint: d.label for d in self.devices}
            rec = LicenseRecord(
                license_id=spec.license_id,
                customer_name=spec.customer_name,
                customer_organization=spec.customer_organization,
                customer_phone=spec.customer_contact,
                edition=spec.edition_label,
                license_type=spec.license_type,
                issued_at_utc=spec.issued_at,
                not_before_utc=spec.not_before,
                expires_at_utc=spec.expires_at,
                max_devices=spec.max_devices,
                device_count=len(spec.device_fingerprints),
                device_fingerprints=spec.device_fingerprints,
                device_labels=labels_map,
                features=spec.features,
                key_id=vault.key_id,
                key_fingerprint=vault.fingerprint,
                signed_daqlic_content=gen.json_text,
                notes=spec.notes,
                status="active",
            )
            self.registry_db.record_license(rec)

            self._show_success_dialog(save_path, gen)
            self.licenseGenerated.emit(spec.license_id)
            self._auto_generate_license_id()

        except Exception as e:
            QMessageBox.critical(self, "خطا در تولید لایسنس", f"عملیات متوقف شد: {e}")

    def _show_success_dialog(self, file_path: str, gen: GeneratedLicense) -> None:
        dlg = QDialog(self)
        dlg.setWindowTitle("لایسنس با موفقیت تولید شد")
        dlg.setLayoutDirection(Qt.RightToLeft)
        dlg.resize(520, 280)

        layout = QVBoxLayout(dlg)
        layout.setSpacing(12)

        lbl = QLabel(
            f"✅ فایل لایسنس تجاری دقیق با موفقیت امضا، اعتبارسنجی و ذخیره شد:\n\n"
            f"مسیر ذخیره:\n{file_path}\n\n"
            f"شناسه لایسنس: {gen.license_id}\n"
            f"مشتری: {gen.customer_name}"
        )
        lbl.setWordWrap(True)
        lbl.setStyleSheet("font-size: 12px; line-height: 1.6;")
        layout.addWidget(lbl)

        btn_row = QHBoxLayout()
        btn_open_folder = QPushButton("📂 باز کردن پوشه فایل")
        btn_open_folder.clicked.connect(lambda: os.startfile(os.path.dirname(file_path)))
        btn_row.addWidget(btn_open_folder)

        btn_copy_summary = QPushButton("📋 کپی خلاصه جهت ارسال به مشتری")
        btn_copy_summary.setObjectName("BtnSecondary")
        summary_text = (
            f"سلام، فایل لایسنس نسخه تجاری دقیق برای شما صادر شد:\n"
            f"نام مشتری: {gen.customer_name}\n"
            f"شماره لایسنس: {gen.license_id}\n"
            f"نوع لایسنس: {'دائمی' if gen.spec.license_type == 'perpetual' else 'اشتراکی'}\n"
            f"تعداد دستگاه مجاز: {gen.spec.max_devices}\n"
            f"لطفاً فایل .daqlic پیوست‌شده را در نرم‌افزار دقیق از بخش «مدیریت لایسنس» وارد فرمایید."
        )
        btn_copy_summary.clicked.connect(lambda: pyperclip.copy(summary_text))
        btn_row.addWidget(btn_copy_summary)

        btn_ok = QPushButton("تأیید")
        btn_ok.setObjectName("BtnSuccess")
        btn_ok.clicked.connect(dlg.accept)
        btn_row.addWidget(btn_ok)

        layout.addLayout(btn_row)
        dlg.exec()

    def _on_copy_compact_token(self) -> None:
        vault = self.get_active_vault()
        if not vault or not vault.is_unlocked:
            QMessageBox.warning(self, "گاوصندوق قفل است", "ابتدا گاوصندوق را باز کنید.")
            return

        try:
            spec = self._build_license_spec()
            spec.validate()
            gen = LicenseBuilder.build_and_sign(spec, vault)
            pyperclip.copy(gen.compact_token)
            QMessageBox.information(
                self,
                "کپی شد",
                "کد متنی لایسنس در حافظه کلیپ‌بورد کپی شد.",
            )
        except Exception as e:
            QMessageBox.warning(self, "خطا", str(e))

    def load_spec_for_clone_or_renew(self, record: LicenseRecord) -> None:
        """Populate the generator inputs with an existing license record for fast renewal."""
        self.ed_customer_name.setText(record.customer_name)
        self.ed_customer_org.setText(record.customer_organization or "")
        self.ed_customer_phone.setText(record.customer_phone or "")
        self.ed_notes.setText(f"تمدید شده از لایسنس {record.license_id}")

        self.devices = []
        for fp in record.device_fingerprints:
            lbl = record.device_labels.get(fp, "دستگاه")
            self.devices.append(DeviceEntry(fingerprint=fp, label=lbl, is_valid=True))
        self._refresh_devices_table()
        self.sb_max_devices.setValue(max(record.max_devices, len(self.devices)))

        # Update features
        for k, cb in self._feature_checkboxes.items():
            cb.setChecked(record.features.get(k, True))

        # Extend expiry by 1 year from now
        self._auto_generate_license_id()
        self._apply_duration_days(365)
