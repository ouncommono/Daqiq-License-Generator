use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{MoveWindow, ShowWindow, SW_HIDE, SW_SHOW};

use crate::state::AppState;
use crate::win32_util::{
    add_combobox_item, create_button, create_checkbox, create_combobox, create_edit, create_label,
    get_checkbox_checked, get_combobox_selected_index, get_window_text, set_checkbox_checked,
    set_combobox_selected_index, set_window_text, Fonts,
};

pub const ID_PRESET_TRIAL: isize = 1101;
pub const ID_PRESET_STANDARD: isize = 1102;
pub const ID_PRESET_PRO: isize = 1103;
pub const ID_PRESET_ENTERPRISE: isize = 1104;
pub const ID_PRESET_PERPETUAL: isize = 1105;

pub const ID_TXT_CUSTOMER_NAME: isize = 1110;
pub const ID_TXT_LICENSE_ID: isize = 1111;
pub const ID_TXT_CUSTOMER_ORG: isize = 1112;
pub const ID_TXT_CUSTOMER_CONTACT: isize = 1113;
pub const ID_TXT_HWID: isize = 1114;
pub const ID_CB_MAX_DEVICES: isize = 1115;
pub const ID_CB_MATCH_POLICY: isize = 1116;

pub const ID_CB_LICENSE_TYPE: isize = 1117;
pub const ID_TXT_ISSUED_DATE: isize = 1118;
pub const ID_TXT_NOT_BEFORE_DATE: isize = 1119;
pub const ID_TXT_EXPIRES_DATE: isize = 1120;
pub const ID_TXT_GRACE_DAYS: isize = 1121;
pub const ID_CB_GRACE_MODE: isize = 1122;
pub const ID_TXT_WARNING_DAYS: isize = 1123;
pub const ID_TXT_CLOCK_SKEW: isize = 1124;
pub const ID_TXT_MIN_VERSION: isize = 1125;
pub const ID_TXT_MAX_VERSION: isize = 1126;

pub const ID_CHK_FEATURE_BASE: isize = 1130; // 1130..1139

pub const ID_LBL_STATUS: isize = 1149;
pub const ID_BTN_GENERATE_SAVE: isize = 1150;
pub const ID_BTN_COPY_TOKEN: isize = 1151;
pub const ID_BTN_PREVIEW_JSON: isize = 1152;

pub struct GeneratorView {
    pub parent: HWND,
    pub lbl_header: HWND,
    pub btn_preset_trial: HWND,
    pub btn_preset_standard: HWND,
    pub btn_preset_pro: HWND,
    pub btn_preset_enterprise: HWND,
    pub btn_preset_perpetual: HWND,

    // Column 1 (Right in RTL)
    pub lbl_cust_name: HWND,
    pub txt_cust_name: HWND,
    pub lbl_lic_id: HWND,
    pub txt_lic_id: HWND,
    pub lbl_cust_org: HWND,
    pub txt_cust_org: HWND,
    pub lbl_cust_contact: HWND,
    pub txt_cust_contact: HWND,
    pub lbl_hwid: HWND,
    pub txt_hwid: HWND,
    pub lbl_max_devices: HWND,
    pub cb_max_devices: HWND,
    pub lbl_match_policy: HWND,
    pub cb_match_policy: HWND,

    // Column 2 (Left in RTL)
    pub lbl_lic_type: HWND,
    pub cb_lic_type: HWND,
    pub lbl_issued_date: HWND,
    pub txt_issued_date: HWND,
    pub lbl_not_before: HWND,
    pub txt_not_before: HWND,
    pub lbl_expires: HWND,
    pub txt_expires: HWND,

    pub lbl_grace_days: HWND,
    pub txt_grace_days: HWND,
    pub lbl_grace_mode: HWND,
    pub cb_grace_mode: HWND,
    pub lbl_warning_days: HWND,
    pub txt_warning_days: HWND,
    pub lbl_clock_skew: HWND,
    pub txt_clock_skew: HWND,

