use chrono::{Datelike, Utc};
use daqiq_lic_core::gregorian_to_jalali;

#[allow(dead_code)]
pub const PERSIAN_MONTHS: [&str; 12] = [
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
];

pub fn current_jalali_today() -> (i32, u32, u32) {
    let now = Utc::now();
    gregorian_to_jalali(now.year(), now.month(), now.day()).unwrap_or((1405, 9, 1))
}

pub fn format_jalali_date(y: i32, m: u32, d: u32) -> String {
    format!("{:04}/{:02}/{:02}", y, m, d)
}

pub fn parse_jalali_date(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = s.trim().split(&['/', '-'][..]).collect();
    if parts.len() == 3 {
        let y = parts[0].trim().parse::<i32>().ok()?;
        let m = parts[1].trim().parse::<u32>().ok()?;
        let d = parts[2].trim().parse::<u32>().ok()?;
        if (1..=12).contains(&m) && (1..=31).contains(&d) {
            return Some((y, m, d));
        }
    }
    None
}

pub fn add_jalali_months(y: i32, m: u32, d: u32, months: i32) -> (i32, u32, u32) {
    let total_months = (y * 12 + (m as i32 - 1)) + months;
    let new_y = total_months / 12;
    let new_m = (total_months % 12 + 1) as u32;
    let max_d = if new_m <= 6 {
        31
    } else if new_m <= 11 {
        30
    } else {
        29
    };
    let new_d = d.min(max_d);
    (new_y, new_m, new_d)
}

#[allow(dead_code)]
pub fn jalali_to_utc_iso_string(jalali_str: &str, end_of_day: bool) -> Result<String, String> {
    daqiq_lic_core::jalali_str_to_utc_iso(jalali_str, end_of_day)
        .map_err(|e| format!("تاریخ شمسی نامعتبر: {e}"))
}

pub fn utc_iso_to_jalali_display(iso_str: &str) -> String {
    daqiq_lic_core::utc_iso_to_jalali_str(iso_str).unwrap_or_else(|_| iso_str.to_string())
}
