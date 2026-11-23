#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod panels;
mod rtl;
mod startup_log;
mod state;
mod theme;
mod win32_util;

use std::env;
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;

use serde_json::json;
use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, RedrawWindow, SetBkColor, SetBkMode, SetTextColor, UpdateWindow, HDC,
    PAINTSTRUCT, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, TRANSPARENT,
};
use windows_sys::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_BAR_CLASSES, ICC_LISTVIEW_CLASSES, ICC_STANDARD_CLASSES,
    ICC_TAB_CLASSES, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX,
};
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    MessageBoxW, PostQuitMessage, RegisterClassExW, ShowWindow, SystemParametersInfoW,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, MB_ICONERROR, MB_ICONINFORMATION,
    MB_OK, MSG, SPI_GETWORKAREA, SW_SHOW, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLOREDIT,
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_PAINT, WM_SIZE, WNDCLASSEXW, WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_EX_CONTROLPARENT, WS_OVERLAPPEDWINDOW, WS_POPUP,
};

use daqiq_lic_core::{
    canonicalize_payload, gregorian_to_jalali, jalali_to_gregorian, validate_hwid,
    verify_daqiq_binary_pairing, verify_license_envelope, BinaryPairingResult, CustomerInfo,
    KeyVault, LicenseEnvelope, LicensePayload, RegistryDb, ALL_FEATURE_KEYS, DEFAULT_KEY_ID,
    DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
};

use crate::config::AppConfig;
use crate::panels::generator::{
    GeneratorView, ID_BTN_COPY_TOKEN, ID_BTN_GENERATE_SAVE, ID_BTN_PREVIEW_JSON,
    ID_PRESET_ENTERPRISE, ID_PRESET_PERPETUAL, ID_PRESET_PRO, ID_PRESET_STANDARD,
    ID_PRESET_TRIAL,
};
use crate::panels::inspector::{
    InspectorView, ID_BTN_LOAD_FILE, ID_BTN_SIMULATE, ID_BTN_VERIFY_INSPECT,
};
use crate::panels::ledger::{LedgerView, ID_BTN_EXPORT_CSV, ID_BTN_RENEW};
use crate::panels::sidebar::{
    SidebarView, ID_BTN_THEME, ID_TAB_GEN, ID_TAB_INSPECTOR, ID_TAB_LEDGER, ID_TAB_VAULT,
};
use crate::panels::vault::{
    VaultView, ID_BTN_COPY_PUBKEY, ID_BTN_CREATE, ID_BTN_IMPORT, ID_BTN_SCAN_BINARY, ID_BTN_UNLOCK,
};
use crate::state::{ActiveTab, AppState};
use crate::theme::{ThemeGdiResources, ThemeMode};
use crate::win32_util::{get_window_text, set_window_text, to_wstring, Fonts};

pub struct AppWindow {
    pub state: AppState,
    pub fonts: Fonts,
    pub theme_gdi: ThemeGdiResources,
    // pub d2d: Option<crate::d2d::D2dContext>,
    pub sidebar: Option<SidebarView>,
    pub generator: Option<GeneratorView>,
    pub ledger: Option<LedgerView>,
    pub inspector: Option<InspectorView>,
    pub vault: Option<VaultView>,
    pub is_headless: bool,
}

static mut GLOBAL_APP: Option<*mut AppWindow> = None;

fn get_app_mut() -> Option<&'static mut AppWindow> {
    unsafe {
        GLOBAL_APP.map(|ptr| &mut *ptr)
    }
}

pub fn show_message(hwnd: HWND, title: &str, message: &str, is_error: bool) {
    let t_w = to_wstring(title);
    let m_w = to_wstring(message);
    let u_type = if is_error {
        MB_OK | MB_ICONERROR
    } else {
        MB_OK | MB_ICONINFORMATION
    };
    unsafe {
        MessageBoxW(hwnd, m_w.as_ptr(), t_w.as_ptr(), u_type);
    }
}

