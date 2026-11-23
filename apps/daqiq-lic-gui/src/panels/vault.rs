use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{MoveWindow, ShowWindow, SW_HIDE, SW_SHOW};

use daqiq_lic_core::BinaryPairingResult;

use crate::state::AppState;
use crate::win32_util::{
    create_button, create_edit, create_label, create_label_left, set_window_text, Fonts,
};

pub const ID_BTN_COPY_PUBKEY: isize = 1402;
pub const ID_BTN_SCAN_BINARY: isize = 1403;

pub const ID_TXT_UNLOCK_PASS: isize = 1406;
pub const ID_BTN_UNLOCK: isize = 1407;

pub const ID_TXT_CREATE_ID: isize = 1408;
pub const ID_TXT_CREATE_PASS: isize = 1409;
pub const ID_BTN_CREATE: isize = 1410;

pub const ID_TXT_IMPORT_ID: isize = 1411;
pub const ID_TXT_IMPORT_RAW: isize = 1412;
pub const ID_TXT_IMPORT_PASS: isize = 1413;
pub const ID_BTN_IMPORT: isize = 1414;

pub struct VaultView {
    pub parent: HWND,
    pub lbl_header: HWND,

    // Card 1: Active Key
    pub lbl_card1_title: HWND,
    pub lbl_key_id: HWND,
    pub lbl_pubkey_title: HWND,
    pub txt_pubkey: HWND,
    pub btn_copy_pubkey: HWND,
    pub lbl_fingerprint: HWND,
    pub lbl_pairing_badge: HWND,

    // Card 2: Binary Pairing Diagnostic Scanner
    pub lbl_card2_title: HWND,
    pub lbl_card2_desc: HWND,
    pub btn_scan_binary: HWND,
    pub lbl_binary_path: HWND,
    pub lbl_binary_result: HWND,

    // Card 3: Security Operations
    pub lbl_card3_title: HWND,

    // Unlock
    pub lbl_unlock_title: HWND,
    pub txt_unlock_pass: HWND,
    pub btn_unlock: HWND,

    // Create
    pub lbl_create_title: HWND,
    pub txt_create_id: HWND,
    pub txt_create_pass: HWND,
    pub btn_create: HWND,

    // Import
    pub lbl_import_title: HWND,
    pub txt_import_id: HWND,
    pub txt_import_raw: HWND,
    pub txt_import_pass: HWND,
    pub btn_import: HWND,
}

