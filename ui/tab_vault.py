"""Tab 4: Key Vault Management and Settings (مدیریت کلیدهای امضا و تنظیمات).

Manages encrypted vault creation, unlocking, locking, passphrase changing,
raw/PEM secret key importing, public key export for Daqiq PR #38, and preferences.
"""

from __future__ import annotations

import os
import shutil
from typing import Optional, Callable

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont
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
    QDialog,
    QScrollArea,
    QFrame,
    QComboBox,
)

import pyperclip

from core.key_vault import (
    KeyVault,
    UnlockedVault,
    VaultError,
    InvalidPassphraseError,
)


class CreateVaultDialog(QDialog):
    """Dialog for generating a new Ed25519 keypair and encrypted vault."""

    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setWindowTitle("ساخت گاوصندوق کلید امضای جدید")
        self.setLayoutDirection(Qt.RightToLeft)
        self.resize(460, 260)
        self.result_vault: Optional[UnlockedVault] = None
        self._build_ui()

    def _build_ui(self) -> None:
        layout = QVBoxLayout(self)
        layout.setSpacing(12)

        g = QGridLayout()
        g.addWidget(QLabel("شناسه کلید (Key ID):"), 0, 0)
        self.ed_key_id = QLineEdit("daqiq-prod-v1")
        g.addWidget(self.ed_key_id, 0, 1)

        g.addWidget(QLabel("رمز عبور گاوصندوق:"), 1, 0)
        self.ed_pass1 = QLineEdit()
        self.ed_pass1.setEchoMode(QLineEdit.Password)
        self.ed_pass1.setPlaceholderText("حداقل ۸ نویسه شامل حروف و اعداد")
        g.addWidget(self.ed_pass1, 1, 1)

        g.addWidget(QLabel("تکرار رمز عبور:"), 2, 0)
        self.ed_pass2 = QLineEdit()
        self.ed_pass2.setEchoMode(QLineEdit.Password)
        g.addWidget(self.ed_pass2, 2, 1)

        layout.addLayout(g)

        hint = QLabel("🔒 کلید خصوصی با الگوریتم AES-256-GCM و ۶۰۰٬۰۰۰ دور PBKDF2 رمزنگاری خواهد شد.")
        hint.setStyleSheet("font-size: 11px; color: #0284c7;")
        layout.addWidget(hint)

        btn_row = QHBoxLayout()
        btn_create = QPushButton("ایجاد و ذخیره گاوصندوق")
        btn_create.setObjectName("BtnSuccess")
        btn_create.clicked.connect(self._on_create)
        btn_row.addWidget(btn_create)

        btn_cancel = QPushButton("انصراف")
        btn_cancel.setObjectName("BtnSecondary")
        btn_cancel.clicked.connect(self.reject)
        btn_row.addWidget(btn_cancel)

        layout.addLayout(btn_row)

    def _on_create(self) -> None:
        key_id = self.ed_key_id.text().strip()
        p1 = self.ed_pass1.text()
        p2 = self.ed_pass2.text()

        if not key_id:
            QMessageBox.warning(self, "خطا", "شناسه کلید را وارد کنید.")
            return
        if not p1 or len(p1) < 6:
            QMessageBox.warning(self, "خطا", "رمز عبور باید حداقل ۶ نویسه باشد.")
            return
        if p1 != p2:
            QMessageBox.warning(self, "خطا", "تکرار رمز عبور همخوانی ندارد.")
            return

        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "ذخیره فایل گاوصندوق کلید امضا",
            f"vault_{key_id}.dqvault",
            "فایل‌های گاوصندوق کلید (*.dqvault *.dqkey);;تمام فایل‌ها (*.*)",
        )
        if not file_path:
            return

        try:
            unlocked, _ = KeyVault.create_vault(
                key_id=key_id,
                passphrase=p1,
                file_path=file_path,
            )
            self.result_vault = unlocked
            self.accept()
        except Exception as e:
            QMessageBox.critical(self, "خطا در ساخت گاوصندوق", str(e))


