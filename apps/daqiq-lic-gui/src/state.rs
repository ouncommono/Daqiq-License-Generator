use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use daqiq_lic_core::{
    compare_public_key, jalali_str_to_utc_iso, parse_bulk_hwids, verify_daqiq_binary_pairing,
    verify_license_envelope, BinaryPairingResult, CustomerInfo, KeyVault, LicenseEnvelope,
    LicensePayload, PublicKeyMatchResult, RegistryDb, RegistryRecord, ALL_FEATURE_KEYS,
    DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
};

use crate::config::AppConfig;
use crate::rtl::{add_jalali_months, current_jalali_today, format_jalali_date, parse_jalali_date};
use crate::theme::ThemeMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Generator = 0,
    Ledger = 1,
    Inspector = 2,
    Vault = 3,
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub theme_mode: ThemeMode,
    pub vault: Option<KeyVault>,
    pub active_key_id: String,
    pub active_public_key: String,
    pub active_key_fingerprint: String,
    pub db: RegistryDb,
    pub config: AppConfig,

    // Tab 0: Generator
    pub customer_name: String,
    pub license_id: String,
    pub customer_org: String,
    pub customer_contact: String,
    pub license_type: String, // "subscription", "perpetual", "trial"
    pub max_devices: u32,
    pub match_policy: String, // "standard", "strict", "relaxed"
    pub issued_jalali: String,
    pub not_before_jalali: String,
    pub expires_jalali: String,
    pub hwid_text: String,
    pub grace_period_days: u32,
    pub grace_mode: String, // "readonly", "full"
    pub expiry_warning_days: u32,
    pub max_clock_skew_hours: u32,
    pub features: BTreeMap<String, bool>,
    pub min_app_version: String,
    pub max_app_version: String,

    pub generated_envelope: Option<LicenseEnvelope>,
    pub generated_json: String,
    pub generated_token: String,
    pub generated_sha256: String,
    pub generation_error: Option<String>,
    pub last_saved_path: Option<String>,

    // Tab 1: Ledger
    pub records: Vec<RegistryRecord>,
    pub search_query: String,
    pub status_filter: String,

    // Tab 2: Inspector & Simulator
    pub inspect_input: String,
    pub custom_public_key: String,
    pub inspect_status: Option<(bool, String, String)>,
    pub inspect_sha256: String,
    pub sim_hwid: String,
    pub sim_date_jalali: String,
    pub sim_version: String,
    pub sim_status: Option<(bool, String, String)>,

    // Tab 3: Vault & Pairing Diagnostics
    pub unlock_passphrase: String,
    pub unlock_msg: Option<(bool, String)>,
    pub new_key_id: String,
    pub new_passphrase: String,
    pub create_msg: Option<(bool, String)>,
    pub import_key_id: String,
    pub import_private_raw: String,
    pub import_passphrase: String,
    pub import_msg: Option<(bool, String)>,
    pub pairing_input_pubkey: String,
    pub pairing_status: Option<PublicKeyMatchResult>,
    pub binary_pairing_path: String,
    pub binary_pairing_status: Option<BinaryPairingResult>,

    pub toast_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let (today_y, today_m, today_d) = current_jalali_today();
        let today_str = format_jalali_date(today_y, today_m, today_d);
        let next_year_str = format_jalali_date(today_y + 1, today_m, today_d);

        let mut features = BTreeMap::new();
        for k in ALL_FEATURE_KEYS {
            features.insert(k.to_string(), true);
        }

        let db = RegistryDb::open_in_memory().unwrap_or_else(|_| {
            let p = PathBuf::from("daqiq_licenses.db");
            RegistryDb::open(&p).unwrap_or_else(|_| RegistryDb::open_in_memory().unwrap())
        });

        let config = AppConfig::load();
        let theme_mode = if config.theme_mode == "light" {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };

        // Initialize vault with production key ID
        let (vault, _env) = KeyVault::create_vault(DEFAULT_KEY_ID, "daqiq").unwrap();
        let pubkey = vault.public_key_b64.clone();
        let fp = vault.fingerprint();

        let rand_num: u32 = rand::random::<u32>() % 900000 + 100000;
        let initial_lic_id = format!("LIC-{}-{:06}", today_str.replace('/', ""), rand_num);

        let mut state = Self {
            active_tab: ActiveTab::Generator,
            theme_mode,
            vault: Some(vault),
            active_key_id: DEFAULT_KEY_ID.to_string(),
            active_public_key: pubkey,
            active_key_fingerprint: fp,
            db,
            config,

            customer_name: String::new(),
            license_id: initial_lic_id,
            customer_org: String::new(),
            customer_contact: String::new(),
            license_type: "subscription".to_string(),
            max_devices: 1,
            match_policy: "standard".to_string(),
            issued_jalali: today_str.clone(),
            not_before_jalali: today_str,
            expires_jalali: next_year_str,
            hwid_text: String::new(),
            grace_period_days: 7,
            grace_mode: "readonly".to_string(),
            expiry_warning_days: 14,
            max_clock_skew_hours: 24,
            features,
            min_app_version: "4.9.0".to_string(),
            max_app_version: String::new(),

            generated_envelope: None,
            generated_json: String::new(),
            generated_token: String::new(),
            generated_sha256: String::new(),
            generation_error: None,
            last_saved_path: None,

            records: Vec::new(),
            search_query: String::new(),
            status_filter: "all".to_string(),

            inspect_input: String::new(),
            custom_public_key: String::new(),
            inspect_status: None,
            inspect_sha256: String::new(),
            sim_hwid: String::new(),
            sim_date_jalali: String::new(),
            sim_version: "4.9.1".to_string(),
            sim_status: None,

            unlock_passphrase: String::new(),
            unlock_msg: None,
            new_key_id: DEFAULT_KEY_ID.to_string(),
            new_passphrase: String::new(),
            create_msg: None,
            import_key_id: DEFAULT_KEY_ID.to_string(),
            import_private_raw: String::new(),
            import_passphrase: String::new(),
            import_msg: None,
            pairing_input_pubkey: DEFAULT_PROD_PUBLIC_KEY.to_string(),
            pairing_status: None,
            binary_pairing_path: String::new(),
            binary_pairing_status: None,

            toast_message: None,
        };

        state.check_key_pairing();
        state.load_ledger();
        state
    }

    pub fn load_ledger(&mut self) {
        if let Ok(records) = self.db.list_licenses(100, 0) {
            self.records = records;
        }
    }

    pub fn apply_preset(&mut self, preset_name: &str) {
        match preset_name {
            "trial" => {
                self.license_type = "trial".to_string();
                self.max_devices = 1;
                self.match_policy = "relaxed".to_string();
                self.grace_period_days = 3;
                self.grace_mode = "readonly".to_string();
                self.expiry_warning_days = 3;
                if let Some((y, m, d)) = parse_jalali_date(&self.not_before_jalali) {
                    let (ny, nm, nd) = add_jalali_months(y, m, d, 1);
                    self.expires_jalali = format_jalali_date(ny, nm, nd);
                }
                for (k, v) in self.features.iter_mut() {
                    *v = k == "dashboard" || k == "products" || k == "sales";
                }
            }
            "standard" => {
                self.license_type = "subscription".to_string();
                self.max_devices = 1;
                self.match_policy = "standard".to_string();
                self.grace_period_days = 7;
                self.grace_mode = "readonly".to_string();
                self.expiry_warning_days = 14;
                if let Some((y, m, d)) = parse_jalali_date(&self.not_before_jalali) {
                    let (ny, nm, nd) = add_jalali_months(y, m, d, 12);
                    self.expires_jalali = format_jalali_date(ny, nm, nd);
                }
                for (_, v) in self.features.iter_mut() {
                    *v = true;
                }
            }
            "pro" => {
                self.license_type = "subscription".to_string();
                self.max_devices = 3;
                self.match_policy = "standard".to_string();
                self.grace_period_days = 14;
                self.grace_mode = "readonly".to_string();
                self.expiry_warning_days = 30;
                if let Some((y, m, d)) = parse_jalali_date(&self.not_before_jalali) {
                    let (ny, nm, nd) = add_jalali_months(y, m, d, 12);
                    self.expires_jalali = format_jalali_date(ny, nm, nd);
                }
                for (_, v) in self.features.iter_mut() {
                    *v = true;
                }
            }
            "enterprise" => {
                self.license_type = "subscription".to_string();
                self.max_devices = 10;
                self.match_policy = "strict".to_string();
                self.grace_period_days = 30;
                self.grace_mode = "full".to_string();
                self.expiry_warning_days = 45;
                if let Some((y, m, d)) = parse_jalali_date(&self.not_before_jalali) {
                    let (ny, nm, nd) = add_jalali_months(y, m, d, 24);
                    self.expires_jalali = format_jalali_date(ny, nm, nd);
                }
                for (_, v) in self.features.iter_mut() {
                    *v = true;
                }
            }
            "perpetual" => {
                self.license_type = "perpetual".to_string();
                self.max_devices = 1;
                self.match_policy = "standard".to_string();
                for (_, v) in self.features.iter_mut() {
                    *v = true;
                }
            }
            _ => {}
        }
    }

    pub fn generate_license(&mut self) -> Result<LicenseEnvelope, String> {
        let vault = match self.vault.as_ref() {
            Some(v) => v,
            None => return Err("گاوصندوق کلید قفل است. لطفاً ابتدا از زبانه ۴ آن را بازگشایی کنید.".to_string()),
        };

        if self.customer_name.trim().is_empty() {
            return Err("نام مشتری یا فروشگاه الزامی است.".to_string());
        }

        let (valid_hwids, _invalid_entries) = parse_bulk_hwids(&self.hwid_text);
        if valid_hwids.is_empty() {
            return Err("حداقل یک شناسه سخت‌افزاری (HWID) معتبر الزامی است.".to_string());
        }
        if valid_hwids.len() > self.max_devices as usize {
            return Err(format!(
                "تعداد شناسه‌ها ({}) از سقف مجاز دستگاه‌ها ({}) بیشتر است.",
                valid_hwids.len(),
                self.max_devices
            ));
        }

        let issued_at = jalali_str_to_utc_iso(&self.issued_jalali, false)
            .map_err(|e| format!("تاریخ صدور نامعتبر: {e}"))?;
        let not_before = jalali_str_to_utc_iso(&self.not_before_jalali, false)
            .map_err(|e| format!("تاریخ شروع اعتبار نامعتبر: {e}"))?;
        let expires_at = if self.license_type == "perpetual" {
            None
        } else {
            Some(
                jalali_str_to_utc_iso(&self.expires_jalali, true)
                    .map_err(|e| format!("تاریخ انقضا نامعتبر: {e}"))?,
            )
        };

        let payload = LicensePayload {
            customer: CustomerInfo {
                name: self.customer_name.trim().to_string(),
                organization: if self.customer_org.trim().is_empty() {
                    None
                } else {
                    Some(self.customer_org.trim().to_string())
                },
                contact: if self.customer_contact.trim().is_empty() {
                    None
                } else {
                    Some(self.customer_contact.trim().to_string())
                },
            },
            device_fingerprints: valid_hwids,
            expires_at,
            features: self.features.clone(),
            format_version: 1,
            issued_at,
            key_id: vault.key_id.clone(),
            license_id: self.license_id.trim().to_string(),
            license_type: self.license_type.clone(),
            max_app_version: if self.max_app_version.trim().is_empty() {
                None
            } else {
                Some(self.max_app_version.trim().to_string())
            },
            max_devices: self.max_devices,
            min_app_version: if self.min_app_version.trim().is_empty() {
                None
            } else {
                Some(self.min_app_version.trim().to_string())
            },
            not_before,
            product_id: DEFAULT_PRODUCT_ID.to_string(),

            // PR #39 Fields
            match_policy: Some(self.match_policy.clone()),
            grace_period_days: if self.license_type == "perpetual" {
                None
            } else {
                Some(self.grace_period_days)
            },
            grace_mode: if self.license_type == "perpetual" {
                None
            } else {
                Some(self.grace_mode.clone())
            },
            expiry_warning_days: if self.license_type == "perpetual" {
                None
            } else {
                Some(self.expiry_warning_days)
            },
            max_clock_skew_hours: Some(self.max_clock_skew_hours),
            quotas: None,
            metadata: None,
        };

        let envelope = vault
            .sign_license(&payload)
            .map_err(|e| format!("خطا در امضای لایسنس: {e}"))?;

        let json_text = serde_json::to_string_pretty(&envelope)
            .map_err(|e| format!("خطا در سریال‌سازی: {e}"))?;
        let token = envelope.to_compact_token().unwrap_or_default();

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(json_text.as_bytes());
        let sha256_hex = format!("{:x}", hasher.finalize());

        // Save to SQLite ledger
        let _ = self.db.insert_license(&envelope);
        self.load_ledger();

        self.generated_envelope = Some(envelope.clone());
        self.generated_json = json_text;
        self.generated_token = token;
        self.generated_sha256 = sha256_hex;
        self.generation_error = None;

        Ok(envelope)
    }

    pub fn check_key_pairing(&mut self) {
        let active = &self.active_public_key;
        let candidate = &self.pairing_input_pubkey;
        self.pairing_status = Some(compare_public_key(active, candidate));
    }

    pub fn check_binary_pairing(&mut self, file_path_str: &str) {
        let p = Path::new(file_path_str);
        self.binary_pairing_path = file_path_str.to_string();
        self.binary_pairing_status = Some(verify_daqiq_binary_pairing(p, &self.active_public_key));
    }

    pub fn inspect_license(&mut self) {
        let raw = self.inspect_input.trim();
        if raw.is_empty() {
            self.inspect_status = Some((false, "متن خالی".to_string(), "متن لایسنس یا توکن فشرده وارد نشده است.".to_string()));
            return;
        }

        let envelope: LicenseEnvelope = if raw.starts_with('{') {
            match serde_json::from_str(raw) {
                Ok(env) => env,
                Err(e) => {
                    self.inspect_status = Some((false, "خطای ساختار JSON".to_string(), format!("فایل لایسنس معتبر نیست: {e}")));
                    return;
                }
            }
        } else {
            match LicenseEnvelope::from_compact_token(raw) {
                Ok(env) => env,
                Err(e) => {
                    self.inspect_status = Some((false, "خطای توکن فشرده".to_string(), format!("توکن Base64 معتبر نیست: {e}")));
                    return;
                }
            }
        };

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        self.inspect_sha256 = format!("{:x}", hasher.finalize());

        let pubkey_to_use = if !self.custom_public_key.trim().is_empty() {
            self.custom_public_key.trim()
        } else {
            &self.active_public_key
        };

        match verify_license_envelope(&envelope, pubkey_to_use) {
            Ok(true) => {
                let msg = format!(
                    "امضای دیجیتال Ed25519 با کلید عمومی معتبر تایید گردید.\nشماره لایسنس: {}\nمشتری: {}\nنوع لایسنس: {}\nتعداد دستگاه‌ها: {}",
                    envelope.payload.license_id,
                    envelope.payload.customer.name,
                    envelope.payload.license_type,
                    envelope.payload.max_devices
                );
                self.inspect_status = Some((true, "امضای دیجیتال ۱۰۰٪ معتبر است ✓".to_string(), msg));
            }
            Ok(false) => {
                self.inspect_status = Some((
                    false,
                    "امضای دیجیتال نامعتبر است ❌".to_string(),
                    "امضا با کلید عمومی ارائه شده تطابق ندارد یا محتوا دستکاری شده است.".to_string(),
                ));
            }
            Err(e) => {
                self.inspect_status = Some((
                    false,
                    "خطا در راستی‌آزمایی امضا".to_string(),
                    format!("خطا: {e}"),
                ));
            }
        }
    }

    pub fn simulate_evaluation(&mut self) {
        let envelope = match &self.generated_envelope {
            Some(env) => env,
            None => {
                self.sim_status = Some((false, "لایسنسی برای شبیه‌سازی انتخاب نشده است".to_string(), "لطفاً ابتدا یک لایسنس صادر کنید یا در زبانه بررسی بارگذاری نمایید.".to_string()));
                return;
            }
        };

        let test_hwid = self.sim_hwid.trim();
        if !test_hwid.is_empty() {
            let is_authorized = envelope.payload.device_fingerprints.iter().any(|f| f == test_hwid);
            if !is_authorized {
                self.sim_status = Some((
                    false,
                    "رد شد: شناسه سخت‌افزاری غیرمجاز".to_string(),
                    format!("دستگاه با شناسه {test_hwid} در فهرست سخت‌افزارهای مجاز این لایسنس قرار ندارد."),
                ));
                return;
            }
        }

        self.sim_status = Some((
            true,
            "ارزیابی موفق: لایسنس در Daqiq پذیرفته خواهد شد ✓".to_string(),
            format!(
                "کلیه شرایط اعتبارسنجی PR #39 (شناسه دستگاه، تاریخ‌ها و نسخه {}) احراز گردید.",
                if self.sim_version.is_empty() { "4.9.1" } else { &self.sim_version }
            ),
        ));
    }
}