impl VaultView {
    pub fn create(parent: HWND, fonts: &Fonts) -> Self {
        let lbl_header = create_label(
            parent,
            "گاوصندوق کلیدها و آزمون تطابق کلید (Key-Pairing Diagnostics)",
            20, 10, 700, 24,
            1400,
            fonts.heading,
        );

        // Card 1
        let lbl_card1_title = create_label(parent, "۱. کلید فعال برای صدور و امضای لایسنس:", 20, 42, 600, 20, 0, fonts.subheading);
        let lbl_key_id = create_label_left(parent, "Key ID: daqiq-prod-v1", 20, 68, 300, 18, 0, fonts.bold);
        let lbl_pubkey_title = create_label_left(parent, "Public Key (Base64):", 20, 92, 200, 18, 0, fonts.small);
        let txt_pubkey = create_edit(parent, "", 20, 112, 500, 24, 1401, fonts.mono, false, false, true, false);
        let btn_copy_pubkey = create_button(parent, "کپی کلید عمومی", 530, 110, 150, 28, ID_BTN_COPY_PUBKEY, fonts.regular);
        let lbl_fingerprint = create_label_left(parent, "Fingerprint: ...", 20, 142, 650, 18, 0, fonts.mono);
        let lbl_pairing_badge = create_label(parent, "وضعیت تطابق: در حال استعلام...", 20, 166, 650, 22, 0, fonts.bold);

        // Card 2: Binary Pairing Diagnostic Scanner
        let lbl_card2_title = create_label(
            parent,
            "۲. آزمون تطابق باینری Daqiq.exe (Binary Pairing Diagnostic Scanner)",
            20, 200, 700, 20,
            0,
            fonts.subheading,
        );
        let lbl_card2_desc = create_label(
            parent,
            "بررسی مستقیم فایل Daqiq.exe یا licensing/keys.py جهت اطمینان از قرار داشتن کلید عمومی فعال:",
            20, 224, 750, 18,
            0,
            fonts.regular,
        );
        let btn_scan_binary = create_button(
            parent,
            "انتخاب فایل Daqiq.exe یا keys.py جهت بررسی تطابق...",
            20, 248, 380, 32,
            ID_BTN_SCAN_BINARY,
            fonts.bold,
        );
        let lbl_binary_path = create_label_left(parent, "مسیر فایل: (هیچ فایلی انتخاب نشده است)", 20, 285, 750, 18, 0, fonts.small);
        let lbl_binary_result = create_label(
            parent,
            "نتیجه بررسی: در انتظار انتخاب فایل Daqiq.exe...",
            20, 308, 750, 48,
            0,
            fonts.bold,
        );

        // Card 3: Security Operations
        let lbl_card3_title = create_label(parent, "۳. عملیات گاوصندوق کلیدها:", 20, 365, 500, 20, 0, fonts.subheading);

        // Unlock
        let lbl_unlock_title = create_label(parent, "بازگشایی گاوصندوق با کلمه عبور:", 20, 395, 250, 18, 0, fonts.small);
        let txt_unlock_pass = create_edit(parent, "", 20, 415, 200, 26, ID_TXT_UNLOCK_PASS, fonts.regular, false, false, false, true);
        let btn_unlock = create_button(parent, "بازگشایی", 230, 414, 100, 28, ID_BTN_UNLOCK, fonts.regular);

        // Create
        let lbl_create_title = create_label(parent, "ایجاد کلید جدید در گاوصندوق:", 20, 455, 300, 18, 0, fonts.small);
        let txt_create_id = create_edit(parent, "daqiq-prod-v2", 20, 475, 150, 26, ID_TXT_CREATE_ID, fonts.regular, false, false, false, false);
        let txt_create_pass = create_edit(parent, "", 180, 475, 150, 26, ID_TXT_CREATE_PASS, fonts.regular, false, false, false, true);
        let btn_create = create_button(parent, "ساخت کلید", 340, 474, 110, 28, ID_BTN_CREATE, fonts.regular);

        // Import
        let lbl_import_title = create_label(parent, "ورود کلید خصوصی موجود (Base64 یا Hex):", 20, 515, 350, 18, 0, fonts.small);
        let txt_import_id = create_edit(parent, "daqiq-imported-v1", 20, 535, 140, 26, ID_TXT_IMPORT_ID, fonts.regular, false, false, false, false);
        let txt_import_raw = create_edit(parent, "", 170, 535, 200, 26, ID_TXT_IMPORT_RAW, fonts.mono, false, false, false, true);
        let txt_import_pass = create_edit(parent, "", 380, 535, 140, 26, ID_TXT_IMPORT_PASS, fonts.regular, false, false, false, true);
        let btn_import = create_button(parent, "وارد کردن کلید", 530, 534, 120, 28, ID_BTN_IMPORT, fonts.regular);

        Self {
            parent,
            lbl_header,
            lbl_card1_title,
            lbl_key_id,
            lbl_pubkey_title,
            txt_pubkey,
            btn_copy_pubkey,
            lbl_fingerprint,
            lbl_pairing_badge,
            lbl_card2_title,
            lbl_card2_desc,
            btn_scan_binary,
            lbl_binary_path,
            lbl_binary_result,
            lbl_card3_title,
            lbl_unlock_title,
            txt_unlock_pass,
            btn_unlock,
            lbl_create_title,
            txt_create_id,
            txt_create_pass,
            btn_create,
            lbl_import_title,
            txt_import_id,
            txt_import_raw,
            txt_import_pass,
            btn_import,
        }
    }