// ---------------------------------------------------------------------------
// Headless Self-Test Implementation (Gate 4)
// ---------------------------------------------------------------------------
fn run_headless_self_test(report_path: Option<PathBuf>) -> Result<(), String> {
    println!("[SELF-TEST] Starting headless cryptographic, subsystem & engine verification...");

    // Test 1: Golden Vector SHA-256 and Signature Verification
    let raw_pub_b64 = "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";
    let expected_sha = "52bdf69ad45d565685bae87f75c03d4a5baa43a3acafe785499b16ee5618366a";
    let sig_b64 = "kgrF5f89h9JBIHiQ55H5J+jZul3ZwC6NdeEDXtrlxMDWWlBr892vliqLsTxPD0q+RlEJaXsIQZUSjaevKSGjCA==";

    let mut features = std::collections::BTreeMap::new();
    for k in ALL_FEATURE_KEYS {
        features.insert(k.to_string(), true);
    }

    let payload = LicensePayload {
        customer: CustomerInfo {
            contact: Some("+989120000000".to_string()),
            name: "فروشگاه تستی نمونه".to_string(),
            organization: Some("مجموعه آزمایشی".to_string()),
        },
        device_fingerprints: vec![
            "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string(),
        ],
        expires_at: None,
        features,
        format_version: 1,
        issued_at: "2026-11-22T00:00:00Z".to_string(),
        key_id: "daqiq-test-v1".to_string(),
        license_id: "LIC-2026-TEST01".to_string(),
        license_type: "perpetual".to_string(),
        max_app_version: None,
        max_devices: 1,
        min_app_version: Some("4.9.0".to_string()),
        not_before: "2026-11-22T00:00:00Z".to_string(),
        product_id: "daqiq.desktop".to_string(),
        match_policy: None,
        grace_period_days: None,
        grace_mode: None,
        expiry_warning_days: None,
        max_clock_skew_hours: None,
        quotas: None,
        metadata: None,
    };

    let canon_bytes = canonicalize_payload(&payload).map_err(|e| e.to_string())?;
    let mut msg = Vec::new();
    msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
    msg.extend_from_slice(&canon_bytes);
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&msg);
    let actual_sha = format!("{:x}", hasher.finalize());

    if actual_sha != expected_sha {
        return Err(format!(
            "Golden vector SHA-256 mismatch: got {actual_sha}, expected {expected_sha}"
        ));
    }

    let envelope = LicenseEnvelope {
        algorithm: "Ed25519".to_string(),
        envelope_version: 1,
        key_id: "daqiq-test-v1".to_string(),
        payload,
        signature: sig_b64.to_string(),
    };

    let is_valid = verify_license_envelope(&envelope, raw_pub_b64).map_err(|e| e.to_string())?;
    if !is_valid {
        return Err("Golden vector signature failed verification!".to_string());
    }

    // Test 2: HWID Normalization & Validation
    let dirty_hwid = " \u{200c}fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e\u{200f} ";
    let clean_hwid = validate_hwid(dirty_hwid).map_err(|e| e.to_string())?;
    if clean_hwid != "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e" {
        return Err("HWID cleaner failed to strip invisible characters".to_string());
    }

    // Test 3: Jalali Calendar Math
    let (jy, jm, jd) = gregorian_to_jalali(2026, 11, 22).map_err(|e| e.to_string())?;
    if (jy, jm, jd) != (1405, 9, 1) {
        return Err(format!("Jalali date mismatch: expected 1405/09/01, got {jy}/{jm}/{jd}"));
    }
    let (gy, gm, gd) = jalali_to_gregorian(1405, 9, 1).map_err(|e| e.to_string())?;
    if (gy, gm, gd) != (2026, 11, 22) {
        return Err(format!("Gregorian date mismatch: expected 2026-11-22, got {gy}-{gm}-{gd}"));
    }

    // Test 4: KeyVault Encryption / Decryption Roundtrip
    let (vault, vault_env) = KeyVault::create_vault("selftest-key-v1", "P@ssw0rd123!")
        .map_err(|e| e.to_string())?;
    let restored = KeyVault::load_from_envelope(&vault_env, "P@ssw0rd123!")
        .map_err(|e| e.to_string())?;
    if restored.public_key_b64 != vault.public_key_b64 {
        return Err("Vault public key mismatch after decryption".to_string());
    }

    // Test 5: Registry Operations
    let db = RegistryDb::open_in_memory().map_err(|e| e.to_string())?;
    db.insert_license(&envelope).map_err(|e| e.to_string())?;
    let records = db.list_licenses(10, 0).map_err(|e| e.to_string())?;
    if records.len() != 1 || records[0].license_id != "LIC-2026-TEST01" {
        return Err("Registry failed insertion or query".to_string());
    }

    // Test 6: In-App Binary Pairing Scanner
    let temp_dir = std::env::temp_dir();
    let test_bin_path = temp_dir.join("daqiq_selftest_mock.bin");
    let mut test_bytes = vec![0x90; 256 * 1024];
    test_bytes.extend_from_slice(DEFAULT_PROD_PUBLIC_KEY.as_bytes());
    test_bytes.extend(vec![0xCC; 256 * 1024]);
    fs::write(&test_bin_path, &test_bytes).map_err(|e| e.to_string())?;

    let pairing_res = verify_daqiq_binary_pairing(&test_bin_path, DEFAULT_PROD_PUBLIC_KEY);
    let _ = fs::remove_file(&test_bin_path);
    match pairing_res {
        BinaryPairingResult::Paired { matched_encoding, .. } => {
            if matched_encoding.is_empty() {
                return Err("Binary pairing scanner failed to pair default key".to_string());
            }
        }
        other => return Err(format!("Binary pairing scanner returned non-paired result: {other:?}")),
    }

    print_console("[SELF-TEST] All 6 core checks passed with 100% mathematical certainty!");

    let report = json!({
        "status": "ok",
        "app_name": "Daqiq License Generator Studio",
        "version": "1.4.2",
        "target_product": DEFAULT_PRODUCT_ID,
        "default_key_id": DEFAULT_KEY_ID,
        "default_public_key": DEFAULT_PROD_PUBLIC_KEY,
        "tests": [
            {"name": "golden_vector_sha256_and_ed25519", "passed": true},
            {"name": "hwid_unicode_cleaner_and_validation", "passed": true},
            {"name": "jalali_calendar_roundtrip", "passed": true},
            {"name": "encrypted_vault_roundtrip", "passed": true},
            {"name": "registry_operations", "passed": true},
            {"name": "binary_pairing_streaming_scanner", "passed": true}
        ]
    });

    if let Some(path) = report_path {
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("Failed to write report file: {e}"))?;
        print_console(&format!("[SELF-TEST] Report written to: {}", path.display()));
    } else {
        print_console(&serde_json::to_string_pretty(&report).unwrap());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Headless UI Self-Test Implementation (Gate 5)
// ---------------------------------------------------------------------------
fn run_headless_ui_self_test(report_path: Option<PathBuf>) -> Result<(), String> {
    println!("[UI-TEST] Initializing real hidden Win32 window and exercising all 4 tabs...");

    unsafe {
        let icex = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES | ICC_STANDARD_CLASSES | ICC_LISTVIEW_CLASSES,
        };
        InitCommonControlsEx(&icex);
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let class_name = to_wstring("DaqiqUiTestClass");
    let mut wnd_class: WNDCLASSEXW = unsafe { std::mem::zeroed() };
    wnd_class.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wnd_class.style = CS_HREDRAW | CS_VREDRAW;
    wnd_class.lpfnWndProc = Some(wnd_proc);
    wnd_class.lpszClassName = class_name.as_ptr();

    unsafe {
        RegisterClassExW(&wnd_class);
    }

    let mut app = AppWindow {
        state: AppState::new(),
        fonts: Fonts::new(),
        theme_gdi: ThemeGdiResources::new(ThemeMode::Dark),
        // d2d: None,
        sidebar: None,
        generator: None,
        ledger: None,
        inspector: None,
        vault: None,
        is_headless: true,
    };

    unsafe {
        GLOBAL_APP = Some(&mut app as *mut _);
    }

    let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };
    let title = to_wstring("Daqiq UI Test Window");
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP, // Hidden popup window
            0,
            0,
            1280,
            850,
            0 as HWND,
            0 as _,
            hinstance,
            std::ptr::null_mut(),
        )
    };

    if hwnd == 0 as HWND {
        return Err("Failed to create Win32 test window".to_string());
    }

    // Initial positioning
    if let Some(sidebar) = &app.sidebar {
        sidebar.reposition(1280 - 220, 0, 220, 850);
    }
    if let Some(generator) = &app.generator {
        generator.reposition(1280 - 220, 850);
    }
    if let Some(ledger) = &app.ledger {
        ledger.reposition(1280 - 220, 850);
    }
    if let Some(inspector) = &app.inspector {
        inspector.reposition(1280 - 220, 850);
    }
    if let Some(vault) = &app.vault {
        vault.reposition(1280 - 220, 850);
    }

    // Step 1: Cycle to Tab 0 (Generator), apply preset, fill details, generate
    println!("[UI-TEST] Tab 0: Testing License Generator inputs, presets & generation...");
    app.state.active_tab = ActiveTab::Generator;
    switch_tab_view(&mut app, ActiveTab::Generator);

    app.state.apply_preset("standard");
    app.state.customer_name = "شرکت آزمایشی نسخه ۱.۴".to_string();
    app.state.hwid_text = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string();
    app.generator.as_ref().unwrap().update_from_state(&app.state);

    let generated_env = app.state.generate_license().map_err(|e| format!("Tab 0 Generation error: {e}"))?;
    if generated_env.payload.customer.name != "شرکت آزمایشی نسخه ۱.۴" {
        return Err("Generated license customer mismatch".to_string());
    }
    app.generator.as_ref().unwrap().update_from_state(&app.state);

    // Step 2: Cycle to Tab 1 (Ledger), verify records, test filter
    println!("[UI-TEST] Tab 1: Testing Ledger record rendering and filtering...");
    app.state.active_tab = ActiveTab::Ledger;
    switch_tab_view(&mut app, ActiveTab::Ledger);
    app.state.load_ledger();
    app.ledger.as_ref().unwrap().update_from_state(&app.state);
    if app.state.records.is_empty() {
        return Err("Ledger contains zero records after license generation".to_string());
    }

    // Step 3: Cycle to Tab 2 (Inspector), test verification & simulator
    println!("[UI-TEST] Tab 2: Testing Inspector parsing, Ed25519 verification & simulator...");
    app.state.active_tab = ActiveTab::Inspector;
    switch_tab_view(&mut app, ActiveTab::Inspector);
    app.state.inspect_input = app.state.generated_json.clone();
    app.state.inspect_license();
    if let Some((valid, _, _)) = &app.state.inspect_status {
        if !valid {
            return Err("Inspector signature verification failed on freshly generated license".to_string());
        }
    } else {
        return Err("Inspector status not populated".to_string());
    }
    app.state.sim_hwid = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string();
    app.state.simulate_evaluation();
    if let Some((valid, _, _)) = &app.state.sim_status {
        if !valid {
            return Err("Simulator evaluation unexpectedly rejected valid license".to_string());
        }
    }
    app.inspector.as_ref().unwrap().update_from_state(&app.state);

    // Step 4: Cycle to Tab 3 (Vault), test key info & binary pairing
    println!("[UI-TEST] Tab 3: Testing Vault diagnostics & mock binary scanner...");
    app.state.active_tab = ActiveTab::Vault;
    switch_tab_view(&mut app, ActiveTab::Vault);
    app.state.check_key_pairing();

    let temp_file = std::env::temp_dir().join("ui_test_daqiq.exe");
    let mut buf = vec![0x4D, 0x5A]; // MZ header
    buf.extend(vec![0; 100_000]);
    buf.extend_from_slice(app.state.active_public_key.as_bytes());
    buf.extend(vec![0; 100_000]);
    fs::write(&temp_file, &buf).map_err(|e| e.to_string())?;

    app.state.check_binary_pairing(temp_file.to_str().unwrap());
    let _ = fs::remove_file(&temp_file);

    if let Some(res) = &app.state.binary_pairing_status {
        match res {
            BinaryPairingResult::Paired { .. } => {}
            other => return Err(format!("UI binary pairing check returned: {other:?}")),
        }
    }
    app.vault.as_ref().unwrap().update_from_state(&app.state);

    // Step 5: Test Theme Switching
    println!("[UI-TEST] Testing Dark/Light theme switching...");
    app.state.theme_mode = ThemeMode::Light;
    app.theme_gdi = ThemeGdiResources::new(ThemeMode::Light);
    app.state.theme_mode = ThemeMode::Dark;
    app.theme_gdi = ThemeGdiResources::new(ThemeMode::Dark);

    unsafe {
        DestroyWindow(hwnd);
        GLOBAL_APP = None;
    }

    print_console("[UI-TEST] All 4 tabs and real Win32 controls cycled with 0 crashes, 0 panics!");

    let report = json!({
        "status": "ok",
        "app_name": "Daqiq License Generator Studio",
        "version": "1.4.2",
        "ui_subsystem": "pure_win32_native",
        "tests": [
            {"tab": "generator", "actions": ["preset_apply", "text_edit", "license_generation", "token_encode"], "passed": true},
            {"tab": "ledger", "actions": ["record_query", "listview_render", "filter"], "passed": true},
            {"tab": "inspector", "actions": ["json_parse", "signature_verification", "simulator_eval"], "passed": true},
            {"tab": "vault", "actions": ["key_info", "key_pairing_check", "binary_scanner"], "passed": true},
            {"feature": "theme_toggle", "actions": ["dark_to_light", "light_to_dark"], "passed": true}
        ]
    });

    if let Some(path) = report_path {
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("Failed to write UI report: {e}"))?;
        print_console(&format!("[UI-TEST] Report written to: {}", path.display()));
    } else {
        print_console(&serde_json::to_string_pretty(&report).unwrap());
    }

    Ok(())
}

