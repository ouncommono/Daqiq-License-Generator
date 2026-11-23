use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use crate::error::LicError;

const G_DAYS_IN_MONTH: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const J_DAYS_IN_MONTH: [i32; 12] = [31, 31, 31, 31, 31, 31, 30, 30, 30, 30, 30, 29];

/// Check if a Jalali year is leap year
pub fn is_jalali_leap(jy: i32) -> bool {
    let breaks = [1, 5, 9, 13, 17, 22, 26, 30];
    let rem = ((jy % 33) + 33) % 33;
    breaks.contains(&rem)
}

/// Convert Gregorian (year, month 1..=12, day 1..=31) to Jalali (jy, jm, jd)
pub fn gregorian_to_jalali(gyear: i32, gmonth: u32, gday: u32) -> Result<(i32, u32, u32), LicError> {
    if gmonth < 1 || gmonth > 12 || gday < 1 || gday > 31 {
        return Err(LicError::Date(format!("Invalid Gregorian date: {gyear}-{gmonth}-{gday}")));
    }

    let gy = gyear - 1600;
    let gm = (gmonth - 1) as usize;
    let mut j_day_no = 365 * gy + (gy + 3) / 4 - (gy + 99) / 100
        + (gy + 399) / 400 + (gday as i32) - 1 - 79;

    for i in 0..gm {
        j_day_no += G_DAYS_IN_MONTH[i];
    }
    if gm > 1 && ((gy % 4 == 0 && gy % 100 != 0) || (gy % 400 == 0)) {
        j_day_no += 1;
    }

    let j_np = j_day_no / 12053;
    j_day_no %= 12053;
    let mut jy = 979 + 33 * j_np + 4 * (j_day_no / 1461);
    j_day_no %= 1461;

    if j_day_no >= 366 {
        j_day_no -= 1;
        jy += j_day_no / 365;
        j_day_no %= 365;
    }

    let mut jm = 0;
    for i in 0..11 {
        if j_day_no < J_DAYS_IN_MONTH[i] {
            break;
        }
        j_day_no -= J_DAYS_IN_MONTH[i];
        jm = i + 1;
    }

    Ok((jy, (jm + 1) as u32, (j_day_no + 1) as u32))
}

/// Convert Jalali (year, month 1..=12, day 1..=31) to Gregorian (gy, gm, gd)
pub fn jalali_to_gregorian(jyear: i32, jmonth: u32, jday: u32) -> Result<(i32, u32, u32), LicError> {
    if jmonth < 1 || jmonth > 12 || jday < 1 || jday > 31 {
        return Err(LicError::Date(format!("Invalid Jalali date: {jyear}/{jmonth}/{jday}")));
    }

    let jy = jyear - 979;
    let jm = (jmonth - 1) as usize;
    let mut g_day_no = 365 * jy + (jy / 33) * 8 + (jy % 33 + 3) / 4 + (jday as i32) - 1 + 79;

    for i in 0..jm {
        g_day_no += J_DAYS_IN_MONTH[i];
    }

    let mut gy = 1600 + 400 * (g_day_no / 146097);
    g_day_no %= 146097;

    let mut leap = 1;
    if g_day_no >= 36525 {
        g_day_no -= 1;
        gy += 100 * (g_day_no / 36524);
        g_day_no %= 36524;

        if g_day_no >= 365 {
            g_day_no += 1;
        } else {
            leap = 0;
        }
    }

    gy += 4 * (g_day_no / 1461);
    g_day_no %= 1461;

    if g_day_no >= 366 {
        leap = 0;
        g_day_no -= 1;
        gy += g_day_no / 365;
        g_day_no %= 365;
    }

    let mut i = 0usize;
    while i < 11 && g_day_no >= (G_DAYS_IN_MONTH[i] + if i == 1 && leap == 1 { 1 } else { 0 }) {
        g_day_no -= G_DAYS_IN_MONTH[i] + if i == 1 && leap == 1 { 1 } else { 0 };
        i += 1;
    }

    let gmonth = (i + 1) as u32;
    let gday = (g_day_no + 1) as u32;

    Ok((gy, gmonth, gday))
}