    pub fn reposition(&self, total_w: i32, _total_h: i32) {
        let content_w = total_w - 40;
        unsafe {
            MoveWindow(self.lbl_header, 20, 10, content_w, 24, 1);
            MoveWindow(self.lbl_binary_result, 20, 308, content_w, 48, 1);
        }
    }

    pub fn show(&self, visible: bool) {
        let flag = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.lbl_header, flag);
            ShowWindow(self.lbl_card1_title, flag);
            ShowWindow(self.lbl_key_id, flag);
            ShowWindow(self.lbl_pubkey_title, flag);
            ShowWindow(self.txt_pubkey, flag);
            ShowWindow(self.btn_copy_pubkey, flag);
            ShowWindow(self.lbl_fingerprint, flag);
            ShowWindow(self.lbl_pairing_badge, flag);

            ShowWindow(self.lbl_card2_title, flag);
            ShowWindow(self.lbl_card2_desc, flag);
            ShowWindow(self.btn_scan_binary, flag);
            ShowWindow(self.lbl_binary_path, flag);
            ShowWindow(self.lbl_binary_result, flag);

            ShowWindow(self.lbl_card3_title, flag);
            ShowWindow(self.lbl_unlock_title, flag);
            ShowWindow(self.txt_unlock_pass, flag);
            ShowWindow(self.btn_unlock, flag);

            ShowWindow(self.lbl_create_title, flag);
            ShowWindow(self.txt_create_id, flag);
            ShowWindow(self.txt_create_pass, flag);
            ShowWindow(self.btn_create, flag);

            ShowWindow(self.lbl_import_title, flag);
            ShowWindow(self.txt_import_id, flag);
            ShowWindow(self.txt_import_raw, flag);
            ShowWindow(self.txt_import_pass, flag);
            ShowWindow(self.btn_import, flag);
        }
    }

    pub fn update_from_state(&self, state: &AppState) {
        set_window_text(self.lbl_key_id, &format!("Key ID: {}", state.active_key_id));
        set_window_text(self.txt_pubkey, &state.active_public_key);
        set_window_text(self.lbl_fingerprint, &format!("Key FP: {}", state.key_fp_display()));

        let pairing_text = if state.is_paired_with_official_daqiq() {
            "وضعیت تطابق: ۱۰۰٪ منطبق با کلید رسمی Daqiq v4.9.2 (Paired) ✓"
        } else {
            "هشدار: مغایرت با کلید رسمی Daqiq v4.9.2 (7925-3864-0164)! لایسنس‌های صادره رد خواهند شد."
        };
        set_window_text(self.lbl_pairing_badge, pairing_text);

        if !state.binary_pairing_path.is_empty() {
            set_window_text(self.lbl_binary_path, &format!("مسیر فایل: {}", state.binary_pairing_path));
        }

        if let Some(res) = &state.binary_pairing_status {
            let res_text = match res {
                BinaryPairingResult::Paired { matched_encoding, offset, public_key_fingerprint } => {
                    format!("تطابق کامل احراز گردید! کلید فعال با انکودینگ {matched_encoding} در آفست {offset} شناسایی شد (اثر انگشت: {public_key_fingerprint}) ✓")
                }
                BinaryPairingResult::Mismatch { guidance_fa, .. } => {
                    guidance_fa.clone()
                }
                BinaryPairingResult::FileNotFound => "خطا: فایل انتخابی یافت نشد.".to_string(),
                BinaryPairingResult::IoError(e) => format!("خطای خواندن فایل: {e}"),
                BinaryPairingResult::InvalidActiveKey(e) => format!("خطا در کلید فعال: {e}"),
            };
            set_window_text(self.lbl_binary_result, &res_text);
        }
    }
}