class ImportKeyDialog(QDialog):
    """Dialog for importing existing Ed25519 secret keys."""

    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setWindowTitle("وارد کردن کلید خصوصی موجود (Raw / PEM)")
        self.setLayoutDirection(Qt.RightToLeft)
        self.resize(500, 320)
        self.result_vault: Optional[UnlockedVault] = None
        self._build_ui()

    def _build_ui(self) -> None:
        layout = QVBoxLayout(self)
        layout.setSpacing(10)

        layout.addWidget(QLabel("متن کلید خصوصی (Base64، Hex یا PEM):"))
        self.txt_key = QTextEdit()
        self.txt_key.setFont(QFont("Consolas", 10))
        pem_hint = "-----" + "BEGIN" + " " + "PRIVATE" + " " + "KEY" + "-----\n...\nیا کد Base64 ۳۲ بایتی"
        self.txt_key.setPlaceholderText(pem_hint)
        layout.addWidget(self.txt_key, 1)

        g = QGridLayout()
        g.addWidget(QLabel("شناسه کلید (Key ID):"), 0, 0)
        self.ed_key_id = QLineEdit("daqiq-imported-v1")
        g.addWidget(self.ed_key_id, 0, 1)

        g.addWidget(QLabel("رمز عبور گاوصندوق جدید:"), 1, 0)
        self.ed_pass = QLineEdit()
        self.ed_pass.setEchoMode(QLineEdit.Password)
        g.addWidget(self.ed_pass, 1, 1)

        layout.addLayout(g)

        btn_row = QHBoxLayout()
        btn_import = QPushButton("وارد کردن و رمزنگاری در گاوصندوق")
        btn_import.setObjectName("BtnSuccess")
        btn_import.clicked.connect(self._on_import)
        btn_row.addWidget(btn_import)

        btn_cancel = QPushButton("انصراف")
        btn_cancel.setObjectName("BtnSecondary")
        btn_cancel.clicked.connect(self.reject)
        btn_row.addWidget(btn_cancel)

        layout.addLayout(btn_row)

    def _on_import(self) -> None:
        raw_text = self.txt_key.toPlainText().strip()
        key_id = self.ed_key_id.text().strip()
        p = self.ed_pass.text()

        if not raw_text or not key_id or not p:
            QMessageBox.warning(self, "خطا", "لطفاً تمام فیلدها را پر کنید.")
            return

        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "ذخیره گاوصندوق برای کلید واردشده",
            f"vault_{key_id}.dqvault",
            "فایل‌های گاوصندوق کلید (*.dqvault *.dqkey);;تمام فایل‌ها (*.*)",
        )
        if not file_path:
            return

        try:
            unlocked, _ = KeyVault.import_raw_private_key(
                raw_key_data=raw_text,
                key_id=key_id,
                passphrase=p,
                file_path=file_path,
            )
            self.result_vault = unlocked
            self.accept()
        except Exception as e:
            QMessageBox.critical(self, "خطا در وارد کردن کلید", str(e))


