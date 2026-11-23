use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{MoveWindow, ShowWindow, SW_HIDE, SW_SHOW};

use crate::state::AppState;
use crate::win32_util::{
    add_combobox_item, add_listview_column, add_listview_row, clear_listview, create_button,
    create_combobox, create_edit, create_label, create_listview, get_combobox_selected_index,
    set_combobox_selected_index, set_window_text, Fonts,
};

pub const ID_TXT_SEARCH: isize = 1201;
pub const ID_CB_STATUS_FILTER: isize = 1202;
pub const ID_LISTVIEW: isize = 1203;
pub const ID_BTN_RENEW: isize = 1204;
pub const ID_BTN_EXPORT_CSV: isize = 1205;
pub const ID_BTN_RESAVE: isize = 1206;

pub struct LedgerView {
    pub parent: HWND,
    pub lbl_header: HWND,
    pub lbl_kpi_total: HWND,
    pub lbl_kpi_active: HWND,
    pub lbl_kpi_expiring: HWND,
    pub lbl_kpi_expired: HWND,

    pub lbl_search: HWND,
    pub txt_search: HWND,
    pub lbl_filter: HWND,
    pub cb_filter: HWND,

    pub listview: HWND,

    pub btn_renew: HWND,
    pub btn_resave: HWND,
    pub btn_export_csv: HWND,
}

impl LedgerView {
    pub fn create(parent: HWND, fonts: &Fonts) -> Self {
        let lbl_header = create_label(
            parent,
            "دفتر کل و تاریخچه لایسنس‌های صادر شده",
            20, 10, 600, 24,
            1200,
            fonts.heading,
        );

        // KPI Pills
        let lbl_kpi_total = create_label(parent, "کل لایسنس‌ها: ۰", 20, 42, 140, 26, 0, fonts.bold);
        let lbl_kpi_active = create_label(parent, "فعال: ۰", 170, 42, 110, 26, 0, fonts.bold);
        let lbl_kpi_expiring = create_label(parent, "در آستانه انقضا: ۰", 290, 42, 150, 26, 0, fonts.bold);
        let lbl_kpi_expired = create_label(parent, "منقضی شده: ۰", 450, 42, 130, 26, 0, fonts.bold);

        // Filter Bar
        let lbl_search = create_label(parent, "جستجو:", 20, 78, 60, 20, 0, fonts.small);
        let txt_search = create_edit(parent, "", 85, 76, 260, 24, ID_TXT_SEARCH, fonts.regular, true, false, false, false);

        let lbl_filter = create_label(parent, "فیلتر:", 360, 78, 50, 20, 0, fonts.small);
        let cb_filter = create_combobox(parent, 415, 76, 150, 120, ID_CB_STATUS_FILTER, fonts.regular);
        add_combobox_item(cb_filter, "همه لایسنس‌ها");
        add_combobox_item(cb_filter, "فقط فعال");
        add_combobox_item(cb_filter, "فقط منقضی شده");
        set_combobox_selected_index(cb_filter, 0);

        // Table
        let listview = create_listview(parent, 20, 110, 760, 420, ID_LISTVIEW, fonts.regular);
        add_listview_column(listview, 0, 170, "شناسه لایسنس");
        add_listview_column(listview, 1, 160, "نام مشتری");
        add_listview_column(listview, 2, 90, "نوع");
        add_listview_column(listview, 3, 110, "شروع اعتبار");
        add_listview_column(listview, 4, 110, "تاریخ انقضا");
        add_listview_column(listview, 5, 80, "دستگاه");
        add_listview_column(listview, 6, 130, "تاریخ صدور");

        // Action Buttons
        let btn_renew = create_button(parent, "تمدید سریع (Clone & Renew)  ▶", 20, 545, 230, 38, ID_BTN_RENEW, fonts.bold);
        let btn_resave = create_button(parent, "ذخیره مجدد فایل (.daqlic)", 260, 545, 190, 38, ID_BTN_RESAVE, fonts.regular);
        let btn_export_csv = create_button(parent, "خروجی اکسل (CSV UTF-8 BOM)", 460, 545, 210, 38, ID_BTN_EXPORT_CSV, fonts.regular);

        Self {
            parent,
            lbl_header,
            lbl_kpi_total,
            lbl_kpi_active,
            lbl_kpi_expiring,
            lbl_kpi_expired,
            lbl_search,
            txt_search,
            lbl_filter,
            cb_filter,
            listview,
            btn_renew,
            btn_resave,
            btn_export_csv,
        }
    }