    pub lbl_min_ver: HWND,
    pub txt_min_ver: HWND,
    pub lbl_max_ver: HWND,
    pub txt_max_ver: HWND,

    pub lbl_features_title: HWND,
    pub chk_features: Vec<(String, HWND)>,

    // Bottom Action Bar
    pub lbl_status: HWND,
    pub btn_generate_save: HWND,
    pub btn_copy_token: HWND,
    pub btn_preview_json: HWND,
}

impl GeneratorView {
    pub fn create(parent: HWND, fonts: &Fonts) -> Self {
        let lbl_header = create_label(
            parent,
            "صدور لایسنس تجاری Daqiq v4.9.2 (تطابق کامل PR #40)",
            20, 10, 700, 24,
            1100,
            fonts.heading,
        );

        let btn_preset_trial = create_button(parent, "آزمایشی (Trial)", 20, 40, 110, 28, ID_PRESET_TRIAL, fonts.small);
        let btn_preset_standard = create_button(parent, "استاندارد (Standard)", 135, 40, 125, 28, ID_PRESET_STANDARD, fonts.small);
        let btn_preset_pro = create_button(parent, "حرفه‌ای (Pro)", 265, 40, 105, 28, ID_PRESET_PRO, fonts.small);
        let btn_preset_enterprise = create_button(parent, "سازمانی (Enterprise)", 375, 40, 130, 28, ID_PRESET_ENTERPRISE, fonts.small);
        let btn_preset_perpetual = create_button(parent, "دائمی (Perpetual)", 510, 40, 120, 28, ID_PRESET_PERPETUAL, fonts.small);

        // Column 1 (Right side)
        let lbl_cust_name = create_label(parent, "نام مشتری / فروشگاه *", 520, 75, 300, 18, 0, fonts.subheading);
        let txt_cust_name = create_edit(parent, "", 520, 95, 350, 26, ID_TXT_CUSTOMER_NAME, fonts.regular, true, false, false, false);

        let lbl_lic_id = create_label(parent, "شناسه لایسنس *", 520, 125, 300, 18, 0, fonts.small);
        let txt_lic_id = create_edit(parent, "", 520, 145, 350, 26, ID_TXT_LICENSE_ID, fonts.mono, false, false, false, false);

        let lbl_cust_org = create_label(parent, "سازمان / شرکت (اختیاری)", 520, 175, 300, 18, 0, fonts.small);
        let txt_cust_org = create_edit(parent, "", 520, 195, 350, 26, ID_TXT_CUSTOMER_ORG, fonts.regular, true, false, false, false);

        let lbl_cust_contact = create_label(parent, "تلفن / ایمیل تماس (اختیاری)", 520, 225, 300, 18, 0, fonts.small);
        let txt_cust_contact = create_edit(parent, "", 520, 245, 350, 26, ID_TXT_CUSTOMER_CONTACT, fonts.regular, false, false, false, false);

        let lbl_hwid = create_label(parent, "شناسه سخت‌افزاری دستگاه‌ها (HWID - هر دستگاه در یک خط) *", 520, 275, 450, 18, 0, fonts.subheading);
        let txt_hwid = create_edit(parent, "", 520, 295, 450, 140, ID_TXT_HWID, fonts.mono, false, true, false, false);

        let lbl_max_devices = create_label(parent, "سقف مجاز دستگاه‌ها", 520, 445, 180, 18, 0, fonts.small);
        let cb_max_devices = create_combobox(parent, 520, 465, 140, 150, ID_CB_MAX_DEVICES, fonts.regular);
        add_combobox_item(cb_max_devices, "1 دستگاه");
        add_combobox_item(cb_max_devices, "2 دستگاه");
        add_combobox_item(cb_max_devices, "3 دستگاه");
        add_combobox_item(cb_max_devices, "5 دستگاه");
        add_combobox_item(cb_max_devices, "10 دستگاه");
        add_combobox_item(cb_max_devices, "25 دستگاه");
        set_combobox_selected_index(cb_max_devices, 0);

        let lbl_match_policy = create_label(parent, "سیاست تطابق سخت‌افزار", 680, 445, 200, 18, 0, fonts.small);
        let cb_match_policy = create_combobox(parent, 680, 465, 160, 150, ID_CB_MATCH_POLICY, fonts.regular);
        add_combobox_item(cb_match_policy, "standard (استاندارد)");
        add_combobox_item(cb_match_policy, "strict (سخت‌گیرانه)");
        add_combobox_item(cb_match_policy, "relaxed (منعطف)");
        set_combobox_selected_index(cb_match_policy, 0);

        // Column 2 (Left side)
        let lbl_lic_type = create_label(parent, "نوع اشتراک / لایسنس", 20, 75, 200, 18, 0, fonts.subheading);
        let cb_lic_type = create_combobox(parent, 20, 95, 220, 150, ID_CB_LICENSE_TYPE, fonts.regular);
        add_combobox_item(cb_lic_type, "subscription (اشتراکی)");
        add_combobox_item(cb_lic_type, "perpetual (دائمی)");
        add_combobox_item(cb_lic_type, "trial (آزمایشی)");
        set_combobox_selected_index(cb_lic_type, 0);

        let lbl_issued_date = create_label(parent, "تاریخ صدور (شمسی)", 20, 125, 200, 18, 0, fonts.small);
        let txt_issued_date = create_edit(parent, "", 20, 145, 150, 26, ID_TXT_ISSUED_DATE, fonts.mono, false, false, false, false);

        let lbl_not_before = create_label(parent, "شروع اعتبار (شمسی)", 180, 125, 200, 18, 0, fonts.small);
        let txt_not_before = create_edit(parent, "", 180, 145, 150, 26, ID_TXT_NOT_BEFORE_DATE, fonts.mono, false, false, false, false);

        let lbl_expires = create_label(parent, "تاریخ انقضا (شمسی)", 340, 125, 200, 18, 0, fonts.small);
        let txt_expires = create_edit(parent, "", 340, 145, 150, 26, ID_TXT_EXPIRES_DATE, fonts.mono, false, false, false, false);

        // PR #39 Timing & Tolerances
        let lbl_grace_days = create_label(parent, "مهلت تمدید (روز)", 20, 180, 110, 18, 0, fonts.small);
        let txt_grace_days = create_edit(parent, "7", 20, 200, 100, 26, ID_TXT_GRACE_DAYS, fonts.regular, false, false, false, false);

        let lbl_grace_mode = create_label(parent, "حالت مهلت تمدید", 130, 180, 110, 18, 0, fonts.small);
        let cb_grace_mode = create_combobox(parent, 130, 200, 110, 120, ID_CB_GRACE_MODE, fonts.regular);
        add_combobox_item(cb_grace_mode, "readonly");
        add_combobox_item(cb_grace_mode, "full");
        set_combobox_selected_index(cb_grace_mode, 0);

        let lbl_warning_days = create_label(parent, "هشدار انقضا (روز)", 250, 180, 110, 18, 0, fonts.small);
        let txt_warning_days = create_edit(parent, "14", 250, 200, 100, 26, ID_TXT_WARNING_DAYS, fonts.regular, false, false, false, false);

        let lbl_clock_skew = create_label(parent, "انحراف ساعت (ساعت)", 360, 180, 120, 18, 0, fonts.small);
        let txt_clock_skew = create_edit(parent, "24", 360, 200, 100, 26, ID_TXT_CLOCK_SKEW, fonts.regular, false, false, false, false);

        let lbl_min_ver = create_label(parent, "حداقل نسخه Daqiq", 20, 235, 150, 18, 0, fonts.small);
        let txt_min_ver = create_edit(parent, "4.9.0", 20, 255, 140, 26, ID_TXT_MIN_VERSION, fonts.regular, false, false, false, false);

        let lbl_max_ver = create_label(parent, "حداکثر نسخه (اختیاری)", 180, 235, 150, 18, 0, fonts.small);
        let txt_max_ver = create_edit(parent, "", 180, 255, 140, 26, ID_TXT_MAX_VERSION, fonts.regular, false, false, false, false);

        // Feature Entitlements
        let lbl_features_title = create_label(parent, "دسترسی به ماژول‌های Daqiq (Entitlements - PR #39)", 20, 290, 450, 20, 0, fonts.subheading);

        let feature_labels: [(&str, &str); 10] = [
            ("dashboard", "داشبورد مدیریت (dashboard)"),
            ("products", "مدیریت کالاها (products)"),
            ("sales", "فروش و فاکتور (sales)"),
            ("purchases", "خرید و سفارشات (purchases)"),
            ("inventory", "انبارداری (inventory)"),
            ("accounting", "حسابداری و مالی (accounting)"),
            ("reports", "گزارشات آماری (reports)"),
            ("settings", "تنظیمات نرم‌افزار (settings)"),
            ("users", "مدیریت کاربران (users)"),
            ("backup", "پشتیبان‌گیری ابری (backup)"),
        ];

        let mut chk_features = Vec::new();
        for (idx, (key, label)) in feature_labels.iter().enumerate() {
            let col = idx / 5;
            let row = idx % 5;
            let cx = if col == 0 { 20 } else { 250 };
            let cy = 315 + (row as i32) * 26;
            let hwnd = create_checkbox(parent, label, cx, cy, 210, 22, ID_CHK_FEATURE_BASE + idx as isize, fonts.regular, true);
            chk_features.push((key.to_string(), hwnd));
        }

        // Bottom Action Bar
        let lbl_status = create_label(parent, "وضعیت: آماده صدور لایسنس جدید", 20, 520, 400, 22, ID_LBL_STATUS, fonts.bold);
        let btn_generate_save = create_button(parent, "تولید و ذخیره لایسنس (.daqlic)  ✔", 20, 550, 250, 40, ID_BTN_GENERATE_SAVE, fonts.bold);
        let btn_copy_token = create_button(parent, "کپی توکن Base64", 280, 550, 150, 40, ID_BTN_COPY_TOKEN, fonts.regular);
        let btn_preview_json = create_button(parent, "پیش‌نمایش JSON", 440, 550, 140, 40, ID_BTN_PREVIEW_JSON, fonts.regular);

        Self {
            parent,
            lbl_header,
            btn_preset_trial,
            btn_preset_standard,
            btn_preset_pro,
            btn_preset_enterprise,
            btn_preset_perpetual,

            lbl_cust_name,
            txt_cust_name,
            lbl_lic_id,
            txt_lic_id,
            lbl_cust_org,
            txt_cust_org,
            lbl_cust_contact,
            txt_cust_contact,
            lbl_hwid,
            txt_hwid,
            lbl_max_devices,
            cb_max_devices,
            lbl_match_policy,
            cb_match_policy,

            lbl_lic_type,
            cb_lic_type,
            lbl_issued_date,
            txt_issued_date,
            lbl_not_before,
            txt_not_before,
            lbl_expires,
            txt_expires,

            lbl_grace_days,
            txt_grace_days,
            lbl_grace_mode,
            cb_grace_mode,
            lbl_warning_days,
            txt_warning_days,
            lbl_clock_skew,
            txt_clock_skew,

            lbl_min_ver,
            txt_min_ver,
            lbl_max_ver,
            txt_max_ver,

            lbl_features_title,
            chk_features,

            lbl_status,
            btn_generate_save,
            btn_copy_token,
            btn_preview_json,
        }
    }

