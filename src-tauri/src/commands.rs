use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use daqiq_lic_core::{
    canonicalize_payload, clean_hwid_input, jalali_str_to_utc_iso, parse_bulk_hwids,
    parse_daqreq, utc_iso_to_jalali_str, validate_hwid, verify_license_envelope, CustomerInfo,
    DeviceRequest, KeyVault, LicenseEnvelope, LicensePayload, LicenseSpec, RegistryDb,
    RegistryRecord, VaultEnvelope, ALL_FEATURE_KEYS, DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID,
    DEFAULT_PROD_PUBLIC_KEY,
};

pub struct AppState {
    pub vault: Mutex<Option<KeyVault>>,
    pub db: Mutex<RegistryDb>,
}

#[derive(Serialize, Deserialize)]
pub struct GenerateResult {
    pub envelope: LicenseEnvelope,
    pub json_text: String,
    pub sha256_digest: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyResult {
    pub is_valid: bool,
    pub status: String,
    pub message: String,
    pub envelope: Option<LicenseEnvelope>,
    pub sha256_digest: String,
}

#[derive(Serialize, Deserialize)]
pub struct SimulationResult {
    pub is_valid: bool,
    pub status: String,
    pub status_fa: String,
    pub detail: String,
}

#[derive(Serialize, Deserialize)]
pub struct VaultInfo {
    pub key_id: String,
    pub public_key_b64: String,
    pub is_unlocked: bool,
}

#[derive(Serialize, Deserialize)]
pub struct HwidParseResult {
    pub valid_hwids: Vec<String>,
    pub invalid_entries: Vec<String>,
    pub total_valid: usize,
}

pub fn get_app_data_db_path() -> PathBuf {
    let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("DaqiqLicenseGenerator");
    std::fs::create_dir_all(&dir).ok();
    dir.push("registry.sqlite3");
    dir
}

#[tauri::command]
pub fn get_vault_status(state: tauri::State<AppState>) -> Result<VaultInfo, String> {
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    if let Some(v) = guard.as_ref() {
        Ok(VaultInfo {
            key_id: v.key_id.clone(),
            public_key_b64: v.public_key_b64.clone(),
            is_unlocked: true,
        })
    } else {
        Ok(VaultInfo {
            key_id: DEFAULT_KEY_ID.to_string(),
            public_key_b64: DEFAULT_PROD_PUBLIC_KEY.to_string(),
            is_unlocked: false,
        })
    }
}

#[tauri::command]
pub fn create_vault(
    key_id: String,
    passphrase: String,
    state: tauri::State<AppState>,
) -> Result<(VaultInfo, VaultEnvelope), String> {
    if passphrase.trim().is_empty() {
        return Err("رمز عبور نمی‌تواند خالی باشد".to_string());
    }
    let (vault, envelope) = KeyVault::create_vault(&key_id, &passphrase)
        .map_err(|e| format!("خطا در ایجاد مخزن کلید: {e}"))?;

    let info = VaultInfo {
        key_id: vault.key_id.clone(),
        public_key_b64: vault.public_key_b64.clone(),
        is_unlocked: true,
    };

    let mut guard = state.vault.lock().map_err(|e| e.to_string())?;
    *guard = Some(vault);

    Ok((info, envelope))
}

#[tauri::command]
pub fn unlock_vault(
    envelope_json: String,
    passphrase: String,
    state: tauri::State<AppState>,
) -> Result<VaultInfo, String> {
    let envelope: VaultEnvelope = serde_json::from_str(&envelope_json)
        .map_err(|e| format!("فرمت فایل مخزن کلید نامعتبر است: {e}"))?;
    let vault = KeyVault::load_from_envelope(&envelope, &passphrase)
        .map_err(|e| format!("خطا در بازگشایی قفل مخزن: {e}"))?;

    let info = VaultInfo {
        key_id: vault.key_id.clone(),
        public_key_b64: vault.public_key_b64.clone(),
        is_unlocked: true,
    };

    let mut guard = state.vault.lock().map_err(|e| e.to_string())?;
    *guard = Some(vault);

    Ok(info)
}

#[tauri::command]
pub fn lock_vault(state: tauri::State<AppState>) -> Result<(), String> {
    let mut guard = state.vault.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}

#[tauri::command]
pub fn generate_license(
    spec: LicenseSpec,
    state: tauri::State<AppState>,
) -> Result<GenerateResult, String> {
    // 1. Check if vault is unlocked
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or_else(|| {
        "مخزن کلید امضا قفل است. لطفاً ابتدا مخزن کلید را بازگشایی کرده یا کلید جدید ایجاد نمایید."
            .to_string()
    })?;

    // 2. Validate customer name
    if spec.customer_name.trim().is_empty() {
        return Err("نام مشتری نمی‌تواند خالی باشد".to_string());
    }

    // 3. Validate device fingerprints
    if spec.device_fingerprints.is_empty() {
        return Err("حداقل یک شناسه سخت‌افزاری (HWID) باید وارد شود".to_string());
    }
    if spec.device_fingerprints.len() > spec.max_devices as usize {
        return Err(format!(
            "تعداد شناسه‌ها ({}) از حداکثر دستگاه‌های مجاز ({}) بیشتر است",
            spec.device_fingerprints.len(),
            spec.max_devices
        ));
    }
    for fp in &spec.device_fingerprints {
        validate_hwid(fp).map_err(|e| e.to_string())?;
    }

    // 4. Build payload
    let payload = LicensePayload {
        customer: CustomerInfo {
            name: spec.customer_name.trim().to_string(),
            organization: spec.customer_organization.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            contact: spec.customer_contact.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        },
        device_fingerprints: spec.device_fingerprints,
        expires_at: if spec.license_type == "perpetual" {
            None
        } else {
            spec.expires_at
        },
        features: spec.features,
        format_version: 1,
        issued_at: spec.issued_at,
        key_id: vault.key_id.clone(),
        license_id: spec.license_id.trim().to_string(),
        license_type: spec.license_type,
        max_app_version: spec.max_app_version.filter(|s| !s.trim().is_empty()),
        max_devices: spec.max_devices,
        min_app_version: spec.min_app_version.filter(|s| !s.trim().is_empty()),
        not_before: spec.not_before,
        product_id: DEFAULT_PRODUCT_ID.to_string(),
    };

    // 5. Sign license
    let envelope = vault
        .sign_license(&payload)
        .map_err(|e| format!("خطا در امضای لایسنس: {e}"))?;

    let json_text = serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("خطا در سریال‌سازی لایسنس: {e}"))?;