    pub fn reposition(&self, total_w: i32, total_h: i32) {
        let bottom_y = total_h - 60;
        let table_h = bottom_y - 125;
        unsafe {
            MoveWindow(self.lbl_header, 20, 10, total_w - 40, 24, 1);
            MoveWindow(self.listview, 20, 110, total_w - 40, table_h, 1);

            MoveWindow(self.btn_renew, 20, bottom_y, 230, 38, 1);
            MoveWindow(self.btn_resave, 260, bottom_y, 190, 38, 1);
            MoveWindow(self.btn_export_csv, 460, bottom_y, 220, 38, 1);
        }
    }

    pub fn show(&self, visible: bool) {
        let flag = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.lbl_header, flag);
            ShowWindow(self.lbl_kpi_total, flag);
            ShowWindow(self.lbl_kpi_active, flag);
            ShowWindow(self.lbl_kpi_expiring, flag);
            ShowWindow(self.lbl_kpi_expired, flag);
            ShowWindow(self.lbl_search, flag);
            ShowWindow(self.txt_search, flag);
            ShowWindow(self.lbl_filter, flag);
            ShowWindow(self.cb_filter, flag);
            ShowWindow(self.listview, flag);
            ShowWindow(self.btn_renew, flag);
            ShowWindow(self.btn_resave, flag);
            ShowWindow(self.btn_export_csv, flag);
        }
    }

    pub fn update_from_state(&self, state: &AppState) {
        let total = state.records.len();
        let active = state.records.iter().filter(|r| r.expires_at.is_none() || r.expires_at.as_deref().unwrap_or("") > "2026-11-23").count();
        let expired = total.saturating_sub(active);

        set_window_text(self.lbl_kpi_total, &format!("کل لایسنس‌ها: {total}"));
        set_window_text(self.lbl_kpi_active, &format!("فعال: {active}"));
        set_window_text(self.lbl_kpi_expiring, "در آستانه انقضا: ۰");
        set_window_text(self.lbl_kpi_expired, &format!("منقضی شده: {expired}"));

        clear_listview(self.listview);

        let query = state.search_query.to_lowercase();
        let _filter_idx = get_combobox_selected_index(self.cb_filter);

        for (idx, r) in state.records.iter().enumerate() {
            if !query.is_empty() {
                let match_name = r.customer_name.to_lowercase().contains(&query);
                let match_lic = r.license_id.to_lowercase().contains(&query);
                let match_contact = r.customer_contact.as_deref().unwrap_or("").to_lowercase().contains(&query);
                if !match_name && !match_lic && !match_contact {
                    continue;
                }
            }

            let not_before_display = crate::rtl::utc_iso_to_jalali_display(&r.not_before);
            let expires_display = if let Some(exp) = &r.expires_at {
                crate::rtl::utc_iso_to_jalali_display(exp)
            } else {
                "نامحدود (دائمی)".to_string()
            };
            let issued_display = crate::rtl::utc_iso_to_jalali_display(&r.issued_at);
            let dev_str = r.max_devices.to_string();

            add_listview_row(
                self.listview,
                idx as i32,
                &[
                    &r.license_id,
                    &r.customer_name,
                    &r.license_type,
                    &not_before_display,
                    &expires_display,
                    &dev_str,
                    &issued_display,
                ],
            );
        }
    }
}