class TabVault(QWidget):
    """Tab 4: Key Vault & Settings."""

    vaultChanged = Signal(object)  # Emits new active UnlockedVault or None
    themeChanged = Signal(bool)    # Emits True for dark, False for light

    def __init__(
        self,
        get_active_vault_callback,
        set_active_vault_callback,
        is_dark_theme: bool = False,
        parent: Optional[QWidget] = None,
    ):
        super().__init__(parent)
        self.get_active_vault = get_active_vault_callback
        self.set_active_vault = set_active_vault_callback
        self.is_dark_theme = is_dark_theme

        self.setLayoutDirection(Qt.RightToLeft)
        self._build_ui()
        self.refresh_vault_display()

    def _build_ui(self) -> None:
        scroll = QScrollArea(self)
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.NoFrame)

        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(14)

        # 1. Active Vault Status Card
        layout.addWidget(self._build_status_card())

        # 2. Key Vault Operations Card
        layout.addWidget(self._build_actions_card())

        # 3. Preferences Card
        layout.addWidget(self._build_preferences_card())

        # 4. About App Box
        layout.addWidget(self._build_about_card())

        scroll.setWidget(container)
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(scroll)

    def _build_status_card(self) -> QGroupBox:
        box = QGroupBox("وضعیت گاوصندوق کلید امضا (Active Key Vault)")
        layout = QVBoxLayout(box)
        layout.setSpacing(10)

        # Pill & Lock button
        top_row = QHBoxLayout()
        self.lbl_vault_status_badge = QLabel("🔒 کلید امضا: قفل / بارگذاری نشده")
        self.lbl_vault_status_badge.setStyleSheet(
            "font-size: 13px; font-weight: bold; padding: 8px 16px; border-radius: 6px; background-color: #fee2e2; color: #991b1b;"
        )
        top_row.addWidget(self.lbl_vault_status_badge)
        top_row.addStretch()

        self.btn_lock_vault = QPushButton("🔒 قفل کردن کلید")
        self.btn_lock_vault.setObjectName("BtnDanger")
        self.btn_lock_vault.clicked.connect(self._on_lock_vault)
        top_row.addWidget(self.btn_lock_vault)

        layout.addLayout(top_row)

        # Details Grid
        g = QGridLayout()
        g.setSpacing(8)

        self.lbl_val_key_id = QLabel("-")
        self.lbl_val_fp = QLabel("-")
        self.lbl_val_pub_key = QLabel("-")
        self.lbl_val_pub_key.setFont(QFont("Consolas", 10))
        self.lbl_val_pub_key.setTextInteractionFlags(Qt.TextSelectableByMouse)

        g.addWidget(QLabel("شناسه کلید (Key ID):"), 0, 0)
        g.addWidget(self.lbl_val_key_id, 0, 1)

        g.addWidget(QLabel("اثرانگشت کلید (Fingerprint):"), 1, 0)
        g.addWidget(self.lbl_val_fp, 1, 1)

        g.addWidget(QLabel("کلید عمومی (Base64):"), 2, 0)
        g.addWidget(self.lbl_val_pub_key, 2, 1)

        layout.addLayout(g)

        # Copy buttons
        btn_row = QHBoxLayout()
        self.btn_copy_pub = QPushButton("📋 کپی کلید عمومی برای Daqiq")
        self.btn_copy_pub.clicked.connect(self._copy_public_key)
        btn_row.addWidget(self.btn_copy_pub)

        self.btn_copy_snippet = QPushButton("📋 کپی کد پایتون جهت درج در keys.py")
        self.btn_copy_snippet.setObjectName("BtnSecondary")
        self.btn_copy_snippet.clicked.connect(self._copy_code_snippet)
        btn_row.addWidget(self.btn_copy_snippet)

        btn_row.addStretch()
        layout.addLayout(btn_row)

        return box

    def _build_actions_card(self) -> QGroupBox:
        box = QGroupBox("عملیات و مدیریت گاوصندوق کلیدها")
        layout = QGridLayout(box)
        layout.setSpacing(10)

        # Action 1: Create New Vault
        btn_create = QPushButton("➕ ساخت گاوصندوق کلید جدید")
        btn_create.setObjectName("BtnSuccess")
        btn_create.clicked.connect(self._on_create_vault)
        layout.addWidget(btn_create, 0, 0)

        # Action 2: Open Existing Vault
        btn_open = QPushButton("📂 باز کردن فایل کلید موجود (.dqvault)")
        btn_open.clicked.connect(self._on_open_vault)
        layout.addWidget(btn_open, 0, 1)

        # Action 3: Change Passphrase
        btn_change_pass = QPushButton("🔑 تغییر رمز عبور گاوصندوق")
        btn_change_pass.setObjectName("BtnSecondary")
        btn_change_pass.clicked.connect(self._on_change_passphrase)
        layout.addWidget(btn_change_pass, 1, 0)

        # Action 4: Import Raw/PEM Key
        btn_import = QPushButton("📥 وارد کردن کلید خصوصی موجود")
        btn_import.setObjectName("BtnSecondary")
        btn_import.clicked.connect(self._on_import_key)
        layout.addWidget(btn_import, 1, 1)

        # Action 5: Backup Vault
        btn_backup = QPushButton("💾 پشتیبان‌گیری از فایل گاوصندوق")
        btn_backup.setObjectName("BtnSecondary")
        btn_backup.clicked.connect(self._on_backup_vault)
        layout.addWidget(btn_backup, 2, 0)

        return box

    def _build_preferences_card(self) -> QGroupBox:
        box = QGroupBox("تنظیمات و ترجیحات برنامه")
        layout = QGridLayout(box)
        layout.setSpacing(10)

        layout.addWidget(QLabel("تم ظاهری نرم‌افزار:"), 0, 0)
        self.cb_theme = QComboBox()
        self.cb_theme.addItem("روشن (Light Theme)", False)
        self.cb_theme.addItem("تیره (Dark Theme)", True)
        self.cb_theme.setCurrentIndex(1 if self.is_dark_theme else 0)
        self.cb_theme.currentIndexChanged.connect(self._on_theme_changed)
        layout.addWidget(self.cb_theme, 0, 1)

        layout.addWidget(QLabel("مسیر پیش‌فرض ذخیره لایسنس‌ها:"), 1, 0)
        path_row = QHBoxLayout()
        self.ed_default_dir = QLineEdit(os.path.expanduser("~"))
        path_row.addWidget(self.ed_default_dir, 1)

        btn_browse_dir = QPushButton("انتخاب پوشه...")
        btn_browse_dir.setObjectName("BtnSecondary")
        btn_browse_dir.clicked.connect(self._on_browse_output_dir)
        path_row.addWidget(btn_browse_dir)

        layout.addLayout(path_row, 1, 1)
        return box

    def _build_about_card(self) -> QFrame:
        frame = QFrame()
        frame.setObjectName("MetricCard")
        layout = QVBoxLayout(frame)
        layout.setSpacing(8)

        title = QLabel("مولد لایسنس تجاری دقیق — Daqiq License Generator v1.0.0")
        title.setStyleSheet("font-size: 14px; font-weight: bold; color: #0284c7;")
        layout.addWidget(title)

        desc = QLabel(
            "استودیوی آفلاین تولید، مدیریت و بازرسی لایسنس‌های رمزنگاری نامتقارن برای نرم‌افزار ERP دقیق (v4.9.0).\n"
            "مبتنی بر استانداردهای Ed25519 (RFC 8032)، گاوصندوق AES-256-GCM با ۶۰۰٬۰۰۰ دور PBKDF2، "
            "شناسه سخت‌افزاری پایدار SMBIOS (fp_v1)، و رابط کاربری اختصاصی راست‌به‌چپ فارسی (PySide6)."
        )
        desc.setWordWrap(True)
        desc.setStyleSheet("font-size: 11px; color: #64748b; line-height: 1.6;")
        layout.addWidget(desc)

        return frame

    def refresh_vault_display(self) -> None:
        vault = self.get_active_vault()
        if vault and vault.is_unlocked:
            self.lbl_vault_status_badge.setText(
                f"🔓 کلید امضا: فعال و آماده امضا [{vault.key_id} | اثرانگشت: {vault.short_fingerprint}]"
            )
            self.lbl_vault_status_badge.setStyleSheet(
                "font-size: 13px; font-weight: bold; padding: 8px 16px; border-radius: 6px; background-color: #d1fae5; color: #065f46;"
            )
            self.lbl_val_key_id.setText(vault.key_id)
            self.lbl_val_fp.setText(vault.fingerprint)
            self.lbl_val_pub_key.setText(vault.public_key_b64)
            self.btn_lock_vault.setEnabled(True)
            self.btn_copy_pub.setEnabled(True)
            self.btn_copy_snippet.setEnabled(True)
        else:
            self.lbl_vault_status_badge.setText("🔒 کلید امضا: قفل / بارگذاری نشده")
            self.lbl_vault_status_badge.setStyleSheet(
                "font-size: 13px; font-weight: bold; padding: 8px 16px; border-radius: 6px; background-color: #fee2e2; color: #991b1b;"
            )
            self.lbl_val_key_id.setText("-")
            self.lbl_val_fp.setText("-")
            self.lbl_val_pub_key.setText("-")
            self.btn_lock_vault.setEnabled(False)
            self.btn_copy_pub.setEnabled(False)
            self.btn_copy_snippet.setEnabled(False)

    def _on_create_vault(self) -> None:
        dlg = CreateVaultDialog(self)
        if dlg.exec() == QDialog.Accepted and dlg.result_vault:
            self.set_active_vault(dlg.result_vault)
            self.refresh_vault_display()
            self.vaultChanged.emit(dlg.result_vault)
            QMessageBox.information(
                self,
                "گاوصندوق ایجاد شد",
                f"گاوصندوق کلید با موفقیت ایجاد و ذخیره شد.\nشناسه کلید: {dlg.result_vault.key_id}",
            )

    def _on_open_vault(self) -> None:
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "انتخاب فایل گاوصندوق کلید امضا",
            "",
            "فایل‌های گاوصندوق کلید (*.dqvault *.dqkey);;تمام فایل‌ها (*.*)",
        )
        if not file_path:
            return

        passphrase, ok = QMessageBox.getText(
            self,
            "رمز عبور گاوصندوق",
            "لطفاً رمز عبور گاوصندوق را جهت بازگشایی وارد کنید:",
            QLineEdit.Password,
        )
        if not ok or not passphrase:
            return

        try:
            unlocked = KeyVault.unlock_vault_from_file(file_path, passphrase)
            self.set_active_vault(unlocked)
            self.refresh_vault_display()
            self.vaultChanged.emit(unlocked)
            QMessageBox.information(
                self,
                "گاوصندوق باز شد",
                f"کلید امضای {unlocked.key_id} با موفقیت در حافظه رم بازگشایی شد.",
            )
        except InvalidPassphraseError:
            QMessageBox.critical(self, "خطا", "رمز عبور اشتباه است یا فایل دستکاری شده است.")
        except Exception as e:
            QMessageBox.critical(self, "خطا در باز کردن گاوصندوق", str(e))

    def _on_lock_vault(self) -> None:
        vault = self.get_active_vault()
        if vault:
            vault.lock()
            self.set_active_vault(None)
            self.refresh_vault_display()
            self.vaultChanged.emit(None)
            QMessageBox.information(
                self,
                "قفل شد",
                "کلید خصوصی با موفقیت از حافظه موقت پاک و گاوصندوق قفل گردید.",
            )

    def _on_change_passphrase(self) -> None:
        vault = self.get_active_vault()
        if not vault or not vault.is_unlocked:
            QMessageBox.warning(self, "گاوصندوق قفل است", "ابتدا گاوصندوق را باز کنید.")
            return

        p1, ok1 = QMessageBox.getText(self, "رمز عبور جدید", "رمز عبور جدید را وارد کنید:", QLineEdit.Password)
        if not ok1 or not p1:
            return
        p2, ok2 = QMessageBox.getText(self, "تکرار رمز عبور", "تکرار رمز عبور جدید:", QLineEdit.Password)
        if not ok2 or p1 != p2:
            QMessageBox.warning(self, "خطا", "تکرار رمز عبور همخوانی ندارد.")
            return

        try:
            KeyVault.change_vault_passphrase(vault, p1)
            QMessageBox.information(self, "موفقیت", "رمز عبور گاوصندوق با موفقیت تغییر یافت.")
        except Exception as e:
            QMessageBox.critical(self, "خطا", str(e))

    def _on_import_key(self) -> None:
        dlg = ImportKeyDialog(self)
        if dlg.exec() == QDialog.Accepted and dlg.result_vault:
            self.set_active_vault(dlg.result_vault)
            self.refresh_vault_display()
            self.vaultChanged.emit(dlg.result_vault)
            QMessageBox.information(
                self,
                "کلید وارد شد",
                f"کلید خصوصی با شناسه {dlg.result_vault.key_id} با موفقیت در گاوصندوق ذخیره شد.",
            )

    def _on_backup_vault(self) -> None:
        vault = self.get_active_vault()
        if not vault or not vault.vault_path or not os.path.isfile(vault.vault_path):
            QMessageBox.warning(self, "خطا", "مسیر فایل گاوصندوق فعال مشخص نیست.")
            return

        save_path, _ = QFileDialog.getSaveFileName(
            self,
            "ذخیره نسخه پشتیبان گاوصندوق",
            f"backup_{os.path.basename(vault.vault_path)}",
            "فایل‌های گاوصندوق کلید (*.dqvault *.dqkey);;تمام فایل‌ها (*.*)",
        )
        if not save_path:
            return

        try:
            shutil.copy2(vault.vault_path, save_path)
            QMessageBox.information(self, "موفقیت", f"نسخه پشتیبان با موفقیت در مسیر زیر ایجاد شد:\n{save_path}")
        except Exception as e:
            QMessageBox.critical(self, "خطا در پشتیبان‌گیری", str(e))

    def _copy_public_key(self) -> None:
        vault = self.get_active_vault()
        if vault and vault.public_key_b64:
            pyperclip.copy(vault.public_key_b64)
            QMessageBox.information(self, "کپی شد", "کلید عمومی (Base64) در کلیپ‌بورد کپی شد.")

    def _copy_code_snippet(self) -> None:
        vault = self.get_active_vault()
        if vault:
            pyperclip.copy(vault.export_daqiq_code_snippet())
            QMessageBox.information(self, "کپی شد", "کد پایتون جهت تنظیم در Daqiq کپی شد.")

    def _on_theme_changed(self) -> None:
        is_dark = bool(self.cb_theme.currentData())
        self.is_dark_theme = is_dark
        self.themeChanged.emit(is_dark)

    def _on_browse_output_dir(self) -> None:
        folder = QFileDialog.getExistingDirectory(self, "انتخاب پوشه پیش‌فرض خروجی لایسنس‌ها")
        if folder:
            self.ed_default_dir.setText(folder)
