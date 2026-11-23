#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod rtl;
mod state;
mod theme;
mod views;

use std::env;
use std::fs;
use std::path::PathBuf;

use iced::widget::{column, container, row};
use iced::{window, Element, Font, Length, Size, Task, Theme};
use serde_json::json;

use daqiq_lic_core::{
    canonicalize_payload, gregorian_to_jalali, jalali_to_gregorian, validate_hwid,
    verify_license_envelope, CustomerInfo, KeyVault, LicenseEnvelope, LicensePayload, RegistryDb,
    ALL_FEATURE_KEYS, DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
};
use state::{ActiveTab, AppState, Message, Preset, ThemeMode};
use theme::{dark_theme, light_theme};

fn run_headless_self_test(report_path: Option<PathBuf>) -> Result<(), String> {
    println!("[SELF-TEST] Starting headless cryptographic, subsystem & GUI verification...");

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

    // Test 5: In-Memory SQLite Registry
    let db = RegistryDb::open_in_memory().map_err(|e| e.to_string())?;
    db.insert_license(&envelope).map_err(|e| e.to_string())?;
    let records = db.list_licenses(10, 0).map_err(|e| e.to_string())?;
    if records.len() != 1 || records[0].license_id != "LIC-2026-TEST01" {
        return Err("SQLite in-memory registry failed insertion/query".to_string());
    }

    // Test 6: Headless GUI State Machine & Message Reducer Tests
    let mut state = AppState::new();
    state.active_key_id = vault.key_id.clone();
    state.active_public_key = vault.public_key_b64.clone();
    state.vault = Some(vault);
    state.customer_name = "مشتری تستی".to_string();
    state.hwid_text = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string();
    state.update(Message::HwidTextChanged(state.hwid_text.clone()));
    state.update(Message::ApplyPreset(Preset::Standard));
    state.update(Message::GenerateLicenseClicked);

    if state.generated_envelope.is_none() || state.generated_json.is_empty() {
        return Err("Headless GUI state machine failed to generate license!".to_string());
    }

    state.update(Message::SwitchTab(ActiveTab::Ledger));
    state.update(Message::ToggleTheme);
    state.update(Message::SwitchTab(ActiveTab::Inspector));
    state.inspect_text = state.generated_json.clone();
    state.update(Message::VerifyInspectClicked);
    if let Some((valid, _, _)) = &state.inspect_status {
        if !valid {
            return Err("Headless inspector verification of generated license failed!".to_string());
        }
    } else {
        return Err("Inspector status not populated!".to_string());
    }

    println!("[SELF-TEST] All 6 internal checks (including headless GUI reducer) passed!");

    let report = json!({
        "status": "ok",
        "app_name": "Daqiq License Generator Studio",
        "version": "1.3.0",
        "target_product": DEFAULT_PRODUCT_ID,
        "default_key_id": DEFAULT_KEY_ID,
        "default_public_key": DEFAULT_PROD_PUBLIC_KEY,
        "tests": [
            {"name": "golden_vector_sha256_and_ed25519", "passed": true},
            {"name": "hwid_unicode_cleaner_and_validation", "passed": true},
            {"name": "jalali_calendar_roundtrip", "passed": true},
            {"name": "encrypted_vault_roundtrip", "passed": true},
            {"name": "sqlite_registry_operations", "passed": true},
            {"name": "gui_state_machine_reducer", "passed": true}
        ]
    });

    if let Some(path) = report_path {
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("Failed to write report file: {e}"))?;
        println!("[SELF-TEST] Report written to: {}", path.display());
    } else {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    }

    Ok(())
}

impl AppState {
    fn update_task(&mut self, message: Message) -> Task<Message> {
        self.update(message);
        Task::none()
    }

    fn view_app(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.active_tab {
            ActiveTab::Generator => views::generator::view_generator(self),
            ActiveTab::Ledger => views::registry::view_registry(self),
            ActiveTab::Inspector => views::inspector::view_inspector(self),
            ActiveTab::Vault => views::vault::view_vault(self),
        };

        let main_area = column![
            views::header::view_header(self),
            container(content)
                .height(Length::Fill)
                .width(Length::Fill),
        ]
        .height(Length::Fill)
        .width(Length::Fill);

        // Right-Anchored RTL Sidebar (~220px) + Main Area
        row![
            container(main_area)
                .height(Length::Fill)
                .width(Length::Fill),
            views::sidebar::view_sidebar(self),
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    fn theme_app(&self) -> Theme {
        if self.theme_mode == ThemeMode::Dark {
            dark_theme()
        } else {
            light_theme()
        }
    }
}

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();

    // Check for --self-test flag
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

    // Check for --version flag
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("1.3.0");
        std::process::exit(0);
    }

    let window_settings = window::Settings {
        size: Size::new(1280.0, 850.0),
        min_size: Some(Size::new(1024.0, 700.0)),
        position: window::Position::Centered,
        ..Default::default()
    };

    iced::application(AppState::new, AppState::update_task, AppState::view_app)
        .title("استودیو صدور لایسنس نرم‌افزار دقیق (نسخه ۱.۳.۰ - Pure Native Rust)")
        .theme(AppState::theme_app)
        .window(window_settings)
        .font(include_bytes!("../assets/fonts/Vazirmatn-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Vazirmatn-Bold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice())
        .default_font(Font {
            family: iced::font::Family::Name("Vazirmatn"),
            ..Font::DEFAULT
        })
        .run()
}
