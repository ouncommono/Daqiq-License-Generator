use std::collections::BTreeMap;
use std::path::PathBuf;
use sha2::{Digest, Sha256};

use daqiq_lic_core::{
    canonicalize_payload, clean_hwid_input, compare_public_key,
    parse_bulk_hwids, parse_daqreq, verify_license_envelope, CustomerInfo, KeyVault,
    LicenseEnvelope, LicensePayload, PublicKeyMatchResult, RegistryDb, RegistryRecord,
    ALL_FEATURE_KEYS, DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
};

use crate::rtl::{
    add_jalali_months, current_jalali_today, format_jalali_date, jalali_to_utc_iso_string,
    parse_jalali_date,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Generator,
    Ledger,
    Inspector,
    Vault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Trial,
    Standard,
    Pro,
    Enterprise,
    Perpetual,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(ActiveTab),
    ToggleTheme,

    // Generator inputs
    CustomerNameChanged(String),
    LicenseIdChanged(String),
    RegenerateLicenseId,
    CustomerOrgChanged(String),
    CustomerContactChanged(String),
    LicenseTypeSelected(String),
    MaxDevicesChanged(String),
    IssuedJalaliChanged(String),
    NotBeforeJalaliChanged(String),
    ExpiresJalaliChanged(String),
    QuickDurationMonths(i32),
    ApplyPreset(Preset),
    HwidTextChanged(String),
    PasteHwidFromClipboard,
    ImportDaqreqFile,
    MatchPolicyChanged(String),
    GracePeriodDaysChanged(String),
    GraceModeChanged(String),
    ExpiryWarningDaysChanged(String),
    MaxClockSkewHoursChanged(String),
    FeatureToggled(String, bool),
    SelectAllFeatures,
    DeselectAllFeatures,
    MinVersionChanged(String),
    MaxVersionChanged(String),
    ToggleJsonPreview,
    GenerateLicenseClicked,
    CopyGeneratedLicense,
    CopyCompactToken,
    SaveGeneratedLicense,

    // Ledger actions
    LedgerSearchChanged(String),
    RefreshLedger,
    ExportLedgerCsv,
    CloneAndRenew(String),

    // Inspector & Simulator
    InspectTextChanged(String),
    InspectFilePicked,
    CustomPubkeyChanged(String),
    VerifyInspectClicked,
    SimHwidChanged(String),
    SimDateChanged(String),
    SimVersionChanged(String),
    RunSimulationClicked,

    // Vault actions & Pairing Checker
    UnlockPassphraseChanged(String),
    UnlockVaultClicked,
    LockVaultClicked,
    NewKeyIdChanged(String),
    NewPassphraseChanged(String),
    CreateVaultClicked,
    CopyPublicKeyForDaqiq,
    PairingInputPubkeyChanged(String),
    CheckPairingClicked,

    // Toast
    DismissToast,
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub theme_mode: ThemeMode,
    pub vault: Option<KeyVault>,
    pub active_key_id: String,
    pub active_public_key: String,
    pub active_key_fingerprint: String,
    pub db: RegistryDb,

    // Tab 1: Generator
    pub customer_name: String,
    pub license_id: String,
    pub customer_org: String,
    pub customer_contact: String,
    pub license_type: String, // "perpetual", "subscription", or "trial"
    pub max_devices: u32,
    pub issued_jalali: String,
    pub not_before_jalali: String,
    pub expires_jalali: String,
    pub hwid_text: String,
    pub match_policy: String, // "standard", "strict", "relaxed"
    pub grace_period_days: u32,
    pub grace_mode: String, // "readonly", "full"
    pub expiry_warning_days: u32,
    pub max_clock_skew_hours: u32,
    pub features: BTreeMap<String, bool>,
    pub min_app_version: String,
    pub max_app_version: String,

    pub generated_envelope: Option<LicenseEnvelope>,
    pub generated_json: String,
    pub generated_compact_token: String,
    pub generated_sha256: String,
    pub generation_error: Option<String>,
    pub show_json_preview: bool,

    // Pre-computed Cached Previews (Zero-Work view() Rule)
    pub cached_hwid_valid_count: usize,
    pub cached_hwid_has_error: bool,
    pub cached_hwid_badge: String,
    pub cached_jalali_summary: String,
    pub cached_validation_status_text: String,
    pub cached_validation_is_ok: bool,

    // Tab 2: Ledger
    pub records: Vec<RegistryRecord>,
    pub ledger_search: String,
    pub ledger_error: Option<String>,

    // Tab 3: Inspector & Simulator
    pub inspect_text: String,
    pub custom_public_key: String,
    pub inspect_status: Option<(bool, String, String)>,
    pub inspect_sha256: String,
    pub sim_hwid: String,
    pub sim_date_jalali: String,
    pub sim_version: String,
    pub sim_status: Option<(bool, String, String)>,

    // Tab 4: Vault & Pairing Checker
    pub unlock_passphrase: String,
    pub unlock_status_msg: Option<(bool, String)>,
    pub new_key_id: String,
    pub new_passphrase: String,
    pub create_status_msg: Option<(bool, String)>,
    pub pairing_input_pubkey: String,
    pub pairing_status: Option<PublicKeyMatchResult>,

    // Notification toast
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

        // Initialize vault with production key ID
        let (vault, _env) = KeyVault::create_vault(DEFAULT_KEY_ID, "daqiq").unwrap();
        let pubkey = vault.public_key_b64.clone();
        let fp = vault.fingerprint();

        let rand_num: u32 = rand::random::<u32>() % 900000 + 100000;
        let initial_lic_id = format!("LIC-{}-{:06}", today_str.replace('/', ""), rand_num);

        let mut state = Self {
            active_tab: ActiveTab::Generator,
            theme_mode: ThemeMode::Dark,
            vault: Some(vault),
            active_key_id: DEFAULT_KEY_ID.to_string(),
            active_public_key: pubkey,
            active_key_fingerprint: fp,
            db,

            customer_name: String::new(),
            license_id: initial_lic_id,
            customer_org: String::new(),
            customer_contact: String::new(),
            license_type: "subscription".to_string(),
            max_devices: 1,
            issued_jalali: today_str.clone(),
            not_before_jalali: today_str,
            expires_jalali: next_year_str,
            hwid_text: String::new(),
            match_policy: "standard".to_string(),
            grace_period_days: 7,
            grace_mode: "readonly".to_string(),
            expiry_warning_days: 14,
            max_clock_skew_hours: 24,
            features,
            min_app_version: "4.9.0".to_string(),
            max_app_version: String::new(),

            generated_envelope: None,
            generated_json: String::new(),
            generated_compact_token: String::new(),
            generated_sha256: String::new(),
            generation_error: None,
            show_json_preview: false,

            cached_hwid_valid_count: 0,
            cached_hwid_has_error: false,
            cached_hwid_badge: "شناسه‌ای وارد نشده است".to_string(),
            cached_jalali_summary: String::new(),
            cached_validation_status_text: String::new(),
            cached_validation_is_ok: false,

            records: Vec::new(),
            ledger_search: String::new(),
            ledger_error: None,

            inspect_text: String::new(),
            custom_public_key: String::new(),
            inspect_status: None,
            inspect_sha256: String::new(),
            sim_hwid: String::new(),
            sim_date_jalali: String::new(),
            sim_version: "4.9.1".to_string(),
            sim_status: None,

            unlock_passphrase: String::new(),
            unlock_status_msg: None,
            new_key_id: "daqiq-prod-v1".to_string(),
            new_passphrase: String::new(),
            create_status_msg: None,
            pairing_input_pubkey: DEFAULT_PROD_PUBLIC_KEY.to_string(),
            pairing_status: None,

            toast_message: None,
        };

        state.refresh_ledger();
        state.recompute_cached_previews();
        state.check_key_pairing();
        state
    }

    pub fn recompute_cached_previews(&mut self) {
        // 1. HWID Analysis
        let (valid, invalid) = parse_bulk_hwids(&self.hwid_text);
        self.cached_hwid_valid_count = valid.len();
        self.cached_hwid_has_error = !invalid.is_empty();

        if self.hwid_text.trim().is_empty() {
            self.cached_hwid_badge = "شناسه‌ای وارد نشده است (حداقل ۱ دستگاه الزامی است)".to_string();
        } else if !invalid.is_empty() {
            self.cached_hwid_badge = format!(
                "⚠️ {} شناسه معتبر | {} ورودی نامعتبر!",
                valid.len(),
                invalid.len()
            );
        } else {
            self.cached_hwid_badge = format!(
                "✓ {} شناسه معتبر آماده قفل سخت‌افزاری (سقف: {})",
                valid.len(),
                self.max_devices
            );
        }

        // 2. Jalali Summary
        if self.license_type == "perpetual" {
            self.cached_jalali_summary = format!(
                "شروع: {} | اعتبار: دائمی (مادام‌العمر)",
                self.not_before_jalali
            );
        } else if self.license_type == "trial" {
            self.cached_jalali_summary = format!(
                "شروع: {} | انقضای آزمایشی: {} (مهلت: {} روز)",
                self.not_before_jalali, self.expires_jalali, self.grace_period_days
            );
        } else {
            self.cached_jalali_summary = format!(
                "شروع: {} | انقضا: {} (مهلت: {} روز)",
                self.not_before_jalali, self.expires_jalali, self.grace_period_days
            );
        }

        // 3. Validation Summary
        let mut issues = Vec::new();
        if self.customer_name.trim().is_empty() {
            issues.push("نام مشتری وارد نشده");
        }
        if self.cached_hwid_valid_count == 0 {
            issues.push("شناسه سخت‌افزاری ندارد");
        }
        if self.cached_hwid_valid_count > self.max_devices as usize {
            issues.push("تعداد شناسه بیش از سقف دستگاه است");
        }
        if self.vault.is_none() {
            issues.push("مخزن کلید قفل است");
        }

        if issues.is_empty() {
            self.cached_validation_is_ok = true;
            self.cached_validation_status_text = "آماده صدور و امضای دیجیتال".to_string();
        } else {
            self.cached_validation_is_ok = false;
            self.cached_validation_status_text = format!("نیاز به تکمیل: {}", issues.join("، "));
        }
    }

    pub fn check_key_pairing(&mut self) {
        if self.pairing_input_pubkey.trim().is_empty() {
            self.pairing_status = None;
        } else {
            self.pairing_status = Some(compare_public_key(
                &self.active_public_key,
                &self.pairing_input_pubkey,
            ));
        }
    }

    pub fn refresh_ledger(&mut self) {
        match self.db.list_licenses(100, 0) {
            Ok(list) => self.records = list,
            Err(e) => self.ledger_error = Some(e.to_string()),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SwitchTab(tab) => {
                self.active_tab = tab;
                if tab == ActiveTab::Ledger {
                    self.refresh_ledger();
                }
            }
            Message::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
            }

            // Generator inputs
            Message::CustomerNameChanged(s) => {
                self.customer_name = s;
                self.recompute_cached_previews();
            }
            Message::LicenseIdChanged(s) => {
                self.license_id = s;
                self.recompute_cached_previews();
            }
            Message::RegenerateLicenseId => {
                let (y, m, d) = current_jalali_today();
                let today_str = format_jalali_date(y, m, d);
                let rand_num: u32 = rand::random::<u32>() % 900000 + 100000;
                self.license_id = format!("LIC-{}-{:06}", today_str.replace('/', ""), rand_num);
                self.recompute_cached_previews();
            }
            Message::CustomerOrgChanged(s) => self.customer_org = s,
            Message::CustomerContactChanged(s) => self.customer_contact = s,
            Message::LicenseTypeSelected(t) => {
                self.license_type = t;
                self.recompute_cached_previews();
            }
            Message::MaxDevicesChanged(s) => {
                if let Ok(n) = s.parse::<u32>() {
                    if n >= 1 {
                        self.max_devices = n;
                        self.recompute_cached_previews();
                    }
                }
            }
            Message::IssuedJalaliChanged(s) => {
                self.issued_jalali = s;
                self.recompute_cached_previews();
            }
            Message::NotBeforeJalaliChanged(s) => {
                self.not_before_jalali = s;
                self.recompute_cached_previews();
            }
            Message::ExpiresJalaliChanged(s) => {
                self.expires_jalali = s;
                self.recompute_cached_previews();
            }
            Message::QuickDurationMonths(months) => {
                if let Some((y, m, d)) = parse_jalali_date(&self.not_before_jalali) {
                    if months == -1 {
                        // Perpetual
                        self.license_type = "perpetual".to_string();
                    } else {
                        if self.license_type == "perpetual" {
                            self.license_type = "subscription".to_string();
                        }
                        let (ny, nm, nd) = add_jalali_months(y, m, d, months);
                        self.expires_jalali = format_jalali_date(ny, nm, nd);
                    }
                    self.recompute_cached_previews();
                }
            }
            Message::ApplyPreset(preset) => {
                match preset {
                    Preset::Trial => {
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
                    Preset::Standard => {
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
                    Preset::Pro => {
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
                    Preset::Enterprise => {
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
                    Preset::Perpetual => {
                        self.license_type = "perpetual".to_string();
                        self.max_devices = 1;
                        self.match_policy = "standard".to_string();
                        for (_, v) in self.features.iter_mut() {
                            *v = true;
                        }
                    }
                }
                self.recompute_cached_previews();
            }
            Message::HwidTextChanged(s) => {
                self.hwid_text = s;
                self.recompute_cached_previews();
            }
            Message::PasteHwidFromClipboard => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            if self.hwid_text.trim().is_empty() {
                                self.hwid_text = trimmed.to_string();
                            } else {
                                self.hwid_text.push('\n');
                                self.hwid_text.push_str(trimmed);
                            }
                            self.recompute_cached_previews();
                            self.toast_message = Some("شناسه سخت‌افزاری از کلیپ‌بورد الصاق شد".to_string());
                        }
                    }
                }
            }
            Message::ImportDaqreqFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Daqiq Device Request", &["daqreq", "json"])
                    .pick_file()
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        match parse_daqreq(&content) {
                            Ok(req) => {
                                let fp = req.device_fingerprint;
                                if self.hwid_text.trim().is_empty() {
                                    self.hwid_text = fp;
                                } else {
                                    self.hwid_text.push('\n');
                                    self.hwid_text.push_str(&fp);
                                }
                                if let Some(label) = req.device_label {
                                    if self.customer_org.trim().is_empty() {
                                        self.customer_org = label;
                                    }
                                }
                                self.recompute_cached_previews();
                                self.toast_message = Some("درخواست سخت‌افزار (.daqreq) با موفقیت وارد شد".to_string());
                            }
                            Err(e) => {
                                self.toast_message = Some(format!("خطا در وارد کردن درخواست: {e}"));
                            }
                        }
                    }
                }
            }
            Message::MatchPolicyChanged(p) => self.match_policy = p,
            Message::GracePeriodDaysChanged(s) => {
                if let Ok(n) = s.parse::<u32>() {
                    self.grace_period_days = n;
                    self.recompute_cached_previews();
                }
            }
            Message::GraceModeChanged(m) => self.grace_mode = m,
            Message::ExpiryWarningDaysChanged(s) => {
                if let Ok(n) = s.parse::<u32>() {
                    self.expiry_warning_days = n;
                }
            }
            Message::MaxClockSkewHoursChanged(s) => {
                if let Ok(n) = s.parse::<u32>() {
                    self.max_clock_skew_hours = n;
                }
            }
            Message::FeatureToggled(k, v) => {
                self.features.insert(k, v);
            }
            Message::SelectAllFeatures => {
                for (_, v) in self.features.iter_mut() {
                    *v = true;
                }
            }
            Message::DeselectAllFeatures => {
                for (_, v) in self.features.iter_mut() {
                    *v = false;
                }
            }
            Message::MinVersionChanged(v) => self.min_app_version = v,
            Message::MaxVersionChanged(v) => self.max_app_version = v,
            Message::ToggleJsonPreview => {
                self.show_json_preview = !self.show_json_preview;
            }

            Message::GenerateLicenseClicked => self.generate_license(),

            Message::CopyGeneratedLicense => {
                if !self.generated_json.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&self.generated_json);
                        self.toast_message = Some("فایل کامل لایسنس (JSON) به کلیپ‌بورد کپی شد".to_string());
                    }
                }
            }
            Message::CopyCompactToken => {
                if !self.generated_compact_token.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&self.generated_compact_token);
                        self.toast_message = Some("توکن متنی لایسنس (Base64) به کلیپ‌بورد کپی شد".to_string());
                    }
                }
            }
            Message::SaveGeneratedLicense => {
                if !self.generated_json.is_empty() {
                    let file_name = format!("{}.daqlic", self.license_id.trim());
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Daqiq License", &["daqlic"])
                        .set_file_name(&file_name)
                        .save_file()
                    {
                        if std::fs::write(&path, &self.generated_json).is_ok() {
                            self.toast_message = Some(format!(
                                "فایل لایسنس با موفقیت در {} ذخیره شد",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }
                }
            }

            // Ledger
            Message::LedgerSearchChanged(q) => {
                self.ledger_search = q.clone();
                if q.trim().is_empty() {
                    self.refresh_ledger();
                } else {
                    match self.db.search_licenses(&q) {
                        Ok(list) => self.records = list,
                        Err(e) => self.ledger_error = Some(e.to_string()),
                    }
                }
            }
            Message::RefreshLedger => self.refresh_ledger(),
            Message::ExportLedgerCsv => {
                match self.db.export_csv() {
                    Ok(csv_data) => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("CSV Spreadsheet", &["csv"])
                            .set_file_name("daqiq_sales_ledger.csv")
                            .save_file()
                        {
                            if std::fs::write(&path, csv_data).is_ok() {
                                self.toast_message = Some("خروجی اکسل/CSV (UTF-8 BOM) با موفقیت ذخیره شد".to_string());
                            }
                        }
                    }
                    Err(e) => self.toast_message = Some(format!("خطا در تهیه CSV: {e}")),
                }
            }
            Message::CloneAndRenew(lic_id) => {
                if let Some(rec) = self.records.iter().find(|r| r.license_id == lic_id) {
                    self.customer_name = rec.customer_name.clone();
                    self.customer_org = rec.customer_org.clone().unwrap_or_default();
                    self.customer_contact = rec.customer_contact.clone().unwrap_or_default();
                    self.license_type = rec.license_type.clone();
                    self.max_devices = rec.max_devices;
                    self.hwid_text = rec.device_fingerprints.replace(',', "\n");

                    let (y, m, d) = current_jalali_today();
                    self.issued_jalali = format_jalali_date(y, m, d);
                    self.not_before_jalali = format_jalali_date(y, m, d);
                    let (ny, nm, nd) = add_jalali_months(y, m, d, 12);
                    self.expires_jalali = format_jalali_date(ny, nm, nd);

                    let rand_num: u32 = rand::random::<u32>() % 900000 + 100000;
                    self.license_id = format!("LIC-{}-{:06}", self.issued_jalali.replace('/', ""), rand_num);

                    self.recompute_cached_previews();
                    self.active_tab = ActiveTab::Generator;
                    self.toast_message = Some(format!("مشخصات لایسنس {} برای تمدید بارگذاری شد", lic_id));
                }
            }

            // Inspector & Simulator
            Message::InspectTextChanged(s) => {
                self.inspect_text = s;
                self.inspect_status = None;
            }
            Message::InspectFilePicked => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Daqiq License", &["daqlic", "json", "txt"])
                    .pick_file()
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.inspect_text = content;
                        self.inspect_status = None;
                        self.verify_inspected_license();
                    }
                }
            }
            Message::CustomPubkeyChanged(k) => {
                self.custom_public_key = k;
            }
            Message::VerifyInspectClicked => self.verify_inspected_license(),
            Message::SimHwidChanged(h) => self.sim_hwid = h,
            Message::SimDateChanged(d) => self.sim_date_jalali = d,
            Message::SimVersionChanged(v) => self.sim_version = v,
            Message::RunSimulationClicked => self.run_simulation(),

            // Vault actions & Pairing Checker
            Message::UnlockPassphraseChanged(p) => self.unlock_passphrase = p,
            Message::UnlockVaultClicked => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Daqiq Vault", &["dqvault", "json"])
                    .pick_file()
                {
                    match KeyVault::load_from_file(&path, &self.unlock_passphrase) {
                        Ok(vault) => {
                            self.active_key_id = vault.key_id.clone();
                            self.active_public_key = vault.public_key_b64.clone();
                            self.active_key_fingerprint = vault.fingerprint();
                            self.vault = Some(vault);
                            self.unlock_status_msg = Some((true, "مخزن کلید با موفقیت باز شد!".to_string()));
                            self.check_key_pairing();
                            self.recompute_cached_previews();
                            self.toast_message = Some("مخزن کلید امضا باز شد".to_string());
                        }
                        Err(e) => {
                            self.unlock_status_msg = Some((false, format!("خطا در باز کردن مخزن: {e}")));
                        }
                    }
                }
            }
            Message::LockVaultClicked => {
                self.vault = None;
                self.recompute_cached_previews();
                self.toast_message = Some("مخزن کلیدها قفل شد".to_string());
            }
            Message::NewKeyIdChanged(id) => self.new_key_id = id,
            Message::NewPassphraseChanged(p) => self.new_passphrase = p,
            Message::CreateVaultClicked => {
                if self.new_key_id.trim().is_empty() {
                    self.create_status_msg = Some((false, "شناسه کلید الزامی است".to_string()));
                    return;
                }
                if self.new_passphrase.len() < 6 {
                    self.create_status_msg = Some((false, "رمز عبور باید حداقل ۶ نویسه باشد".to_string()));
                    return;
                }
                match KeyVault::create_vault(&self.new_key_id, &self.new_passphrase) {
                    Ok((vault, env)) => {
                        let file_name = format!("{}.dqvault", self.new_key_id.trim());
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Daqiq Vault", &["dqvault"])
                            .set_file_name(&file_name)
                            .save_file()
                        {
                            let json_text = serde_json::to_string_pretty(&env).unwrap_or_default();
                            let _ = std::fs::write(&path, json_text);
                        }
                        self.active_key_id = vault.key_id.clone();
                        self.active_public_key = vault.public_key_b64.clone();
                        self.active_key_fingerprint = vault.fingerprint();
                        self.vault = Some(vault);
                        self.create_status_msg = Some((true, "مخزن کلید جدید ساخته و بارگذاری شد!".to_string()));
                        self.check_key_pairing();
                        self.recompute_cached_previews();
                        self.toast_message = Some("مخزن کلید جدید ایجاد شد".to_string());
                    }
                    Err(e) => {
                        self.create_status_msg = Some((false, format!("خطا در ساخت مخزن: {e}")));
                    }
                }
            }
            Message::CopyPublicKeyForDaqiq => {
                let code_snippet = format!("\"{}\": \"{}\",", self.active_key_id, self.active_public_key);
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&code_snippet);
                    self.toast_message = Some("کد کلید عمومی برای Daqiq در کلیپ‌بورد کپی شد".to_string());
                }
            }
            Message::PairingInputPubkeyChanged(k) => {
                self.pairing_input_pubkey = k;
                self.check_key_pairing();
            }
            Message::CheckPairingClicked => {
                self.check_key_pairing();
            }

            Message::DismissToast => {
                self.toast_message = None;
            }
        }
    }

    fn generate_license(&mut self) {
        let vault = match self.vault.as_ref() {
            Some(v) => v,
            None => {
                self.generation_error = Some("مخزن کلید امضا قفل است. لطفاً ابتدا از زبانه «گاوصندوق» قفل آن را باز نمایید.".to_string());
                return;
            }
        };

        if self.customer_name.trim().is_empty() {
            self.generation_error = Some("نام مشتری / فروشگاه نمی‌تواند خالی باشد.".to_string());
            return;
        }

        let (valid_hwids, _invalid_entries) = parse_bulk_hwids(&self.hwid_text);
        if valid_hwids.is_empty() {
            self.generation_error = Some("حداقل یک شناسه سخت‌افزاری (HWID) معتبر باید وارد شود.".to_string());
            return;
        }
        if valid_hwids.len() > self.max_devices as usize {
            self.generation_error = Some(format!(
                "تعداد شناسه‌ها ({}) از سقف مجاز دستگاه‌ها ({}) بیشتر است.",
                valid_hwids.len(),
                self.max_devices
            ));
            return;
        }

        let issued_at = match jalali_to_utc_iso_string(&self.issued_jalali, false) {
            Ok(iso) => iso,
            Err(e) => {
                self.generation_error = Some(e);
                return;
            }
        };
        let not_before = match jalali_to_utc_iso_string(&self.not_before_jalali, false) {
            Ok(iso) => iso,
            Err(e) => {
                self.generation_error = Some(e);
                return;
            }
        };
        let expires_at = if self.license_type == "perpetual" {
            None
        } else {
            match jalali_to_utc_iso_string(&self.expires_jalali, true) {
                Ok(iso) => Some(iso),
                Err(e) => {
                    self.generation_error = Some(e);
                    return;
                }
            }
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

        match vault.sign_license(&payload) {
            Ok(envelope) => {
                let json_text = serde_json::to_string_pretty(&envelope).unwrap_or_default();
                let compact_token = envelope.to_compact_token().unwrap_or_default();
                let canon_bytes = canonicalize_payload(&envelope.payload).unwrap_or_default();
                let mut msg = Vec::new();
                msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
                msg.extend_from_slice(&canon_bytes);
                let mut hasher = Sha256::new();
                hasher.update(&msg);
                let digest = format!("{:x}", hasher.finalize());

                let _ = self.db.insert_license(&envelope);

                self.generated_envelope = Some(envelope);
                self.generated_json = json_text;
                self.generated_compact_token = compact_token;
                self.generated_sha256 = digest;
                self.generation_error = None;
                self.toast_message = Some("لایسنس دیجیتال با موفقیت تولید و امضا شد!".to_string());
            }
            Err(e) => {
                self.generation_error = Some(format!("خطا در صدور لایسنس: {e}"));
            }
        }
    }

    fn verify_inspected_license(&mut self) {
        let clean = self.inspect_text.trim();
        if clean.is_empty() {
            self.inspect_status = Some((false, "متن یا فایل لایسنس خالی است".to_string(), String::new()));
            return;
        }

        let envelope: LicenseEnvelope = match LicenseEnvelope::from_compact_token(clean) {
            Ok(env) => env,
            Err(e) => {
                self.inspect_status = Some((false, format!("خطا در خواندن ساختار لایسنس یا توکن: {e}"), String::new()));
                return;
            }
        };

        let pubkey_to_use = if !self.custom_public_key.trim().is_empty() {
            self.custom_public_key.trim().to_string()
        } else if let Some(v) = &self.vault {
            if v.key_id == envelope.key_id {
                v.public_key_b64.clone()
            } else {
                self.active_public_key.clone()
            }
        } else {
            self.active_public_key.clone()
        };

        match verify_license_envelope(&envelope, &pubkey_to_use) {
            Ok(valid) => {
                if valid {
                    let canon = canonicalize_payload(&envelope.payload).unwrap_or_default();
                    let mut msg = Vec::new();
                    msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
                    msg.extend_from_slice(&canon);
                    let mut hasher = Sha256::new();
                    hasher.update(&msg);
                    self.inspect_sha256 = format!("{:x}", hasher.finalize());

                    let info = format!(
                        "امضا معتبر است! خریدار: {} | نوع: {} | دستگاه‌ها: {}",
                        envelope.payload.customer.name,
                        envelope.payload.license_type,
                        envelope.payload.max_devices
                    );
                    self.inspect_status = Some((true, info, self.inspect_sha256.clone()));
                } else {
                    self.inspect_status = Some((
                        false,
                        "امضای دیجیتال لایسنس نامعتبر است یا با کلید عمومی تطابق ندارد.".to_string(),
                        String::new(),
                    ));
                }
            }
            Err(e) => {
                self.inspect_status = Some((false, format!("خطا در اعتبارسنجی: {e}"), String::new()));
            }
        }
    }

    fn run_simulation(&mut self) {
        let clean = self.inspect_text.trim();
        let envelope: LicenseEnvelope = match LicenseEnvelope::from_compact_token(clean) {
            Ok(env) => env,
            Err(_) => {
                self.sim_status = Some((false, "ابتدا یک لایسنس معتبر در بالا باز یا الصاق کنید.".to_string(), String::new()));
                return;
            }
        };

        let test_hwid = clean_hwid_input(&self.sim_hwid);
        if test_hwid.is_empty() {
            self.sim_status = Some((false, "شناسه سخت‌افزاری آزمایشی وارد نشده است.".to_string(), String::new()));
            return;
        }

        let is_hw_authorized = envelope
            .payload
            .device_fingerprints
            .iter()
            .any(|fp| clean_hwid_input(fp) == test_hwid);

        if !is_hw_authorized {
            self.sim_status = Some((
                false,
                "عدم تأیید: شناسه دستگاه در فهرست سخت‌افزارهای مجاز این لایسنس نیست.".to_string(),
                format!("شناسه بررسی شده: {test_hwid}"),
            ));
            return;
        }

        self.sim_status = Some((
            true,
            "تأیید کامل! نرم‌افزار دقیق v4.9.1 این لایسنس را بر روی این دستگاه می‌پذیرد.".to_string(),
            format!("شناسه منطبق: {test_hwid}"),
        ));
    }
}
