use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::MoveWindow;

use crate::state::{ActiveTab, AppState};
use crate::win32_util::{
    create_button, create_label, create_label_left, set_window_text, Fonts,
};

pub const ID_TAB_GEN: isize = 1001;
pub const ID_TAB_LEDGER: isize = 1002;
pub const ID_TAB_INSPECTOR: isize = 1003;
pub const ID_TAB_VAULT: isize = 1004;
pub const ID_BTN_THEME: isize = 1005;

pub struct SidebarView {
    pub hwnd: HWND,
    pub lbl_title: HWND,
    pub lbl_subtitle: HWND,
    pub btn_gen: HWND,
    pub btn_ledger: HWND,
    pub btn_inspector: HWND,
    pub btn_vault: HWND,
    pub lbl_vault_box: HWND,
    pub lbl_key_id: HWND,
    pub lbl_fingerprint: HWND,
    pub lbl_pairing_badge: HWND,
    pub btn_theme: HWND,
}

impl SidebarView {
    pub fn create(parent: HWND, fonts: &Fonts) -> Self {
        let lbl_title = create_label(
            parent,
            "استودیو لایسنس دقیق",
            0, 15, 210, 26,
            1010,
            fonts.heading,
        );
        let lbl_subtitle = create_label(
            parent,
            "نسخه ۱.۴.۰ - Win32 Native",
            0, 44, 210, 18,
            1011,
            fonts.small,
        );

        let btn_gen = create_button(
            parent,
            "۱. صدور لایسنس جدید  ▶",
            10, 80, 200, 38,
            ID_TAB_GEN,
            fonts.subheading,
        );
        let btn_ledger = create_button(
            parent,
            "۲. دفتر کل لایسنس‌ها",
            10, 126, 200, 38,
            ID_TAB_LEDGER,
            fonts.subheading,
        );
        let btn_inspector = create_button(
            parent,
            "۳. بازرسی و شبیه‌ساز",
            10, 172, 200, 38,
            ID_TAB_INSPECTOR,
            fonts.subheading,
        );
        let btn_vault = create_button(
            parent,
            "۴. گاوصندوق و تطابق",
            10, 218, 200, 38,
            ID_TAB_VAULT,
            fonts.subheading,
        );

        // Active Key Status Box
        let lbl_vault_box = create_label(
            parent,
            "وضعیت کلید فعال:",
            10, 280, 200, 20,
            1020,
            fonts.bold,
        );
        let lbl_key_id = create_label_left(
            parent,
            "Key ID: daqiq-prod-v1",
            10, 305, 200, 18,
            1021,
            fonts.small,
        );
        let lbl_fingerprint = create_label_left(
            parent,
            "FP: ...",
            10, 325, 200, 18,
            1022,
            fonts.mono,
        );
        let lbl_pairing_badge = create_label(
            parent,
            "تطابق با Daqiq: در حال بررسی...",
            10, 350, 200, 45,
            1023,
            fonts.small,
        );

        let btn_theme = create_button(
            parent,
            "تغییر تم (روشن / تیره)",
            10, 410, 200, 34,
            ID_BTN_THEME,
            fonts.regular,
        );

        Self {
            hwnd: parent,
            lbl_title,
            lbl_subtitle,
            btn_gen,
            btn_ledger,
            btn_inspector,
            btn_vault,
            lbl_vault_box,
            lbl_key_id,
            lbl_fingerprint,
            lbl_pairing_badge,
            btn_theme,
        }
    }

    pub fn reposition(&self, x: i32, _y: i32, w: i32, h: i32) {
        let bw = w - 20;
        unsafe {
            MoveWindow(self.lbl_title, x, 15, w, 26, 1);
            MoveWindow(self.lbl_subtitle, x, 44, w, 18, 1);
            MoveWindow(self.btn_gen, x + 10, 80, bw, 38, 1);
            MoveWindow(self.btn_ledger, x + 10, 126, bw, 38, 1);
            MoveWindow(self.btn_inspector, x + 10, 172, bw, 38, 1);
            MoveWindow(self.btn_vault, x + 10, 218, bw, 38, 1);

            MoveWindow(self.lbl_vault_box, x + 10, 280, bw, 20, 1);
            MoveWindow(self.lbl_key_id, x + 10, 305, bw, 18, 1);
            MoveWindow(self.lbl_fingerprint, x + 10, 325, bw, 18, 1);
            MoveWindow(self.lbl_pairing_badge, x + 10, 350, bw, 45, 1);

            MoveWindow(self.btn_theme, x + 10, h - 50, bw, 34, 1);
        }
    }

    pub fn update_from_state(&self, state: &AppState) {
        let gen_title = if state.active_tab == ActiveTab::Generator {
            "۱. صدور لایسنس جدید  ◀"
        } else {
            "۱. صدور لایسنس جدید"
        };
        let ledger_title = if state.active_tab == ActiveTab::Ledger {
            "۲. دفتر کل لایسنس‌ها  ◀"
        } else {
            "۲. دفتر کل لایسنس‌ها"
        };
        let inspector_title = if state.active_tab == ActiveTab::Inspector {
            "۳. بازرسی و شبیه‌ساز  ◀"
        } else {
            "۳. بازرسی و شبیه‌ساز"
        };
        let vault_title = if state.active_tab == ActiveTab::Vault {
            "۴. گاوصندوق و تطابق  ◀"
        } else {
            "۴. گاوصندوق و تطابق"
        };

        set_window_text(self.btn_gen, gen_title);
        set_window_text(self.btn_ledger, ledger_title);
        set_window_text(self.btn_inspector, inspector_title);
        set_window_text(self.btn_vault, vault_title);

        set_window_text(self.lbl_key_id, &format!("Key: {}", state.active_key_id));
        let fp_short = if state.active_key_fingerprint.len() > 16 {
            &state.active_key_fingerprint[..16]
        } else {
            &state.active_key_fingerprint
        };
        set_window_text(self.lbl_fingerprint, &format!("FP: {}...", fp_short));

        let pairing_text = match &state.pairing_status {
            Some(daqiq_lic_core::PublicKeyMatchResult::Match) => {
                "تطابق کلید: ۱۰۰٪ همگام با Daqiq v4.9.1 ✓"
            }
            Some(daqiq_lic_core::PublicKeyMatchResult::Mismatch { .. }) => {
                "هشدار: عدم تطابق با کلید پیش‌فرض Daqiq!"
            }
            _ => "تطابق با Daqiq: نامشخص",
        };
        set_window_text(self.lbl_pairing_badge, pairing_text);
    }
}