    pub fn reposition(&self, total_w: i32, total_h: i32) {
        let col_w = (total_w - 60) / 2;
        let right_x = col_w + 30;
        let bottom_y = total_h - 70;

        unsafe {
            MoveWindow(self.lbl_header, 20, 10, total_w - 40, 24, 1);

            // Column 1 (Right)
            MoveWindow(self.lbl_cust_name, right_x, 75, col_w, 18, 1);
            MoveWindow(self.txt_cust_name, right_x, 95, col_w, 26, 1);
            MoveWindow(self.lbl_lic_id, right_x, 125, col_w, 18, 1);
            MoveWindow(self.txt_lic_id, right_x, 145, col_w, 26, 1);
            MoveWindow(self.lbl_cust_org, right_x, 175, col_w, 18, 1);
            MoveWindow(self.txt_cust_org, right_x, 195, col_w, 26, 1);
            MoveWindow(self.lbl_cust_contact, right_x, 225, col_w, 18, 1);
            MoveWindow(self.txt_cust_contact, right_x, 245, col_w, 26, 1);
            MoveWindow(self.lbl_hwid, right_x, 275, col_w, 18, 1);
            MoveWindow(self.txt_hwid, right_x, 295, col_w, 140, 1);

            let half_col = (col_w - 10) / 2;
            MoveWindow(self.lbl_max_devices, right_x, 445, half_col, 18, 1);
            MoveWindow(self.cb_max_devices, right_x, 465, half_col, 150, 1);
            MoveWindow(self.lbl_match_policy, right_x + half_col + 10, 445, half_col, 18, 1);
            MoveWindow(self.cb_match_policy, right_x + half_col + 10, 465, half_col, 150, 1);

            // Column 2 (Left)
            MoveWindow(self.lbl_lic_type, 20, 75, col_w, 18, 1);
            MoveWindow(self.cb_lic_type, 20, 95, col_w, 150, 1);

            let d_w = (col_w - 20) / 3;
            MoveWindow(self.lbl_issued_date, 20, 125, d_w, 18, 1);
            MoveWindow(self.txt_issued_date, 20, 145, d_w, 26, 1);
            MoveWindow(self.lbl_not_before, 20 + d_w + 10, 125, d_w, 18, 1);
            MoveWindow(self.txt_not_before, 20 + d_w + 10, 145, d_w, 26, 1);
            MoveWindow(self.lbl_expires, 20 + (d_w + 10) * 2, 125, d_w, 18, 1);
            MoveWindow(self.txt_expires, 20 + (d_w + 10) * 2, 145, d_w, 26, 1);

            let t_w = (col_w - 30) / 4;
            MoveWindow(self.lbl_grace_days, 20, 180, t_w, 18, 1);
            MoveWindow(self.txt_grace_days, 20, 200, t_w, 26, 1);
            MoveWindow(self.lbl_grace_mode, 20 + t_w + 10, 180, t_w, 18, 1);
            MoveWindow(self.cb_grace_mode, 20 + t_w + 10, 200, t_w, 120, 1);
            MoveWindow(self.lbl_warning_days, 20 + (t_w + 10) * 2, 180, t_w, 18, 1);
            MoveWindow(self.txt_warning_days, 20 + (t_w + 10) * 2, 200, t_w, 26, 1);
            MoveWindow(self.lbl_clock_skew, 20 + (t_w + 10) * 3, 180, t_w, 18, 1);
            MoveWindow(self.txt_clock_skew, 20 + (t_w + 10) * 3, 200, t_w, 26, 1);

            let v_w = (col_w - 10) / 2;
            MoveWindow(self.lbl_min_ver, 20, 235, v_w, 18, 1);
            MoveWindow(self.txt_min_ver, 20, 255, v_w, 26, 1);
            MoveWindow(self.lbl_max_ver, 20 + v_w + 10, 235, v_w, 18, 1);
            MoveWindow(self.txt_max_ver, 20 + v_w + 10, 255, v_w, 26, 1);

            MoveWindow(self.lbl_features_title, 20, 290, col_w, 20, 1);
            for (idx, (_, hwnd)) in self.chk_features.iter().enumerate() {
                let col = idx / 5;
                let row = idx % 5;
                let cx = if col == 0 { 20 } else { 20 + v_w };
                let cy = 315 + (row as i32) * 26;
                MoveWindow(*hwnd, cx, cy, v_w - 10, 22, 1);
            }

            // Bottom bar
            MoveWindow(self.lbl_status, 20, bottom_y - 28, total_w - 40, 22, 1);
            MoveWindow(self.btn_generate_save, 20, bottom_y, 250, 40, 1);
            MoveWindow(self.btn_copy_token, 280, bottom_y, 160, 40, 1);
            MoveWindow(self.btn_preview_json, 450, bottom_y, 140, 40, 1);
        }
    }