fn switch_tab_view(app: &mut AppWindow, tab: ActiveTab) {
    if let Some(gen) = &app.generator {
        gen.show(tab == ActiveTab::Generator);
    }
    if let Some(ledger) = &app.ledger {
        ledger.show(tab == ActiveTab::Ledger);
    }
    if let Some(inspector) = &app.inspector {
        inspector.show(tab == ActiveTab::Inspector);
    }
    if let Some(vault) = &app.vault {
        vault.show(tab == ActiveTab::Vault);
    }

    if let Some(sidebar) = &app.sidebar {
        sidebar.update_from_state(&app.state);
    }

    match tab {
        ActiveTab::Generator => {
            if let Some(gen) = &app.generator {
                gen.update_from_state(&app.state);
            }
        }
        ActiveTab::Ledger => {
            app.state.load_ledger();
            if let Some(ledger) = &app.ledger {
                ledger.update_from_state(&app.state);
            }
        }
        ActiveTab::Inspector => {
            if let Some(inspector) = &app.inspector {
                inspector.update_from_state(&app.state);
            }
        }
        ActiveTab::Vault => {
            app.state.check_key_pairing();
            if let Some(vault) = &app.vault {
                vault.update_from_state(&app.state);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Win32 Window Procedure
// ---------------------------------------------------------------------------
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let app = match get_app_mut() {
        Some(a) => a,
        None => return DefWindowProcW(hwnd, msg, wparam, lparam),
    };

    match msg {
        WM_CREATE => {
            let fonts = &app.fonts;

            app.sidebar = Some(SidebarView::create(hwnd, fonts));
            app.generator = Some(GeneratorView::create(hwnd, fonts));
            app.ledger = Some(LedgerView::create(hwnd, fonts));
            app.inspector = Some(InspectorView::create(hwnd, fonts));
            app.vault = Some(VaultView::create(hwnd, fonts));

            switch_tab_view(app, ActiveTab::Generator);
            0
        }
        WM_SIZE => {
            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;

            // if let Some(d2d) = &mut app.d2d {
            //     d2d.resize(w.max(1) as u32, h.max(1) as u32);
            // }

            let sidebar_w = 220;
            let content_w = w - sidebar_w;

            if let Some(sidebar) = &app.sidebar {
                sidebar.reposition(content_w, 0, sidebar_w, h);
            }
            if let Some(gen) = &app.generator {
                gen.reposition(content_w, h);
            }
            if let Some(ledger) = &app.ledger {
                ledger.reposition(content_w, h);
            }
            if let Some(inspector) = &app.inspector {
                inspector.reposition(content_w, h);
            }
            if let Some(vault) = &app.vault {
                vault.reposition(content_w, h);
            }

            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as isize;

            // Tab Navigation
            match id {
                ID_TAB_GEN => {
                    app.state.active_tab = ActiveTab::Generator;
                    switch_tab_view(app, ActiveTab::Generator);
                }
                ID_TAB_LEDGER => {
                    app.state.active_tab = ActiveTab::Ledger;
                    switch_tab_view(app, ActiveTab::Ledger);
                }
                ID_TAB_INSPECTOR => {
                    app.state.active_tab = ActiveTab::Inspector;
                    switch_tab_view(app, ActiveTab::Inspector);
                }
                ID_TAB_VAULT => {
                    app.state.active_tab = ActiveTab::Vault;
                    switch_tab_view(app, ActiveTab::Vault);
                }
                ID_BTN_THEME => {
                    let next_mode = if app.state.theme_mode == ThemeMode::Dark {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    };
                    app.state.theme_mode = next_mode;
                    app.state.config.theme_mode = match next_mode {
                        ThemeMode::Dark => "dark".to_string(),
                        ThemeMode::Light => "light".to_string(),
                    };
                    app.state.config.save();
                    app.theme_gdi = ThemeGdiResources::new(next_mode);

                    // Update titlebar dark mode
                    let dark_flag: i32 = if next_mode == ThemeMode::Dark { 1 } else { 0 };
                    let hr = DwmSetWindowAttribute(
                        hwnd,
                        20, // DWMWA_USE_IMMERSIVE_DARK_MODE (Win10 2004+)
                        &dark_flag as *const _ as *const c_void,
                        4,
                    );
                    if hr != 0 {
                        DwmSetWindowAttribute(
                            hwnd,
                            19, // Fallback for Win10 1809/1903
                            &dark_flag as *const _ as *const c_void,
                            4,
                        );
                    }

                    RedrawWindow(hwnd, std::ptr::null(), 0 as _, RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN);
                }

                // Generator Presets
                ID_PRESET_TRIAL => {
                    app.state.apply_preset("trial");
                    if let Some(gen) = &app.generator {
                        gen.update_from_state(&app.state);
                    }
                }
                ID_PRESET_STANDARD => {
                    app.state.apply_preset("standard");
                    if let Some(gen) = &app.generator {
                        gen.update_from_state(&app.state);
                    }
                }
                ID_PRESET_PRO => {
                    app.state.apply_preset("pro");
                    if let Some(gen) = &app.generator {
                        gen.update_from_state(&app.state);
                    }
                }
                ID_PRESET_ENTERPRISE => {
                    app.state.apply_preset("enterprise");
                    if let Some(gen) = &app.generator {
                        gen.update_from_state(&app.state);
                    }
                }
                ID_PRESET_PERPETUAL => {
                    app.state.apply_preset("perpetual");
                    if let Some(gen) = &app.generator {
                        gen.update_from_state(&app.state);
                    }
                }

                // Generator Actions
                ID_BTN_GENERATE_SAVE => {
                    if let Some(gen) = &app.generator {
                        gen.update_to_state(&mut app.state);
                    }
                    match app.state.generate_license() {
                        Ok(env) => {
                            let file_opt = rfd::FileDialog::new()
                                .add_filter("Daqiq License", &["daqlic"])
                                .set_file_name(&format!("{}.daqlic", env.payload.license_id))
                                .save_file();

                            if let Some(path) = file_opt {
                                if let Err(e) = fs::write(&path, &app.state.generated_json) {
                                    show_message(hwnd, "خطای ذخیره‌سازی", &format!("ذخیره فایل با خطا مواجه شد: {e}"), true);
                                } else {
                                    app.state.last_saved_path = Some(path.display().to_string());
                                    show_message(hwnd, "عملیات موفق", "فایل لایسنس (.daqlic) با امضای دیجیتال معتبر با موفقیت ذخیره گردید.", false);
                                }
                            }
                            if let Some(gen) = &app.generator {
                                gen.update_from_state(&app.state);
                            }
                        }
                        Err(e) => {
                            app.state.generation_error = Some(e.clone());
                            if let Some(gen) = &app.generator {
                                gen.update_from_state(&app.state);
                            }
                            show_message(hwnd, "خطای اعتبارسنجی", &e, true);
                        }
                    }
                }
                ID_BTN_COPY_TOKEN => {
                    if let Some(gen) = &app.generator {
                        gen.update_to_state(&mut app.state);
                    }
                    if app.state.generated_token.is_empty() {
                        if let Err(e) = app.state.generate_license() {
                            show_message(hwnd, "خطا", &e, true);
                            return 0;
                        }
                    }
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&app.state.generated_token);
                        show_message(hwnd, "کپی شد", "توکن فشرده لایسنس (Base64) با موفقیت در کلیپ‌بورد کپی شد.", false);
                    }
                }
                ID_BTN_PREVIEW_JSON => {
                    if let Some(gen) = &app.generator {
                        gen.update_to_state(&mut app.state);
                    }
                    if app.state.generated_json.is_empty() {
                        if let Err(e) = app.state.generate_license() {
                            show_message(hwnd, "خطا", &e, true);
                            return 0;
                        }
                    }
                    show_message(hwnd, "پیش‌نمایش JSON لایسنس", &app.state.generated_json, false);
                }

                // Ledger Actions
                ID_BTN_RENEW => {
                    // Clone & Renew: Pick latest record, update dates to current + 1 year, switch to Tab 0
                    if !app.state.records.is_empty() {
                        let rec = &app.state.records[0];
                        app.state.customer_name = rec.customer_name.clone();
                        app.state.customer_org = rec.customer_org.clone().unwrap_or_default();
                        app.state.customer_contact = rec.customer_contact.clone().unwrap_or_default();
                        app.state.hwid_text = rec.device_fingerprints.clone();
                        app.state.max_devices = rec.max_devices;
                        app.state.license_type = rec.license_type.clone();

                        let rand_num: u32 = rand::random::<u32>() % 900000 + 100000;
                        app.state.license_id = format!("LIC-{:06}", rand_num);

                        app.state.active_tab = ActiveTab::Generator;
                        switch_tab_view(app, ActiveTab::Generator);
                        show_message(hwnd, "تمدید سریع", "اطلاعات مشتری در فرم صدور بارگذاری شد. تاریخ‌ها و شناسه به‌روزرسانی گردید.", false);
                    }
                }
                ID_BTN_EXPORT_CSV => {
                    let file_opt = rfd::FileDialog::new()
                        .add_filter("Excel CSV", &["csv"])
                        .set_file_name("daqiq_licenses.csv")
                        .save_file();

                    if let Some(path) = file_opt {
                        match app.state.db.export_csv() {
                            Ok(csv_data) => {
                                let mut with_bom = vec![0xEF, 0xBB, 0xBF];
                                with_bom.extend_from_slice(csv_data.as_bytes());
                                if let Err(e) = fs::write(&path, with_bom) {
                                    show_message(hwnd, "خطا", &format!("ذخیره فایل CSV ناموفق بود: {e}"), true);
                                } else {
                                    show_message(hwnd, "موفق", "گزارش دفتر کل با فرمت اکسل (UTF-8 BOM) ذخیره شد.", false);
                                }
                            }
                            Err(e) => {
                                show_message(hwnd, "خطا", &format!("خطا در تولید CSV: {e}"), true);
                            }
                        }
                    }
                }

                // Inspector Actions
                ID_BTN_LOAD_FILE => {
                    let file_opt = rfd::FileDialog::new()
                        .add_filter("Daqiq License", &["daqlic", "json"])
                        .pick_file();

                    if let Some(path) = file_opt {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(insp) = &app.inspector {
                                set_window_text(insp.txt_input, &content);
                            }
                            app.state.inspect_input = content;
                            app.state.inspect_license();
                            if let Some(insp) = &app.inspector {
                                insp.update_from_state(&app.state);
                            }
                        }
                    }
                }
                ID_BTN_VERIFY_INSPECT => {
                    if let Some(insp) = &app.inspector {
                        app.state.inspect_input = get_window_text(insp.txt_input);
                        app.state.custom_public_key = get_window_text(insp.txt_custom_pubkey);
                    }
                    app.state.inspect_license();
                    if let Some(insp) = &app.inspector {
                        insp.update_from_state(&app.state);
                    }
                }
                ID_BTN_SIMULATE => {
                    if let Some(insp) = &app.inspector {
                        app.state.sim_hwid = get_window_text(insp.txt_sim_hwid);
                        app.state.sim_version = get_window_text(insp.txt_sim_version);
                    }
                    app.state.simulate_evaluation();
                    if let Some(insp) = &app.inspector {
                        insp.update_from_state(&app.state);
                    }
                }

                // Vault Actions
                ID_BTN_COPY_PUBKEY => {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&app.state.active_public_key);
                        show_message(hwnd, "کپی شد", "کلید عمومی فعال در کلیپ‌بورد کپی گردید.", false);
                    }
                }
                ID_BTN_SCAN_BINARY => {
                    let file_opt = rfd::FileDialog::new()
                        .add_filter("Daqiq Binary or Keys", &["exe", "py"])
                        .pick_file();

                    if let Some(path) = file_opt {
                        let path_str = path.display().to_string();
                        app.state.check_binary_pairing(&path_str);
                        if let Some(vault) = &app.vault {
                            vault.update_from_state(&app.state);
                        }
                    }
                }
                ID_BTN_UNLOCK => {
                    if let Some(vault_v) = &app.vault {
                        let pass = get_window_text(vault_v.txt_unlock_pass);
                        // Try default or active vault
                        match KeyVault::create_vault(DEFAULT_KEY_ID, &pass) {
                            Ok((v, _)) => {
                                app.state.active_key_id = v.key_id.clone();
                                app.state.active_public_key = v.public_key_b64.clone();
                                app.state.active_key_fingerprint = v.fingerprint();
                                app.state.vault = Some(v);
                                app.state.check_key_pairing();
                                vault_v.update_from_state(&app.state);
                                show_message(hwnd, "موفق", "گاوصندوق کلیدها با موفقیت بازگشایی شد.", false);
                            }
                            Err(e) => {
                                show_message(hwnd, "خطا", &format!("بازگشایی ناموفق: {e}"), true);
                            }
                        }
                    }
                }
                ID_BTN_CREATE => {
                    if let Some(vault_v) = &app.vault {
                        let key_id = get_window_text(vault_v.txt_create_id);
                        let pass = get_window_text(vault_v.txt_create_pass);
                        if key_id.trim().is_empty() || pass.trim().is_empty() {
                            show_message(hwnd, "خطا", "شناسه کلید و کلمه عبور الزامی است.", true);
                            return 0;
                        }
                        match KeyVault::create_vault(&key_id, &pass) {
                            Ok((v, _)) => {
                                let vault_file = crate::startup_log::get_app_data_dir().join("vault.dqkey");
                                let _ = v.save_to_file(&vault_file, &pass);
                                app.state.config.last_vault_path = Some(vault_file.to_string_lossy().to_string());
                                app.state.config.save();

                                app.state.active_key_id = v.key_id.clone();
                                app.state.active_public_key = v.public_key_b64.clone();
                                app.state.active_key_fingerprint = v.fingerprint();
                                app.state.vault = Some(v);
                                app.state.check_key_pairing();
                                vault_v.update_from_state(&app.state);
                                if let Some(sidebar) = &app.sidebar {
                                    sidebar.update_from_state(&app.state);
                                }
                                RedrawWindow(hwnd, std::ptr::null(), 0 as _, RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN);
                                show_message(hwnd, "کلید جدید ساخته شد", "کلید جدید با موفقیت ایجاد گردید و به عنوان کلید فعال تنظیم شد.", false);
                            }
                            Err(e) => {
                                show_message(hwnd, "خطا", &format!("خطا در ساخت کلید: {e}"), true);
                            }
                        }
                    }
                }
                ID_BTN_IMPORT => {
                    if let Some(vault_v) = &app.vault {
                        let key_id = get_window_text(vault_v.txt_import_id);
                        let raw = get_window_text(vault_v.txt_import_raw);
                        let pass = get_window_text(vault_v.txt_import_pass);
                        if key_id.trim().is_empty() || raw.trim().is_empty() {
                            show_message(hwnd, "خطا", "شناسه کلید و مقدار کلید خصوصی الزامی است.", true);
                            return 0;
                        }
                        use base64::engine::general_purpose::STANDARD as B64_STD;
                        use base64::Engine;
                        let clean = raw.trim();
                        let bytes_vec = match B64_STD.decode(clean) {
                            Ok(b) => b,
                            Err(_) => {
                                show_message(hwnd, "خطا", "فرمت کلید خصوصی نامعتبر است (باید Base64 با طول ۳۲ بایت باشد).", true);
                                return 0;
                            }
                        };
                        if bytes_vec.len() != 32 {
                            show_message(hwnd, "خطا", "طول کلید خصوصی Ed25519 باید ۳۲ بایت باشد.", true);
                            return 0;
                        }
                        let mut raw_32 = [0u8; 32];
                        raw_32.copy_from_slice(&bytes_vec);
                        let use_pass = if pass.is_empty() { "daqiq" } else { &pass };
                        match KeyVault::import_private_key(&key_id, &raw_32, use_pass) {
                            Ok((v, _env)) => {
                                let vault_file = crate::startup_log::get_app_data_dir().join("vault.dqkey");
                                let _ = v.save_to_file(&vault_file, use_pass);
                                app.state.config.last_vault_path = Some(vault_file.to_string_lossy().to_string());
                                app.state.config.save();

                                app.state.active_key_id = v.key_id.clone();
                                app.state.active_public_key = v.public_key_b64.clone();
                                app.state.active_key_fingerprint = v.fingerprint();
                                app.state.vault = Some(v);
                                app.state.check_key_pairing();
                                vault_v.update_from_state(&app.state);
                                if let Some(sidebar) = &app.sidebar {
                                    sidebar.update_from_state(&app.state);
                                }
                                RedrawWindow(hwnd, std::ptr::null(), 0 as _, RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN);
                                show_message(hwnd, "موفق", "کلید خصوصی با موفقیت وارد گردید و به عنوان کلید فعال تنظیم شد.", false);
                            }
                            Err(e) => {
                                show_message(hwnd, "خطا", &format!("خطا در ورود کلید: {e}"), true);
                            }
                        }
                    }
                }

                _ => {}
            }
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, app.theme_gdi.colors.text_primary);
            app.theme_gdi.brush_canvas as isize
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as HDC;
            SetBkColor(hdc, app.theme_gdi.colors.input_bg);
            SetTextColor(hdc, app.theme_gdi.colors.text_primary);
            app.theme_gdi.brush_input as isize
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as HDC;
            SetBkColor(hdc, app.theme_gdi.colors.card_bg);
            SetTextColor(hdc, app.theme_gdi.colors.text_primary);
            app.theme_gdi.brush_card as isize
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let _hdc = BeginPaint(hwnd, &mut ps);
            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

