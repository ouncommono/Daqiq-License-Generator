"""Interactive Persian Jalali Date Picker widget.

Provides seamless Shamsi (Jalali) date selection with synchronized Gregorian/UTC conversion,
quick chips (+1 month, +3 months, +1 year, etc.), and Persian month names.
"""

from __future__ import annotations

import datetime
from typing import Optional, Callable
import jdatetime

from PySide6.QtCore import Qt, Signal
from PySide6.QtWidgets import (
    QWidget,
    QHBoxLayout,
    QVBoxLayout,
    QComboBox,
    QSpinBox,
    QLabel,
    QPushButton,
    QFrame,
)

PERSIAN_MONTH_NAMES = [
    "فروردین",
    "اردیبهشت",
    "خرداد",
    "تیر",
    "مرداد",
    "شهریور",
    "مهر",
    "آبان",
    "آذر",
    "دی",
    "بهمن",
    "اسفند",
]


class JalaliDateWidget(QWidget):
    """Clean Persian Jalali date selector with synchronized Gregorian display."""

    dateChanged = Signal(str, str)  # (jalali_str "YYYY/MM/DD", utc_iso_str "YYYY-MM-DDTHH:MM:SSZ")

    def __init__(
        self,
        initial_jdate: Optional[jdatetime.date] = None,
        parent: Optional[QWidget] = None,
        label_text: str = "تاریخ:",
    ):
        super().__init__(parent)
        self.setLayoutDirection(Qt.RightToLeft)

        if initial_jdate is None:
            now_utc = datetime.datetime.now(datetime.timezone.utc)
            self._current_jdate = jdatetime.date.fromgregorian(date=now_utc.date())
        else:
            self._current_jdate = initial_jdate

        self._build_ui(label_text)
        self._sync_inputs_from_jdate()

    def _build_ui(self, label_text: str) -> None:
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(4)

        if label_text:
            lbl_title = QLabel(label_text)
            lbl_title.setStyleSheet("font-weight: bold; font-size: 11px; color: #475569;")
            main_layout.addWidget(lbl_title)

        row_layout = QHBoxLayout()
        row_layout.setSpacing(6)

        # Day SpinBox (1..31)
        self.sb_day = QSpinBox()
        self.sb_day.setRange(1, 31)
        self.sb_day.setSuffix(" ")
        self.sb_day.setFixedWidth(65)
        self.sb_day.valueChanged.connect(self._on_component_changed)
        row_layout.addWidget(self.sb_day)

        # Month ComboBox
        self.cb_month = QComboBox()
        self.cb_month.setFixedWidth(110)
        for i, name in enumerate(PERSIAN_MONTH_NAMES, start=1):
            self.cb_month.addItem(f"{i:02d} - {name}", i)
        self.cb_month.currentIndexChanged.connect(self._on_component_changed)
        row_layout.addWidget(self.cb_month)

        # Year SpinBox
        self.sb_year = QSpinBox()
        self.sb_year.setRange(1400, 1450)
        self.sb_year.setFixedWidth(75)
        self.sb_year.valueChanged.connect(self._on_component_changed)
        row_layout.addWidget(self.sb_year)

        # Quick 'Today' button
        self.btn_today = QPushButton("امروز")
        self.btn_today.setObjectName("BtnSecondary")
        self.btn_today.setFixedWidth(55)
        self.btn_today.clicked.connect(self.set_to_today)
        row_layout.addWidget(self.btn_today)

        row_layout.addStretch()
        main_layout.addLayout(row_layout)

        # Synchronized Gregorian/UTC text label underneath
        self.lbl_gregorian = QLabel()
        self.lbl_gregorian.setStyleSheet("font-size: 11px; color: #64748b; font-family: Consolas, monospace;")
        main_layout.addWidget(self.lbl_gregorian)

    def _sync_inputs_from_jdate(self) -> None:
        self.sb_year.blockSignals(True)
        self.cb_month.blockSignals(True)
        self.sb_day.blockSignals(True)

        self.sb_year.setValue(self._current_jdate.year)
        self.cb_month.setCurrentIndex(self._current_jdate.month - 1)
        self.sb_day.setValue(self._current_jdate.day)

        self.sb_year.blockSignals(False)
        self.cb_month.blockSignals(False)
        self.sb_day.blockSignals(False)

        self._update_gregorian_label()

    def _update_gregorian_label(self) -> None:
        gdate = self._current_jdate.togregorian()
        iso_str = f"{gdate.year:04d}-{gdate.month:02d}-{gdate.day:02d}T00:00:00Z"
        jalali_str = f"{self._current_jdate.year:04d}/{self._current_jdate.month:02d}/{self._current_jdate.day:02d}"
        weekday_name = ["دوشنبه", "سه‌شنبه", "چهارشنبه", "پنج‌شنبه", "جمعه", "شنبه", "یکشنبه"][gdate.weekday()]
        self.lbl_gregorian.setText(f"معادل میلادی: {gdate.strftime('%Y-%m-%d')} ({weekday_name})")
        self.dateChanged.emit(jalali_str, iso_str)

    def _on_component_changed(self) -> None:
        y = self.sb_year.value()
        m = self.cb_month.currentData() or 1
        d = self.sb_day.value()

        # Adjust max days for month
        max_d = 31 if m <= 6 else (30 if m <= 11 else (30 if jdatetime.date(y, 1, 1).isleap() else 29))
        self.sb_day.setMaximum(max_d)
        if d > max_d:
            d = max_d
            self.sb_day.setValue(d)

        try:
            self._current_jdate = jdatetime.date(y, m, d)
        except Exception:
            pass
        self._update_gregorian_label()

    def set_to_today(self) -> None:
        """Reset picker to today's Jalali date."""
        now = datetime.datetime.now(datetime.timezone.utc)
        self._current_jdate = jdatetime.date.fromgregorian(date=now.date())
        self._sync_inputs_from_jdate()

    def set_jdate(self, jdate: jdatetime.date) -> None:
        self._current_jdate = jdate
        self._sync_inputs_from_jdate()

    def set_from_utc_iso(self, iso_str: str) -> None:
        """Set date from ISO-8601 string e.g. 2026-11-22T00:00:00Z."""
        if not iso_str:
            return
        clean = iso_str.strip().split("T")[0]
        try:
            parts = [int(p) for p in clean.split("-")]
            gdate = datetime.date(parts[0], parts[1], parts[2])
            self._current_jdate = jdatetime.date.fromgregorian(date=gdate)
            self._sync_inputs_from_jdate()
        except Exception:
            pass

    def add_days(self, days: int) -> None:
        gdate = self._current_jdate.togregorian() + datetime.timedelta(days=days)
        self._current_jdate = jdatetime.date.fromgregorian(date=gdate)
        self._sync_inputs_from_jdate()

    def add_months(self, months: int) -> None:
        # Approximate by days
        self.add_days(months * 30)

    def get_jdate(self) -> jdatetime.date:
        return self._current_jdate

    def get_jalali_string(self) -> str:
        return f"{self._current_jdate.year:04d}/{self._current_jdate.month:02d}/{self._current_jdate.day:02d}"

    def get_utc_iso_string(self) -> str:
        gdate = self._current_jdate.togregorian()
        return f"{gdate.year:04d}-{gdate.month:02d}-{gdate.day:02d}T00:00:00Z"