    pub fn show(&self, visible: bool) {
        let flag = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.lbl_header, flag);
            ShowWindow(self.btn_preset_trial, flag);
            ShowWindow(self.btn_preset_standard, flag);
            ShowWindow(self.btn_preset_pro, flag);
            ShowWindow(self.btn_preset_enterprise, flag);
            ShowWindow(self.btn_preset_perpetual, flag);

            ShowWindow(self.lbl_cust_name, flag);
            ShowWindow(self.txt_cust_name, flag);
            ShowWindow(self.lbl_lic_id, flag);
            ShowWindow(self.txt_lic_id, flag);
            ShowWindow(self.lbl_cust_org, flag);
            ShowWindow(self.txt_cust_org, flag);
            ShowWindow(self.lbl_cust_contact, flag);
            ShowWindow(self.txt_cust_contact, flag);
            ShowWindow(self.lbl_hwid, flag);
            ShowWindow(self.txt_hwid, flag);
            ShowWindow(self.lbl_max_devices, flag);
            ShowWindow(self.cb_max_devices, flag);
            ShowWindow(self.lbl_match_policy, flag);
            ShowWindow(self.cb_match_policy, flag);

            ShowWindow(self.lbl_lic_type, flag);
            ShowWindow(self.cb_lic_type, flag);
            ShowWindow(self.lbl_issued_date, flag);
            ShowWindow(self.txt_issued_date, flag);
            ShowWindow(self.lbl_not_before, flag);
            ShowWindow(self.txt_not_before, flag);
            ShowWindow(self.lbl_expires, flag);
            ShowWindow(self.txt_expires, flag);