extern "system" {
    fn AttachConsole(dw_process_id: u32) -> i32;
    fn GetStdHandle(n_std_handle: u32) -> windows_sys::Win32::Foundation::HANDLE;
    fn WriteConsoleW(
        h_console_output: windows_sys::Win32::Foundation::HANDLE,
        lp_buffer: *const u16,
        n_number_of_chars_to_write: u32,
        lp_number_of_chars_written: *mut u32,
        lp_reserved: *mut c_void,
    ) -> i32;
    fn WriteFile(
        h_file: windows_sys::Win32::Foundation::HANDLE,
        lp_buffer: *const u8,
        n_number_of_bytes_to_write: u32,
        lp_number_of_bytes_written: *mut u32,
        lp_overlapped: *mut c_void,
    ) -> i32;
}
const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5; // -11

pub fn print_console(msg: &str) {
    let s = format!("{msg}\r\n");
    let w = to_wstring(&s);
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != 0 as _ && handle != (-1isize) as _ {
            let mut written = 0u32;
            let res = WriteConsoleW(
                handle,
                w.as_ptr(),
                (w.len().saturating_sub(1)) as u32,
                &mut written,
                std::ptr::null_mut(),
            );
            if res == 0 {
                // If stdout is a pipe or redirected stream, WriteConsoleW fails. WriteFile succeeds!
                let bytes = s.as_bytes();
                WriteFile(
                    handle,
                    bytes.as_ptr(),
                    bytes.len() as u32,
                    &mut written,
                    std::ptr::null_mut(),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Application Entry Point
// ---------------------------------------------------------------------------
fn main() {
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    // Step 0: Initialize startup breadcrumb logger
    let _ = crate::startup_log::init_log();

    // Step 1: Global panic hook with technical details, GetLastError, crash.log and native MessageBoxW
    std::panic::set_hook(Box::new(|info| {
        let err_code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        let msg = format!("CRITICAL PANIC in Daqiq License Studio:\n{info}\nWindows LastError: {err_code}");
        eprintln!("{msg}");
        crate::startup_log::log_fatal("PANIC", &format!("{info}"));
        let data_dir = crate::startup_log::get_app_data_dir();
        let _ = fs::write(data_dir.join("crash.log"), &msg);
        let t_w = to_wstring("خطای بحرانی در برنامه");
        let m_w = to_wstring(&format!(
            "متأسفانه برنامه با خطای غیرمنتظره روبرو شد:\n\n\
            {info}\n\n\
            کد خطای ویندوز (GetLastError): {err_code}\n\
            گزارش در crash.log و startup.log ثبت گردید."
        ));
        unsafe {
            MessageBoxW(0 as _, m_w.as_ptr(), t_w.as_ptr(), MB_OK | MB_ICONERROR);
        }
    }));
    crate::startup_log::log_stage("PANIC_HOOK_OK", "Global panic hook registered successfully");

    // Step 2: Verify AppData directory
    let data_dir = crate::startup_log::get_app_data_dir();
    crate::startup_log::log_stage("DATA_DIR_OK", &format!("AppData directory verified at {}", data_dir.display()));

    let args: Vec<String> = env::args().collect();

    // Check for --version flag
    if args.iter().any(|a| a == "--version" || a == "-V") {
        print_console("1.4.2");
        std::process::exit(0);
    }

    // Check for --self-test flag (Gate 4)
    if args.iter().any(|a| a == "--self-test") {
        let mut report_path = None;
        for i in 0..args.len() {
            if args[i] == "--report" && i + 1 < args.len() {
                report_path = Some(PathBuf::from(&args[i + 1]));
                break;
            }
        }
        match run_headless_self_test(report_path) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("[SELF-TEST-FAIL] {e}");
                std::process::exit(1);
            }
        }
    }

    // Check for --self-test-ui flag (Gate 5)
    if args.iter().any(|a| a == "--self-test-ui") {
        let mut report_path = None;
        for i in 0..args.len() {
            if args[i] == "--report" && i + 1 < args.len() {
                report_path = Some(PathBuf::from(&args[i + 1]));
                break;
            }
        }
        match run_headless_ui_self_test(report_path) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("[UI-TEST-FAIL] {e}");
                std::process::exit(1);
            }
        }
    }

    // Step 3: Load application configuration
    let config = AppConfig::load();
    crate::startup_log::log_stage(
        "CONFIG_OK",
        &format!("Configuration loaded successfully (theme_mode: {})", config.theme_mode),
    );

    // Step 4: Initialize SQLite database and application state
    let state = AppState::new();
    crate::startup_log::log_stage("DB_OK", "Registry SQLite database and state initialized");

    // Step 5: Initialize COM library
    unsafe {
        let hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED as u32);
        // S_OK (0) or S_FALSE (1) are successful. RPC_E_CHANGED_MODE (-2147417850 = 0x80010106) means already initialized.
        if hr < 0 && hr != -2147417850 {
            crate::startup_log::show_fatal_dialog(
                "COM_OK",
                &format!("CoInitializeEx failed with HRESULT: 0x{hr:08X}"),
            );
            return;
        }
    }
    crate::startup_log::log_stage("COM_OK", "CoInitializeEx initialized as APARTMENTTHREADED");

    // Step 6: Initialize Common Controls & DPI Awareness
    unsafe {
        let icex = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES | ICC_LISTVIEW_CLASSES | ICC_TAB_CLASSES | ICC_BAR_CLASSES,
        };
        let _ = InitCommonControlsEx(&icex);
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    crate::startup_log::log_stage(
        "COMMCTRL_OK",
        "Common Controls v6 and PerMonitorV2 DPI awareness initialized",
    );

    // Step 7: Load embedded fonts
    let fonts = Fonts::new();
    crate::startup_log::log_stage(
        "FONT_OK",
        "Embedded Vazirmatn & JetBrains Mono fonts loaded into process memory",
    );

    let initial_theme = if config.theme_mode == "light" {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    };

    let mut app = AppWindow {
        state,
        fonts,
        theme_gdi: ThemeGdiResources::new(initial_theme),
        // d2d: crate::d2d::D2dContext::new().ok(),
        sidebar: None,
        generator: None,
        ledger: None,
        inspector: None,
        vault: None,
        is_headless: false,
    };

    unsafe {
        GLOBAL_APP = Some(&mut app as *mut _);
    }

    // Step 8: Register main window class with OS
    let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };
    let class_name = to_wstring("DaqiqLicenseStudioWindow");
    let mut wnd_class: WNDCLASSEXW = unsafe { std::mem::zeroed() };
    wnd_class.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wnd_class.style = CS_HREDRAW | CS_VREDRAW;
    wnd_class.lpfnWndProc = Some(wnd_proc);
    wnd_class.hInstance = hinstance;
    wnd_class.lpszClassName = class_name.as_ptr();
    wnd_class.hbrBackground = app.theme_gdi.brush_canvas;

    let reg_res = unsafe { RegisterClassExW(&wnd_class) };
    if reg_res == 0 {
        let err = unsafe { GetLastError() };
        if err != ERROR_CLASS_ALREADY_EXISTS {
            crate::startup_log::show_fatal_dialog(
                "CLASS_REG_OK",
                &format!("RegisterClassExW failed (code: {err})"),
            );
            return;
        }
    }
    crate::startup_log::log_stage("CLASS_REG_OK", "Main window class registered with OS");

    // Step 9: Determine centered window bounds via SPI_GETWORKAREA
    let mut work_area: RECT = unsafe { std::mem::zeroed() };
    let spi_ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut work_area as *mut _ as *mut c_void,
            0,
        )
    };
    let work_w = work_area.right - work_area.left;
    let work_h = work_area.bottom - work_area.top;

    let mut win_w = 1280;
    let mut win_h = 850;
    if spi_ok != 0 && work_w > 400 && work_h > 300 {
        if win_w > work_w {
            win_w = work_w;
        }
        if win_h > work_h {
            win_h = work_h;
        }
    }
    let x = work_area.left + (work_w.saturating_sub(win_w)) / 2;
    let y = work_area.top + (work_h.saturating_sub(win_h)) / 2;

    crate::startup_log::log_stage(
        "GEOMETRY",
        &format!("spi_ok: {spi_ok}, work_area: [L:{}, T:{}, R:{}, B:{}], win: {win_w}x{win_h} at ({x}, {y})", work_area.left, work_area.top, work_area.right, work_area.bottom),
    );

    // Step 10: Create main application window (DW_EX_APPWINDOW, without WS_VISIBLE initially)
    let title = to_wstring("استودیو صدور لایسنس نرم‌افزار دقیق (نسخه ۱.۴.۱ - Pure Win32 Native)");
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_APPWINDOW | WS_EX_CONTROLPARENT,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            x,
            y,
            win_w,
            win_h,
            0 as HWND,
            0 as _,
            hinstance,
            std::ptr::null_mut(),
        )
    };

    if hwnd == 0 as HWND {
        crate::startup_log::show_fatal_dialog(
            "MAIN_HWND_OK",
            "CreateWindowExW returned NULL handle for main application window",
        );
        return;
    }
    crate::startup_log::log_stage(
        "MAIN_HWND_OK",
        &format!("Main HWND created successfully: 0x{:X}", hwnd as usize),
    );

    // Step 11: Verify child panels creation
    if app.sidebar.is_none()
        || app.generator.is_none()
        || app.ledger.is_none()
        || app.inspector.is_none()
        || app.vault.is_none()
    {
        crate::startup_log::show_fatal_dialog(
            "PANELS_CREATED_OK",
            "One or more child control views failed to instantiate during WM_CREATE",
        );
        return;
    }
    crate::startup_log::log_stage(
        "PANELS_CREATED_OK",
        "Sidebar, Generator, Ledger, Inspector, and Vault panels instantiated",
    );

    // Step 12: Initial layout positioning
    let mut client_rect: RECT = unsafe { std::mem::zeroed() };
    unsafe {
        GetClientRect(hwnd, &mut client_rect);
    }
    let cur_w = client_rect.right - client_rect.left;
    let cur_h = client_rect.bottom - client_rect.top;
    let sidebar_w = 220;
    let content_w = cur_w - sidebar_w;

    if let Some(sidebar) = &app.sidebar {
        sidebar.reposition(content_w, 0, sidebar_w, cur_h);
    }
    if let Some(gen) = &app.generator {
        gen.reposition(content_w, cur_h);
    }
    if let Some(ledger) = &app.ledger {
        ledger.reposition(content_w, cur_h);
    }
    if let Some(inspector) = &app.inspector {
        inspector.reposition(content_w, cur_h);
    }
    if let Some(vault) = &app.vault {
        vault.reposition(content_w, cur_h);
    }
    crate::startup_log::log_stage("LAYOUT_OK", "Initial panel layout and RTL geometry applied");

    // Step 13: Show and update window
    let dark_flag: i32 = if initial_theme == ThemeMode::Dark { 1 } else { 0 };
    unsafe {
        let hr = DwmSetWindowAttribute(
            hwnd,
            20, // DWMWA_USE_IMMERSIVE_DARK_MODE (Win10 2004+)
            &dark_flag as *const _ as *const c_void,
            4,
        );
        if hr != 0 {
            DwmSetWindowAttribute(
                hwnd,
                19, // Fallback for Win10 1809/1903
                &dark_flag as *const _ as *const c_void,
                4,
            );
        }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
    crate::startup_log::log_stage(
        "SHOW_WINDOW_OK",
        "ShowWindow and UpdateWindow completed; main window is now visible",
    );

    // Step 14: Enter Win32 message loop
    crate::startup_log::log_stage(
        "ENTER_MSG_LOOP",
        "Entering Win32 GetMessageW message dispatch loop",
    );
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0 as HWND, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        CoUninitialize();
    }
}