    let canonical_bytes = canonicalize_payload(&envelope.payload)
        .map_err(|e| format!("خطا در ایجاد بایت‌های استاندارد: {e}"))?;
    let mut msg = Vec::with_capacity(daqiq_lic_core::DOMAIN_PREFIX.len() + canonical_bytes.len());
    msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
    msg.extend_from_slice(&canonical_bytes);
    let mut hasher = Sha256::new();
    hasher.update(&msg);
    let sha256_digest = format!("{:x}", hasher.finalize());

    // 6. Record in SQLite ledger
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    db_guard
        .insert_license(&envelope)
        .map_err(|e| format!("خطا در ذخیره در بایگانی: {e}"))?;

    Ok(GenerateResult {
        envelope,
        json_text,
        sha256_digest,
    })
}

#[tauri::command]
pub fn verify_license(
    json_text: String,
    public_key_b64: Option<String>,
) -> Result<VerifyResult, String> {
    let envelope: LicenseEnvelope = serde_json::from_str(&json_text)
        .map_err(|e| format!("فرمت فایل JSON نامعتبر است: {e}"))?;

    let pubkey = public_key_b64.unwrap_or_else(|| DEFAULT_PROD_PUBLIC_KEY.to_string());

    let canon = canonicalize_payload(&envelope.payload)
        .map_err(|e| format!("خطا در محاسبه محتوای استاندارد: {e}"))?;
    let mut msg = Vec::with_capacity(daqiq_lic_core::DOMAIN_PREFIX.len() + canon.len());
    msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
    msg.extend_from_slice(&canon);
    let mut hasher = Sha256::new();
    hasher.update(&msg);
    let sha256_digest = format!("{:x}", hasher.finalize());

    match verify_license_envelope(&envelope, &pubkey) {
        Ok(true) => Ok(VerifyResult {
            is_valid: true,
            status: "VALID".to_string(),
            message: "امضای دیجیتال لایسنس کاملاً معتبر و دست‌نخورده است.".to_string(),
            envelope: Some(envelope),
            sha256_digest,
        }),
        Ok(false) | Err(daqiq_lic_core::LicError::InvalidSignature) => Ok(VerifyResult {
            is_valid: false,
            status: "INVALID_SIGNATURE".to_string(),
            message: "امضای دیجیتال لایسنس نامعتبر است یا محتوا دستکاری شده است.".to_string(),
            envelope: Some(envelope),
            sha256_digest,
        }),
        Err(e) => Err(format!("خطا در بررسی اعتبار: {e}")),
    }
}

