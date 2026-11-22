// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use commands::{get_app_data_db_path, AppState};
use daqiq_lic_core::{
    canonicalize_payload, gregorian_to_jalali, jalali_to_gregorian, validate_hwid,
    verify_license_envelope, CustomerInfo, KeyVault, LicenseEnvelope, LicensePayload, RegistryDb,
    ALL_FEATURE_KEYS, DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
};
use serde_json::json;

fn run_headless_self_test(report_path: Option<PathBuf>) -> Result<(), String> {
    println!("[SELF-TEST] Starting headless cryptographic & subsystem verification...");

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

    println!("[SELF-TEST] All 5 internal checks passed successfully!");

    let report = json!({
        "status": "ok",
        "app_name": "Daqiq License Generator Studio",
        "version": "1.1.0",
        "target_product": DEFAULT_PRODUCT_ID,
        "default_key_id": DEFAULT_KEY_ID,
        "default_public_key": DEFAULT_PROD_PUBLIC_KEY,
        "tests": [
            {"name": "golden_vector_sha256_and_ed25519", "passed": true},
            {"name": "hwid_unicode_cleaner_and_validation", "passed": true},
            {"name": "jalali_calendar_roundtrip", "passed": true},
            {"name": "encrypted_vault_roundtrip", "passed": true},
            {"name": "sqlite_registry_operations", "passed": true}
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

fn main() {
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

    let db_path = get_app_data_db_path();
    let db = match RegistryDb::open(&db_path) {
        Ok(d) => d,
        Err(_) => RegistryDb::open_in_memory().expect("In-memory database fallback must succeed"),
    };

    let state = AppState {
        vault: Mutex::new(None),
        db: Mutex::new(db),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_vault_status,
            commands::create_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::generate_license,
            commands::verify_license,
            commands::simulate_validation,
            commands::list_registry_records,
            commands::search_registry_records,
            commands::export_registry_csv,
            commands::clean_and_parse_hwids,
            commands::parse_daqreq_file,
            commands::convert_jalali_to_gregorian,
            commands::convert_gregorian_to_jalali,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