            ShowWindow(self.lbl_grace_days, flag);
            ShowWindow(self.txt_grace_days, flag);
            ShowWindow(self.lbl_grace_mode, flag);
            ShowWindow(self.cb_grace_mode, flag);
            ShowWindow(self.lbl_warning_days, flag);
            ShowWindow(self.txt_warning_days, flag);
            ShowWindow(self.lbl_clock_skew, flag);
            ShowWindow(self.txt_clock_skew, flag);

            ShowWindow(self.lbl_min_ver, flag);
            ShowWindow(self.txt_min_ver, flag);
            ShowWindow(self.lbl_max_ver, flag);
            ShowWindow(self.txt_max_ver, flag);

            ShowWindow(self.lbl_features_title, flag);
            for (_, hwnd) in &self.chk_features {
                ShowWindow(*hwnd, flag);
            }

            ShowWindow(self.lbl_status, flag);
            ShowWindow(self.btn_generate_save, flag);
            ShowWindow(self.btn_copy_token, flag);
            ShowWindow(self.btn_preview_json, flag);
        }
    }

    pub fn update_from_state(&self, state: &AppState) {
        set_window_text(self.txt_cust_name, &state.customer_name);
        set_window_text(self.txt_lic_id, &state.license_id);
        set_window_text(self.txt_cust_org, &state.customer_org);
        set_window_text(self.txt_cust_contact, &state.customer_contact);
        set_window_text(self.txt_hwid, &state.hwid_text);

        let dev_idx = match state.max_devices {
            1 => 0,
            2 => 1,
            3 => 2,
            5 => 3,
            10 => 4,
            25 => 5,
            _ => 0,
        };
        set_combobox_selected_index(self.cb_max_devices, dev_idx);

        let policy_idx = match state.match_policy.as_str() {
            "strict" => 1,
            "relaxed" => 2,
            _ => 0,
        };
        set_combobox_selected_index(self.cb_match_policy, policy_idx);

        let type_idx = match state.license_type.as_str() {
            "perpetual" => 1,
            "trial" => 2,
            _ => 0,
        };
        set_combobox_selected_index(self.cb_lic_type, type_idx);

        set_window_text(self.txt_issued_date, &state.issued_jalali);
        set_window_text(self.txt_not_before, &state.not_before_jalali);
        set_window_text(self.txt_expires, &state.expires_jalali);

        set_window_text(self.txt_grace_days, &state.grace_period_days.to_string());
        set_combobox_selected_index(self.cb_grace_mode, if state.grace_mode == "full" { 1 } else { 0 });
        set_window_text(self.txt_warning_days, &state.expiry_warning_days.to_string());
        set_window_text(self.txt_clock_skew, &state.max_clock_skew_hours.to_string());

        set_window_text(self.txt_min_ver, &state.min_app_version);
        set_window_text(self.txt_max_ver, &state.max_app_version);

        for (k, hwnd) in &self.chk_features {
            let checked = state.features.get(k).copied().unwrap_or(false);
            set_checkbox_checked(*hwnd, checked);
        }

        if let Some(err) = &state.generation_error {
            set_window_text(self.lbl_status, &format!("خطا: {err}"));
        } else if state.generated_envelope.is_some() {
            set_window_text(self.lbl_status, "لایسنس با موفقیت تولید شد و معتبر است ✓");
        } else {
            set_window_text(self.lbl_status, "وضعیت: آماده صدور لایسنس جدید");
        }
    }

    pub fn update_to_state(&self, state: &mut AppState) {
        state.customer_name = get_window_text(self.txt_cust_name);
        state.license_id = get_window_text(self.txt_lic_id);
        state.customer_org = get_window_text(self.txt_cust_org);
        state.customer_contact = get_window_text(self.txt_cust_contact);
        state.hwid_text = get_window_text(self.txt_hwid);

        let dev_idx = get_combobox_selected_index(self.cb_max_devices);
        state.max_devices = match dev_idx {
            1 => 2,
            2 => 3,
            3 => 5,
            4 => 10,
            5 => 25,
            _ => 1,
        };

        let policy_idx = get_combobox_selected_index(self.cb_match_policy);
        state.match_policy = match policy_idx {
            1 => "strict".to_string(),
            2 => "relaxed".to_string(),
            _ => "standard".to_string(),
        };

        let type_idx = get_combobox_selected_index(self.cb_lic_type);
        state.license_type = match type_idx {
            1 => "perpetual".to_string(),
            2 => "trial".to_string(),
            _ => "subscription".to_string(),
        };

        state.issued_jalali = get_window_text(self.txt_issued_date);
        state.not_before_jalali = get_window_text(self.txt_not_before);
        state.expires_jalali = get_window_text(self.txt_expires);

        state.grace_period_days = get_window_text(self.txt_grace_days).parse().unwrap_or(7);
        state.grace_mode = if get_combobox_selected_index(self.cb_grace_mode) == 1 {
            "full".to_string()
        } else {
            "readonly".to_string()
        };
        state.expiry_warning_days = get_window_text(self.txt_warning_days).parse().unwrap_or(14);
        state.max_clock_skew_hours = get_window_text(self.txt_clock_skew).parse().unwrap_or(24);

        state.min_app_version = get_window_text(self.txt_min_ver);
        state.max_app_version = get_window_text(self.txt_max_ver);

        for (k, hwnd) in &self.chk_features {
            let checked = get_checkbox_checked(*hwnd);
            state.features.insert(k.clone(), checked);
        }
    }
}
