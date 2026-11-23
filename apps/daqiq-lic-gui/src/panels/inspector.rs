use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{MoveWindow, ShowWindow, SW_HIDE, SW_SHOW};

use crate::state::AppState;
use crate::win32_util::{create_button, create_edit, create_label, set_window_text, Fonts};

pub const ID_TXT_INSPECT_INPUT: isize = 1301;
pub const ID_BTN_LOAD_FILE: isize = 1302;
pub const ID_TXT_CUSTOM_PUBKEY: isize = 1303;
pub const ID_BTN_VERIFY_INSPECT: isize = 1304;

pub const ID_TXT_SIM_HWID: isize = 1306;
pub const ID_TXT_SIM_VERSION: isize = 1307;
pub const ID_BTN_SIMULATE: isize = 1308;

pub struct InspectorView {
    pub parent: HWND,
    pub lbl_header: HWND,

    pub lbl_input_desc: HWND,
    pub txt_input: HWND,
    pub btn_load_file: HWND,

    pub lbl_custom_pubkey: HWND,
    pub txt_custom_pubkey: HWND,
    pub btn_verify: HWND,

    pub lbl_result_box: HWND,
    pub txt_result_details: HWND,

    // Simulator Section
    pub lbl_sim_header: HWND,
    pub lbl_sim_hwid: HWND,
    pub txt_sim_hwid: HWND,
    pub lbl_sim_version: HWND,
    pub txt_sim_version: HWND,
    pub btn_simulate: HWND,
    pub lbl_sim_result: HWND,
}

impl InspectorView {
    pub fn create(parent: HWND, fonts: &Fonts) -> Self {
        let lbl_header = create_label(
            parent,
            "بازرسی، ارزیابی امضا و شبیه‌ساز انطباق با Daqiq v4.9.1",
            20, 10, 700, 24,
            1300,
            fonts.heading,
        );

        let lbl_input_desc = create_label(
            parent,
            "محتوای فایل لایسنس (JSON کامل) یا توکن فشرده Base64 را در کادر زیر وارد کنید:",
            20, 42, 600, 18,
            0,
            fonts.subheading,
        );
        let txt_input = create_edit(
            parent,
            "",
            20, 65, 750, 140,
            ID_TXT_INSPECT_INPUT,
            fonts.mono,
            false,
            true,
            false,
            false,
        );

        let btn_load_file = create_button(
            parent,
            "بارگذاری فایل (.daqlic)...",
            20, 212, 200, 32,
            ID_BTN_LOAD_FILE,
            fonts.regular,
        );

        let lbl_custom_pubkey = create_label(
            parent,
            "کلید عمومی سفارشی (اختیاری):",
            240, 218, 180, 18,
            0,
            fonts.small,
        );
        let txt_custom_pubkey = create_edit(
            parent,
            "",
            425, 214, 250, 26,
            ID_TXT_CUSTOM_PUBKEY,
            fonts.mono,
            false,
            false,
            false,
            false,
        );

        let btn_verify = create_button(
            parent,
            "راستی‌آزمایی امضا و ساختار  ✔",
            685, 212, 210, 32,
            ID_BTN_VERIFY_INSPECT,
            fonts.bold,
        );

        // Result Details
        let lbl_result_box = create_label(
            parent,
            "نتیجه ارزیابی امضا: در انتظار بررسی...",
            20, 255, 600, 22,
            0,
            fonts.bold,
        );
        let txt_result_details = create_edit(
            parent,
            "",
            20, 280, 750, 120,
            0,
            fonts.regular,
            true,
            true,
            true,
            false,
        );

        // Simulator
        let lbl_sim_header = create_label(
            parent,
            "شبیه‌ساز ارزیابی لایسنس در هنگام اجرا توسط Daqiq v4.9.1 (PR #39)",
            20, 415, 600, 22,
            0,
            fonts.subheading,
        );

        let lbl_sim_hwid = create_label(parent, "شناسه دستگاه تست (HWID):", 20, 445, 200, 18, 0, fonts.small);
        let txt_sim_hwid = create_edit(parent, "", 20, 465, 380, 26, ID_TXT_SIM_HWID, fonts.mono, false, false, false, false);

        let lbl_sim_version = create_label(parent, "نسخه نرم‌افزار Daqiq:", 420, 445, 150, 18, 0, fonts.small);
        let txt_sim_version = create_edit(parent, "4.9.1", 420, 465, 140, 26, ID_TXT_SIM_VERSION, fonts.regular, false, false, false, false);

        let btn_simulate = create_button(parent, "شبیه‌سازی بررسی در Daqiq  ▶", 580, 463, 210, 30, ID_BTN_SIMULATE, fonts.bold);

        let lbl_sim_result = create_label(
            parent,
            "نتیجه شبیه‌سازی: لایسنسی تست نشده است.",
            20, 505, 750, 40,
            0,
            fonts.bold,
        );

        Self {
            parent,
            lbl_header,
            lbl_input_desc,
            txt_input,
            btn_load_file,
            lbl_custom_pubkey,
            txt_custom_pubkey,
            btn_verify,
            lbl_result_box,
            txt_result_details,
            lbl_sim_header,
            lbl_sim_hwid,
            txt_sim_hwid,
            lbl_sim_version,
            txt_sim_version,
            btn_simulate,
            lbl_sim_result,
        }
    }

    pub fn reposition(&self, total_w: i32, _total_h: i32) {
        let content_w = total_w - 40;
        unsafe {
            MoveWindow(self.lbl_header, 20, 10, content_w, 24, 1);
            MoveWindow(self.txt_input, 20, 65, content_w, 140, 1);
            MoveWindow(self.txt_result_details, 20, 280, content_w, 120, 1);
            MoveWindow(self.lbl_sim_result, 20, 505, content_w, 40, 1);
        }
    }

    pub fn show(&self, visible: bool) {
        let flag = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.lbl_header, flag);
            ShowWindow(self.lbl_input_desc, flag);
            ShowWindow(self.txt_input, flag);
            ShowWindow(self.btn_load_file, flag);
            ShowWindow(self.lbl_custom_pubkey, flag);
            ShowWindow(self.txt_custom_pubkey, flag);
            ShowWindow(self.btn_verify, flag);
            ShowWindow(self.lbl_result_box, flag);
            ShowWindow(self.txt_result_details, flag);
            ShowWindow(self.lbl_sim_header, flag);
            ShowWindow(self.lbl_sim_hwid, flag);
            ShowWindow(self.txt_sim_hwid, flag);
            ShowWindow(self.lbl_sim_version, flag);
            ShowWindow(self.txt_sim_version, flag);
            ShowWindow(self.btn_simulate, flag);
            ShowWindow(self.lbl_sim_result, flag);
        }
    }

    pub fn update_from_state(&self, state: &AppState) {
        if let Some((_valid, title, details)) = &state.inspect_status {
            set_window_text(self.lbl_result_box, &format!("نتیجه: {title}"));
            set_window_text(self.txt_result_details, details);
        }

        if let Some((_valid, title, details)) = &state.sim_status {
            set_window_text(self.lbl_sim_result, &format!("{title} - {details}"));
        }
    }
}
