"""Tab 2: Issued Licenses Ledger (سوابق لایسنس‌های صادرشده).

Provides search, status filters, summary metrics, fast Clone & Renew,
re-saving .daqlic files, and CSV/Excel exports.
"""

from __future__ import annotations

import datetime
import json
import os
from typing import Optional, List, Dict, Any

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
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
    QFileDialog,
    QMessageBox,
    QDialog,
    QFrame,
    QTextEdit,
)

import jdatetime
import pyperclip

from core.registry_db import RegistryDB, LicenseRecord


def format_iso_to_jalali(iso_str: Optional[str]) -> str:
    """Format UTC ISO date to Shamsi string 'YYYY/MM/DD'."""
    if not iso_str:
        return "نامحدود (دائمی)"
    try:
        clean = iso_str.strip().split("T")[0]
        parts = [int(p) for p in clean.split("-")]
        gdate = datetime.date(parts[0], parts[1], parts[2])
        jdate = jdatetime.date.fromgregorian(date=gdate)
        return f"{jdate.year:04d}/{jdate.month:02d}/{jdate.day:02d}"
    except Exception:
        return iso_str


class LicenseDetailsDialog(QDialog):
    """Detailed view modal for an issued license."""

    def __init__(self, record: LicenseRecord, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setWindowTitle(f"جزئیات لایسنس {record.license_id}")
        self.setLayoutDirection(Qt.RightToLeft)
        self.resize(600, 480)

        layout = QVBoxLayout(self)
        layout.setSpacing(12)

        info_box = QGroupBox("مشخصات لایسنس")
        g_layout = QGridLayout(info_box)

        g_layout.addWidget(QLabel("شماره لایسنس:"), 0, 0)
        g_layout.addWidget(QLabel(record.license_id), 0, 1)

        g_layout.addWidget(QLabel("نام مشتری:"), 0, 2)
        g_layout.addWidget(QLabel(record.customer_name), 0, 3)

        g_layout.addWidget(QLabel("پلن و نوع:"), 1, 0)
        g_layout.addWidget(QLabel(f"{record.edition} ({'دائمی' if record.license_type == 'perpetual' else 'اشتراکی'})"), 1, 1)

        g_layout.addWidget(QLabel("شماره تماس:"), 1, 2)
        g_layout.addWidget(QLabel(record.customer_phone or "ثبت نشده"), 1, 3)

        g_layout.addWidget(QLabel("تاریخ شروع:"), 2, 0)
        g_layout.addWidget(QLabel(format_iso_to_jalali(record.not_before_utc)), 2, 1)

        g_layout.addWidget(QLabel("تاریخ انقضا:"), 2, 2)
        g_layout.addWidget(QLabel(format_iso_to_jalali(record.expires_at_utc)), 2, 3)

        g_layout.addWidget(QLabel("شناسه کلید امضا:"), 3, 0)
        g_layout.addWidget(QLabel(record.key_id), 3, 1)

        g_layout.addWidget(QLabel("تعداد دستگاه مجاز:"), 3, 2)
        g_layout.addWidget(QLabel(f"{record.device_count} از {record.max_devices}"), 3, 3)

        layout.addWidget(info_box)

        # JSON Viewer
        layout.addWidget(QLabel("محتوای فایل .daqlic:"))
        self.txt_json = QTextEdit()
        self.txt_json.setReadOnly(True)
        self.txt_json.setFont(QFont("Consolas", 10))
        self.txt_json.setPlainText(record.signed_daqlic_content)
        layout.addWidget(self.txt_json, 1)

        btn_row = QHBoxLayout()
        btn_copy = QPushButton("📋 کپی متن JSON")
        btn_copy.clicked.connect(lambda: pyperclip.copy(record.signed_daqlic_content))
        btn_row.addWidget(btn_copy)

        btn_close = QPushButton("بستن")
        btn_close.setObjectName("BtnSecondary")
        btn_close.clicked.connect(self.accept)
        btn_row.addWidget(btn_close)

        layout.addLayout(btn_row)


class TabLedger(QWidget):
    """Tab 2: Issued Licenses Ledger."""

    cloneRequested = Signal(object)  # Emits LicenseRecord for Tab 1 Clone & Renew

    def __init__(self, registry_db: RegistryDB, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.registry_db = registry_db
        self.current_records: List[LicenseRecord] = []
        self.setLayoutDirection(Qt.RightToLeft)
        self._build_ui()
        self.refresh_records()

    def _build_ui(self) -> None:
        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(12)

        # 1. Summary Metric Cards
        layout.addLayout(self._build_metrics_row())

        # 2. Search & Filter Bar
        layout.addLayout(self._build_search_filter_bar())

        # 3. Main Records Table
        self.table = QTableWidget(0, 8)
        self.table.setHorizontalHeaderLabels([
            "شماره لایسنس",
            "مشتری / فروشگاه",
            "پلن",
            "دستگاه‌ها",
            "تاریخ صدور",
            "تاریخ انقضا",
            "وضعیت",
            "عملیات",
        ])
        self.table.horizontalHeader().setSectionResizeMode(0, QHeaderView.Interactive)
        self.table.horizontalHeader().setSectionResizeMode(1, QHeaderView.Stretch)
        self.table.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(4, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(5, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(6, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(7, QHeaderView.ResizeToContents)
        layout.addWidget(self.table, 1)

    def _build_metrics_row(self) -> QHBoxLayout:
        row = QHBoxLayout()
        row.setSpacing(10)

        self.card_total = self._create_metric_card("کل لایسنس‌ها", "۰", "#0284c7")
        self.card_active = self._create_metric_card("فعال", "۰", "#0f766e")
        self.card_expiring = self._create_metric_card("نزدیک به انقضا", "۰", "#d97706")
        self.card_expired = self._create_metric_card("منقضی‌شده", "۰", "#be123c")
        self.card_perpetual = self._create_metric_card("دائمی", "۰", "#6366f1")

        row.addWidget(self.card_total)
        row.addWidget(self.card_active)
        row.addWidget(self.card_expiring)
        row.addWidget(self.card_expired)
        row.addWidget(self.card_perpetual)
        return row

    def _create_metric_card(self, title: str, val: str, color_hex: str) -> QFrame:
        card = QFrame()
        card.setObjectName("MetricCard")
        v = QVBoxLayout(card)
        v.setContentsMargins(10, 8, 10, 8)
        v.setSpacing(4)

        lbl_t = QLabel(title)
        lbl_t.setStyleSheet("font-size: 11px; color: #64748b; font-weight: bold;")
        v.addWidget(lbl_t)

        lbl_v = QLabel(val)
        lbl_v.setObjectName(f"val_{title}")
        lbl_v.setStyleSheet(f"font-size: 18px; font-weight: bold; color: {color_hex};")
        v.addWidget(lbl_v)
        return card

    def _build_search_filter_bar(self) -> QHBoxLayout:
        bar = QHBoxLayout()
        bar.setSpacing(8)

        self.ed_search = QLineEdit()
        self.ed_search.setPlaceholderText("🔍 جستجو بر اساس نام مشتری، شماره لایسنس، تلفن، یا شناسه سخت‌افزاری...")
        self.ed_search.textChanged.connect(self.refresh_records)
        bar.addWidget(self.ed_search, 2)

        # Filter Chip dropdown
        bar.addWidget(QLabel("فیلتر وضعیت:"))
        self.cb_filter = QComboBox()
        self.cb_filter.addItem("همه سوابق", "all")
        self.cb_filter.addItem("فعال", "active")
        self.cb_filter.addItem("نزدیک به انقضا (کمتر از ۱۴ روز)", "expiring_soon")
        self.cb_filter.addItem("منقضی‌شده", "expired")
        self.cb_filter.addItem("دائمی", "perpetual")
        self.cb_filter.currentIndexChanged.connect(self.refresh_records)
        bar.addWidget(self.cb_filter, 1)

        # Export CSV Button
        self.btn_export = QPushButton("📊 خروجی اکسل / CSV")
        self.btn_export.setObjectName("BtnSecondary")
        self.btn_export.clicked.connect(self._on_export_csv)
        bar.addWidget(self.btn_export)

        return bar

    def refresh_records(self) -> None:
        """Fetch records from DB and populate table and metric cards."""
        query = self.ed_search.text().strip()
        status_filter = self.cb_filter.currentData() or "all"

        self.current_records = self.registry_db.search_licenses(query, status_filter)
        self._update_metrics()
        self._populate_table()

    def _update_metrics(self) -> None:
        stats = self.registry_db.get_stats()
        self.card_total.findChild(QLabel, "val_کل لایسنس‌ها").setText(str(stats["total"]))
        self.card_active.findChild(QLabel, "val_فعال").setText(str(stats["active"]))
        self.card_expiring.findChild(QLabel, "val_نزدیک به انقضا").setText(str(stats["expiring_soon"]))
        self.card_expired.findChild(QLabel, "val_منقضی‌شده").setText(str(stats["expired"]))
        self.card_perpetual.findChild(QLabel, "val_دائمی").setText(str(stats["perpetual"]))

    def _populate_table(self) -> None:
        self.table.setRowCount(len(self.current_records))
        for row, rec in enumerate(self.current_records):
            # License ID
            it_id = QTableWidgetItem(rec.license_id)
            it_id.setFont(QFont("Consolas", 10))
            self.table.setItem(row, 0, it_id)

            # Customer Name
            cust_text = rec.customer_name
            if rec.customer_organization:
                cust_text += f" ({rec.customer_organization})"
            self.table.setItem(row, 1, QTableWidgetItem(cust_text))

            # Plan
            self.table.setItem(row, 2, QTableWidgetItem(rec.edition))

            # Devices
            dev_text = f"{rec.device_count} / {rec.max_devices}"
            it_dev = QTableWidgetItem(dev_text)
            it_dev.setTextAlignment(Qt.AlignCenter)
            self.table.setItem(row, 3, it_dev)

            # Issued at
            it_iss = QTableWidgetItem(format_iso_to_jalali(rec.issued_at_utc))
            it_iss.setTextAlignment(Qt.AlignCenter)
            self.table.setItem(row, 4, it_iss)

            # Expires at
            it_exp = QTableWidgetItem(format_iso_to_jalali(rec.expires_at_utc))
            it_exp.setTextAlignment(Qt.AlignCenter)
            self.table.setItem(row, 5, it_exp)

            # Status Badge
            eff_status = rec.effective_status
            if eff_status == "active":
                badge_text, color = "فعال", "#0f766e"
            elif eff_status == "expiring_soon":
                badge_text, color = "نزدیک انقضا", "#d97706"
            elif eff_status == "expired":
                badge_text, color = "منقضی‌شده", "#be123c"
            elif eff_status == "renewed":
                badge_text, color = "تمدیدشده", "#64748b"
            else:
                badge_text, color = eff_status, "#475569"

            it_badge = QTableWidgetItem(badge_text)
            it_badge.setTextAlignment(Qt.AlignCenter)
            it_badge.setForeground(QColor(color))
            self.table.setItem(row, 6, it_badge)

            # Actions Cell
            act_widget = QWidget()
            act_layout = QHBoxLayout(act_widget)
            act_layout.setContentsMargins(4, 2, 4, 2)
            act_layout.setSpacing(4)

            btn_renew = QPushButton("تمدید")
            btn_renew.setToolTip("تمدید سریع مشخصات این مشتری در استودیوی صدور لایسنس")
            btn_renew.clicked.connect(lambda _, r=rec: self.cloneRequested.emit(r))
            act_layout.addWidget(btn_renew)

            btn_resave = QPushButton("ذخیره")
            btn_resave.setObjectName("BtnSecondary")
            btn_resave.setToolTip("ذخیره مجدد فایل .daqlic بر روی دیسک")
            btn_resave.clicked.connect(lambda _, r=rec: self._resave_daqlic(r))
            act_layout.addWidget(btn_resave)

            btn_view = QPushButton("نمایش")
            btn_view.setObjectName("BtnSecondary")
            btn_view.clicked.connect(lambda _, r=rec: self._view_details(r))
            act_layout.addWidget(btn_view)

            btn_del = QPushButton("حذف")
            btn_del.setObjectName("BtnDanger")
            btn_del.clicked.connect(lambda _, r=rec: self._delete_record(r))
            act_layout.addWidget(btn_del)

            self.table.setCellWidget(row, 7, act_widget)

    def _resave_daqlic(self, record: LicenseRecord) -> None:
        clean_cust = "".join(c for c in record.customer_name if c.isalnum() or c in (" ", "_", "-")).strip()
        default_name = f"Daqiq_{clean_cust}_{record.license_id}.daqlic"
        save_path, _ = QFileDialog.getSaveFileName(
            self,
            "ذخیره مجدد فایل لایسنس (.daqlic)",
            default_name,
            "فایل‌های لایسنس دقیق (*.daqlic);;تمام فایل‌ها (*.*)",
        )
        if not save_path:
            return

        try:
            with open(save_path, "w", encoding="utf-8") as f:
                f.write(record.signed_daqlic_content)
            QMessageBox.information(self, "موفقیت", f"فایل لایسنس با موفقیت در مسیر زیر ذخیره شد:\n{save_path}")
        except Exception as e:
            QMessageBox.critical(self, "خطا در ذخیره فایل", str(e))

    def _view_details(self, record: LicenseRecord) -> None:
        dlg = LicenseDetailsDialog(record, self)
        dlg.exec()

    def _delete_record(self, record: LicenseRecord) -> None:
        reply = QMessageBox.question(
            self,
            "حذف لایسنس از سوابق",
            f"آیا از حذف لایسنس {record.license_id} متعلق به «{record.customer_name}» از دفتر سوابق اطمینان دارید؟",
            QMessageBox.Yes | QMessageBox.No,
            QMessageBox.No,
        )
        if reply == QMessageBox.Yes:
            self.registry_db.delete_license(record.license_id)
            self.refresh_records()

    def _on_export_csv(self) -> None:
        save_path, _ = QFileDialog.getSaveFileName(
            self,
            "خروجی فایل اکسل / CSV سوابق",
            "Daqiq_Licenses_Ledger.csv",
            "فایل‌های CSV (*.csv);;تمام فایل‌ها (*.*)",
        )
        if not save_path:
            return

        try:
            self.registry_db.export_to_csv(save_path)
            QMessageBox.information(
                self,
                "خروجی موفق",
                f"فایل سوابق با موفقیت در قالب CSV ذخیره شد:\n{save_path}",
            )
        except Exception as e:
            QMessageBox.critical(self, "خطا در تهیه خروجی", str(e))