/// Parses a Jalali string `YYYY/MM/DD` and converts to start-of-day UTC ISO timestamp `YYYY-MM-DDTHH:MM:SSZ`
pub fn jalali_str_to_utc_iso(jalali_str: &str, end_of_day: bool) -> Result<String, LicError> {
    let parts: Vec<&str> = jalali_str.split('/').collect();
    if parts.len() != 3 {
        return Err(LicError::Date(format!(
            "فرمت تاریخ شمسی نادرست است (باید YYYY/MM/DD باشد): {jalali_str}"
        )));
    }

    let jy: i32 = parts[0]
        .trim()
        .parse()
        .map_err(|_| LicError::Date("سال شمسی نامعتبر است".to_string()))?;
    let jm: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|_| LicError::Date("ماه شمسی نامعتبر است".to_string()))?;
    let jd: u32 = parts[2]
        .trim()
        .parse()
        .map_err(|_| LicError::Date("روز شمسی نامعتبر است".to_string()))?;

    let (gy, gm, gd) = jalali_to_gregorian(jy, jm, jd)?;
    let date = NaiveDate::from_ymd_opt(gy, gm, gd)
        .ok_or_else(|| LicError::Date("تاریخ معادل میلادی نامعتبر است".to_string()))?;

    let dt = if end_of_day {
        date.and_hms_opt(23, 59, 59).unwrap()
    } else {
        date.and_hms_opt(0, 0, 0).unwrap()
    };

    let utc_dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
    Ok(utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

/// Converts a Gregorian or UTC ISO-8601 string to Jalali `YYYY/MM/DD`
pub fn utc_iso_to_jalali_str(iso_str: &str) -> Result<String, LicError> {
    let clean = iso_str.trim();
    let dt = DateTime::parse_from_rfc3339(clean)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            let naive = chrono::NaiveDateTime::parse_from_str(clean, "%Y-%m-%dT%H:%M:%SZ")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(clean, "%Y-%m-%d %H:%M:%S"))
                .map_err(|e| LicError::Date(format!("خطا در تجزیه تاریخ ISO: {e}")))?;
            Ok::<DateTime<Utc>, LicError>(Utc.from_utc_datetime(&naive))
        })?;

    let (jy, jm, jd) = gregorian_to_jalali(dt.year(), dt.month(), dt.day())?;
    Ok(format!("{jy:04}/{jm:02}/{jd:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_date_conversions() {
        // 2026-11-22 -> 1405-09-01
        let (jy, jm, jd) = gregorian_to_jalali(2026, 11, 22).unwrap();
        assert_eq!((jy, jm, jd), (1405, 9, 1));

        let (gy, gm, gd) = jalali_to_gregorian(1405, 9, 1).unwrap();
        assert_eq!((gy, gm, gd), (2026, 11, 22));

        // 2027-03-21 -> 1406-01-01 (Nowruz)
        let (jy2, jm2, jd2) = gregorian_to_jalali(2027, 3, 21).unwrap();
        assert_eq!((jy2, jm2, jd2), (1406, 1, 1));

        let (gy2, gm2, gd2) = jalali_to_gregorian(1406, 1, 1).unwrap();
        assert_eq!((gy2, gm2, gd2), (2027, 3, 21));
    }

    #[test]
    fn test_iso_formatting() {
        let iso = jalali_str_to_utc_iso("1405/09/01", false).unwrap();
        assert_eq!(iso, "2026-11-22T00:00:00Z");

        let j_str = utc_iso_to_jalali_str("2026-11-22T00:00:00Z").unwrap();
        assert_eq!(j_str, "1405/09/01");
    }
}
