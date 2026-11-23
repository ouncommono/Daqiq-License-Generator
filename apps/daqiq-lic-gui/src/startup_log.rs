use std::fs;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{GetLastError, HWND};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

use crate::win32_util::to_wstring;

pub fn get_app_data_dir() -> PathBuf {
    if let Some(mut p) = dirs::data_local_dir() {
        p.push("DaqiqLicenseGenerator");
        let _ = fs::create_dir_all(&p);
        p
    } else {
        let p = std::env::temp_dir().join("DaqiqLicenseGenerator");
        let _ = fs::create_dir_all(&p);
        p
    }
}

pub fn init_log() -> PathBuf {
    let dir = get_app_data_dir();
    let log_path = dir.join("startup.log");
    let now = chrono::Utc::now().to_rfc3339();
    let header = format!("[{now}] [BOOT_START] Daqiq License Studio v1.4.2 starting...\n");
    let _ = fs::write(&log_path, header);
    log_path
}

pub fn log_stage(stage: &str, details: &str) {
    let dir = get_app_data_dir();
    let log_path = dir.join("startup.log");
    let now = chrono::Utc::now().to_rfc3339();
    let line = format!("[{now}] [{stage}] {details}\n");
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        use std::io::Write;
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

pub fn log_fatal(stage: &str, error: &str) {
    let err_code = unsafe { GetLastError() };
    let dir = get_app_data_dir();
    let log_path = dir.join("startup.log");
    let now = chrono::Utc::now().to_rfc3339();
    let line = format!("[{now}] [FATAL] Stage '{stage}' failed: {error} (GetLastError: {err_code})\n");
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        use std::io::Write;
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

pub fn show_fatal_dialog(stage: &str, technical_details: &str) {
    let err_code = unsafe { GetLastError() };
    log_fatal(stage, technical_details);
    let log_path = get_app_data_dir().join("startup.log");

    let title_w = to_wstring("خطای بحرانی در راه‌اندازی برنامه (Daqiq License Studio)");
    let body = format!(
        "متأسفانه برنامه در مرحله راه‌اندازی با خطا مواجه شد:\n\n\
        • مرحله: {stage}\n\
        • جزئیات فنی: {technical_details}\n\
        • کد خطای ویندوز (GetLastError): {err_code}\n\
        • مسیر فایل گزارش: {}\n\n\
        لطفاً فایل فوق را بررسی کرده یا به پشتیبانی فنی ارائه دهید.",
        log_path.display()
    );
    let body_w = to_wstring(&body);

    unsafe {
        MessageBoxW(0 as HWND, body_w.as_ptr(), title_w.as_ptr(), MB_OK | MB_ICONERROR);
    }
}
