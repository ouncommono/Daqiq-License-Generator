"""Main Desktop Window for Daqiq License Generator.

Persian RTL layout, top status header bar, responsive tabbed interface,
and seamless orchestration between key vault, generator, registry ledger, and inspector.
"""

from __future__ import annotations

import os
import sys
from typing import Optional

from PySide6.QtCore import Qt
from PySide6.QtGui import QIcon, QFont, QAction
from PySide6.QtWidgets import (
    QApplication,
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QTabWidget,
    QFrame,
    QMessageBox,
)

from core.key_vault import UnlockedVault
from core.registry_db import RegistryDB
from .theme import get_theme_stylesheet
from .tab_generator import TabGenerator
from .tab_ledger import TabLedger
from .tab_inspector import TabInspector
from .tab_vault import TabVault

APP_VERSION = "1.0.0"


class MainWindow(QMainWindow):
    """Main Application Window."""

    def __init__(
        self,
        registry_db: Optional[RegistryDB] = None,
        initial_vault: Optional[UnlockedVault] = None,
        is_dark_theme: bool = False,
        parent: Optional[QWidget] = None,
    ):
        super().__init__(parent)
        self.registry_db = registry_db or RegistryDB()
        self._active_vault: Optional[UnlockedVault] = initial_vault
        self.is_dark_theme = is_dark_theme

        self.setWindowTitle(f"مولد لایسنس دقیق — نسخه {APP_VERSION}")
        self.setLayoutDirection(Qt.RightToLeft)
        self.setMinimumSize(960, 680)
        self.resize(1080, 750)

        # Set Window Icon
        icon_path = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "icon.png")
        if os.path.isfile(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        self._build_ui()
        self._apply_theme(self.is_dark_theme)
        self._update_header_vault_status()

    def _build_ui(self) -> None:
        central_widget = QWidget(self)
        self.setCentralWidget(central_widget)

        root_layout = QVBoxLayout(central_widget)
        root_layout.setContentsMargins(0, 0, 0, 0)
        root_layout.setSpacing(0)

        # 1. Top Status Header Bar
        root_layout.addWidget(self._build_header_bar())

        # 2. Main Tab Widget
        self.tab_widget = QTabWidget()
        self.tab_widget.setLayoutDirection(Qt.RightToLeft)

        # Instantiate Tabs
        self.tab_generator = TabGenerator(
            get_active_vault_callback=lambda: self._active_vault,
            registry_db=self.registry_db,
            parent=self,
        )
        self.tab_ledger = TabLedger(
            registry_db=self.registry_db,
            parent=self,
        )
        self.tab_inspector = TabInspector(
            get_active_vault_callback=lambda: self._active_vault,
            parent=self,
        )
        self.tab_vault = TabVault(
            get_active_vault_callback=lambda: self._active_vault,
            set_active_vault_callback=self._set_active_vault,
            is_dark_theme=self.is_dark_theme,
            parent=self,
        )

        # Add tabs
        self.tab_widget.addTab(self.tab_generator, "✨ صدور لایسنس (.daqlic)")
        self.tab_widget.addTab(self.tab_ledger, "📋 سوابق لایسنس‌های صادرشده")
        self.tab_widget.addTab(self.tab_inspector, "🔍 بازرس و شبیه‌ساز لایسنس")
        self.tab_widget.addTab(self.tab_vault, "🔒 گاوصندوق کلیدها و تنظیمات")

        root_layout.addWidget(self.tab_widget, 1)

        # Wire cross-tab signals
        self.tab_generator.licenseGenerated.connect(self._on_license_generated)
        self.tab_ledger.cloneRequested.connect(self._on_clone_requested)
        self.tab_vault.vaultChanged.connect(self._on_vault_changed)
        self.tab_vault.themeChanged.connect(self._apply_theme)

    def _build_header_bar(self) -> QFrame:
        header = QFrame()
        header.setObjectName("HeaderBar")
        header.setFixedHeight(62)

        layout = QHBoxLayout(header)
        layout.setContentsMargins(16, 8, 16, 8)
        layout.setSpacing(12)

        # Logo / Title
        v_title = QVBoxLayout()
        v_title.setSpacing(1)
        lbl_title = QLabel("سامانه صدور و مدیریت لایسنس تجاری دقیق (Daqiq)")
        lbl_title.setStyleSheet("color: #ffffff; font-size: 14px; font-weight: bold;")
        lbl_sub = QLabel(f"نسخه {APP_VERSION} — آفلاین و مبتنی بر رمزنگاری نامتقارن Ed25519")
        lbl_sub.setStyleSheet("color: #94a3b8; font-size: 10px;")
        v_title.addWidget(lbl_title)
        v_title.addWidget(lbl_sub)
        layout.addLayout(v_title)

        layout.addStretch()

        # Vault Status Pill in Header
        self.lbl_header_vault_pill = QLabel("🔒 کلید: قفل")
        self.lbl_header_vault_pill.setStyleSheet(
            "background-color: rgba(239, 68, 68, 0.2); color: #fca5a5; "
            "padding: 5px 12px; border-radius: 12px; font-weight: bold; font-size: 11px;"
        )
        layout.addWidget(self.lbl_header_vault_pill)

        self.btn_header_vault_action = QPushButton("گاوصندوق کلید")
        self.btn_header_vault_action.setObjectName("BtnSecondary")
        self.btn_header_vault_action.setFixedWidth(110)
        self.btn_header_vault_action.clicked.connect(self._on_header_vault_click)
        layout.addWidget(self.btn_header_vault_action)

        # Theme Toggle Button
        self.btn_theme_toggle = QPushButton("🌙 تم تیره")
        self.btn_theme_toggle.setObjectName("BtnSecondary")
        self.btn_theme_toggle.setFixedWidth(90)
        self.btn_theme_toggle.clicked.connect(self._toggle_theme)
        layout.addWidget(self.btn_theme_toggle)

        return header

    def _set_active_vault(self, vault: Optional[UnlockedVault]) -> None:
        self._active_vault = vault
        self._update_header_vault_status()
        self.tab_vault.refresh_vault_display()

    def _update_header_vault_status(self) -> None:
        if self._active_vault and self._active_vault.is_unlocked:
            self.lbl_header_vault_pill.setText(
                f"🔓 کلید فعال: {self._active_vault.key_id} [{self._active_vault.short_fingerprint}]"
            )
            self.lbl_header_vault_pill.setStyleSheet(
                "background-color: rgba(16, 185, 129, 0.25); color: #6ee7b7; "
                "padding: 5px 12px; border-radius: 12px; font-weight: bold; font-size: 11px;"
            )
            self.btn_header_vault_action.setText("🔒 قفل کردن")
        else:
            self.lbl_header_vault_pill.setText("🔒 کلید: قفل")
            self.lbl_header_vault_pill.setStyleSheet(
                "background-color: rgba(239, 68, 68, 0.2); color: #fca5a5; "
                "padding: 5px 12px; border-radius: 12px; font-weight: bold; font-size: 11px;"
            )
            self.btn_header_vault_action.setText("🔓 باز کردن کلید")

    def _on_header_vault_click(self) -> None:
        if self._active_vault and self._active_vault.is_unlocked:
            self._active_vault.lock()
            self._set_active_vault(None)
        else:
            # Switch to Tab 4 for unlocking or opening vault
            self.tab_widget.setCurrentIndex(3)

    def _on_license_generated(self, license_id: str) -> None:
        """Triggered when Tab 1 finishes issuing a license."""
        self.tab_ledger.refresh_records()

    def _on_clone_requested(self, record) -> None:
        """Triggered from Tab 2 Clone & Renew button."""
        self.tab_generator.load_spec_for_clone_or_renew(record)
        self.tab_widget.setCurrentIndex(0)

    def _on_vault_changed(self, vault: Optional[UnlockedVault]) -> None:
        self._active_vault = vault
        self._update_header_vault_status()

    def _toggle_theme(self) -> None:
        self._apply_theme(not self.is_dark_theme)

    def _apply_theme(self, is_dark: bool) -> None:
        self.is_dark_theme = is_dark
        self.setStyleSheet(get_theme_stylesheet(is_dark))
        if hasattr(self, "btn_theme_toggle"):
            self.btn_theme_toggle.setText("☀️ تم روشن" if is_dark else "🌙 تم تیره")