#[tauri::command]
pub fn simulate_validation(
    json_text: String,
    current_hwid: String,
    current_time_iso: Option<String>,
    current_app_version: Option<String>,
    public_key_b64: Option<String>,
) -> Result<SimulationResult, String> {
    let envelope: LicenseEnvelope = serde_json::from_str(&json_text)
        .map_err(|e| format!("فرمت لایسنس نامعتبر است: {e}"))?;

    let pubkey = public_key_b64.unwrap_or_else(|| DEFAULT_PROD_PUBLIC_KEY.to_string());

    // 1. Signature check
    if verify_license_envelope(&envelope, &pubkey).is_err() {
        return Ok(SimulationResult {
            is_valid: false,
            status: "INVALID_SIGNATURE".to_string(),
            status_fa: "امضای نامعتبر یا فایل دستکاری شده".to_string(),
            detail: "امضای دیجیتال این فایل با کلید عمومی تطابق ندارد.".to_string(),
        });
    }

    let p = &envelope.payload;

    // 2. Hardware check
    let clean_hwid = clean_hwid_input(&current_hwid);
    if !p.device_fingerprints.contains(&clean_hwid) {
        return Ok(SimulationResult {
            is_valid: false,
            status: "UNAUTHORIZED_DEVICE".to_string(),
            status_fa: "دستگاه غیرمجاز".to_string(),
            detail: format!(
                "شناسه سخت‌افزاری دستگاه ({clean_hwid}) در لیست دستگاه‌های مجاز این لایسنس وجود ندارد."
            ),
        });
    }

    // 3. Time check
    let now = if let Some(iso) = current_time_iso {
        chrono::DateTime::parse_from_rfc3339(&iso)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    if let Ok(nb) = chrono::DateTime::parse_from_rfc3339(&p.not_before) {
        let nb_utc = nb.with_timezone(&Utc);
        if now < nb_utc {
            return Ok(SimulationResult {
                is_valid: false,
                status: "NOT_YET_VALID".to_string(),
                status_fa: "هنوز فعال نشده".to_string(),
                detail: format!("تاریخ شروع اعتبار این لایسنس {} می‌باشد.", p.not_before),
            });
        }
    }

    if let Some(exp_str) = &p.expires_at {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(exp_str) {
            let exp_utc = exp.with_timezone(&Utc);
            if now > exp_utc {
                return Ok(SimulationResult {
                    is_valid: false,
                    status: "EXPIRED".to_string(),
                    status_fa: "منقضی شده".to_string(),
                    detail: format!("اعتبار اشتراک این لایسنس در تاریخ {exp_str} به پایان رسیده است."),
                });
            }
        }
    }

    // 4. App version check
    let app_ver = current_app_version.unwrap_or_else(|| "4.9.0".to_string());
    if let Some(min_v) = &p.min_app_version {
        if app_ver.as_str() < min_v.as_str() {
            return Ok(SimulationResult {
                is_valid: false,
                status: "UNSUPPORTED_APP_VERSION".to_string(),
                status_fa: "نسخه نرم‌افزار قدیمی است".to_string(),
                detail: format!("حداقل نسخه مورد نیاز نرم‌افزار {min_v} است (نسخه فعلی: {app_ver})."),
            });
        }
    }
    if let Some(max_v) = &p.max_app_version {
        if app_ver.as_str() > max_v.as_str() {
            return Ok(SimulationResult {
                is_valid: false,
                status: "UNSUPPORTED_APP_VERSION".to_string(),
                status_fa: "نسخه نرم‌افزار بالاتر از حد مجاز است".to_string(),
                detail: format!("حداکثر نسخه مجاز نرم‌افزار {max_v} است (نسخه فعلی: {app_ver})."),
            });
        }
    }

    Ok(SimulationResult {
        is_valid: true,
        status: "VALID".to_string(),
        status_fa: "معتبر و فعال".to_string(),
        detail: "این لایسنس تمام شروط اعتبارسنجی را احراز کرده و توسط نرم‌افزار دقیق پذیرفته می‌شود."
            .to_string(),
    })
}

#[tauri::command]
pub fn list_registry_records(
    limit: u32,
    offset: u32,
    state: tauri::State<AppState>,
) -> Result<Vec<RegistryRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_licenses(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_registry_records(
    query: String,
    state: tauri::State<AppState>,
) -> Result<Vec<RegistryRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_licenses(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_registry_csv(state: tauri::State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.export_csv().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clean_and_parse_hwids(text: String) -> Result<HwidParseResult, String> {
    let (valid, invalid) = parse_bulk_hwids(&text);
    let total = valid.len();
    Ok(HwidParseResult {
        valid_hwids: valid,
        invalid_entries: invalid,
        total_valid: total,
    })
}

#[tauri::command]
pub fn parse_daqreq_file(json_str: String) -> Result<DeviceRequest, String> {
    parse_daqreq(&json_str).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn convert_jalali_to_gregorian(jalali_str: String, end_of_day: bool) -> Result<String, String> {
    jalali_str_to_utc_iso(&jalali_str, end_of_day).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn convert_gregorian_to_jalali(iso_str: String) -> Result<String, String> {
    utc_iso_to_jalali_str(&iso_str).map_err(|e| e.to_string())
}
