use std::collections::BTreeMap;
use std::path::PathBuf;
use chrono::Utc;
use sha2::{Digest, Sha256};

use daqiq_lic_core::{
    canonicalize_payload, clean_hwid_input, parse_bulk_hwids, parse_daqreq,
    verify_license_envelope, CustomerInfo, KeyVault, LicenseEnvelope,
    LicensePayload, RegistryDb, RegistryRecord, VaultEnvelope, ALL_FEATURE_KEYS,
    DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
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
    ImportDaqreqFile,
    FeatureToggled(String, bool),
    SelectAllFeatures,
    DeselectAllFeatures,
    MinVersionChanged(String),
    MaxVersionChanged(String),
    GenerateLicenseClicked,
    CopyGeneratedLicense,
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

    // Vault actions
    UnlockPassphraseChanged(String),
    UnlockVaultClicked,
    LockVaultClicked,
    NewKeyIdChanged(String),
    NewPassphraseChanged(String),
    CreateVaultClicked,
    CopyPublicKeyForDaqiq,

    // Toast
    DismissToast,
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub theme_mode: ThemeMode,
    pub vault: Option<KeyVault>,
    pub active_key_id: String,
    pub active_public_key: String,
    pub db: RegistryDb,

    // Tab 1: Generator
    pub customer_name: String,
    pub license_id: String,
    pub customer_org: String,
    pub customer_contact: String,
    pub license_type: String,
    pub max_devices: u32,
    pub issued_jalali: String,
    pub not_before_jalali: String,
    pub expires_jalali: String,
    pub hwid_text: String,
    pub hwid_valid_count: usize,
    pub hwid_error_msg: Option<String>,
    pub features: BTreeMap<String, bool>,
    pub min_app_version: String,
    pub max_app_version: String,
    pub generated_envelope: Option<LicenseEnvelope>,
    pub generated_json: String,
    pub generated_sha256: String,
    pub generation_error: Option<String>,

    // Tab 2: Ledger
    pub records: Vec<RegistryRecord>,
    pub ledger_search: String,
    pub ledger_error: Option<String>,

    // Tab 3: Inspector
    pub inspect_json: String,
    pub custom_public_key: String,
    pub inspect_status: Option<(bool, String, String)>,
    pub inspect_sha256: String,
    pub sim_hwid: String,
    pub sim_date_jalali: String,
    pub sim_version: String,
    pub sim_status: Option<(bool, String, String)>,

    // Tab 4: Vault
    pub unlock_passphrase: String,
    pub unlock_status_msg: Option<(bool, String)>,
    pub new_key_id: String,
    pub new_passphrase: String,
    pub create_status_msg: Option<(bool, String)>,

    // Notification toast
    pub toast_message: Option<String>,
}

pub fn get_app_data_db_path() -> PathBuf {
    let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("DaqiqLicenseGenerator");
    std::fs::create_dir_all(&dir).ok();
    dir.push("registry.sqlite3");
    dir
}

pub fn generate_random_license_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(1000..9999);
    let today = Utc::now().format("%Y%m%d");
    format!("LIC-{today}-{num}")
}

impl AppState {
    pub fn new() -> Self {
        let (jy, jm, jd) = current_jalali_today();
        let issued = format_jalali_date(jy, jm, jd);
        let not_before = issued.clone();
        let (ey, em, ed) = add_jalali_months(jy, jm, jd, 12);
        let expires = format_jalali_date(ey, em, ed);

        let mut features = BTreeMap::new();
        for k in ALL_FEATURE_KEYS {
            features.insert(k.to_string(), true);
        }

        let db_path = get_app_data_db_path();
        let db = match RegistryDb::open(&db_path) {
            Ok(d) => d,
            Err(_) => RegistryDb::open_in_memory().expect("In-memory SQLite fallback must succeed"),
        };

        let initial_records = db.list_licenses(100, 0).unwrap_or_default();

        Self {
            active_tab: ActiveTab::Generator,
            theme_mode: ThemeMode::Dark,
            vault: None,
            active_key_id: DEFAULT_KEY_ID.to_string(),
            active_public_key: DEFAULT_PROD_PUBLIC_KEY.to_string(),
            db,

            customer_name: String::new(),
            license_id: generate_random_license_id(),
            customer_org: String::new(),
            customer_contact: String::new(),
            license_type: "subscription".to_string(),
            max_devices: 1,
            issued_jalali: issued,
            not_before_jalali: not_before,
            expires_jalali: expires,
            hwid_text: String::new(),
            hwid_valid_count: 0,
            hwid_error_msg: None,
            features,
            min_app_version: "4.9.0".to_string(),
            max_app_version: String::new(),
            generated_envelope: None,
            generated_json: String::new(),
            generated_sha256: String::new(),
            generation_error: None,

            records: initial_records,
            ledger_search: String::new(),
            ledger_error: None,

            inspect_json: String::new(),
            custom_public_key: String::new(),
            inspect_status: None,
            inspect_sha256: String::new(),
            sim_hwid: String::new(),
            sim_date_jalali: format_jalali_date(jy, jm, jd),
            sim_version: "4.9.0".to_string(),
            sim_status: None,

            unlock_passphrase: String::new(),
            unlock_status_msg: None,
            new_key_id: "daqiq-custom-v1".to_string(),
            new_passphrase: String::new(),
            create_status_msg: None,

            toast_message: None,
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
            Message::CustomerNameChanged(s) => self.customer_name = s,
            Message::LicenseIdChanged(s) => self.license_id = s,
            Message::RegenerateLicenseId => self.license_id = generate_random_license_id(),
            Message::CustomerOrgChanged(s) => self.customer_org = s,
            Message::CustomerContactChanged(s) => self.customer_contact = s,
            Message::LicenseTypeSelected(t) => self.license_type = t,
            Message::MaxDevicesChanged(s) => {
                if let Ok(n) = s.parse::<u32>() {
                    self.max_devices = n.clamp(1, 100);
                }
            }
            Message::IssuedJalaliChanged(s) => self.issued_jalali = s,
            Message::NotBeforeJalaliChanged(s) => self.not_before_jalali = s,
            Message::ExpiresJalaliChanged(s) => self.expires_jalali = s,
            Message::QuickDurationMonths(m) => {
                if let Some((y, mo, d)) = parse_jalali_date(&self.not_before_jalali) {
                    let (ey, emo, ed) = add_jalali_months(y, mo, d, m);
                    self.expires_jalali = format_jalali_date(ey, emo, ed);
                    self.license_type = "subscription".to_string();
                }
            }
            Message::ApplyPreset(p) => match p {
                Preset::Trial => {
                    self.license_type = "subscription".to_string();
                    self.max_devices = 1;
                    if let Some((y, mo, d)) = parse_jalali_date(&self.issued_jalali) {
                        let (ey, emo, ed) = add_jalali_months(y, mo, d, 1);
                        self.expires_jalali = format_jalali_date(ey, emo, ed);
                    }
                    self.select_all_features();
                    self.toast_message = Some("پریست لایسنس آزمایشی (۱ ماه، ۱ دستگاه) اعمال شد".to_string());
                }
                Preset::Standard => {
                    self.license_type = "subscription".to_string();
                    self.max_devices = 1;
                    if let Some((y, mo, d)) = parse_jalali_date(&self.issued_jalali) {
                        let (ey, emo, ed) = add_jalali_months(y, mo, d, 12);
                        self.expires_jalali = format_jalali_date(ey, emo, ed);
                    }
                    self.select_all_features();
                    self.toast_message = Some("پریست لایسنس استاندارد (۱ سال، ۱ دستگاه) اعمال شد".to_string());
                }
                Preset::Pro => {
                    self.license_type = "subscription".to_string();
                    self.max_devices = 3;
                    if let Some((y, mo, d)) = parse_jalali_date(&self.issued_jalali) {
                        let (ey, emo, ed) = add_jalali_months(y, mo, d, 12);
                        self.expires_jalali = format_jalali_date(ey, emo, ed);
                    }
                    self.select_all_features();
                    self.toast_message = Some("پریست لایسنس حرفه‌ای (۱ سال، ۳ دستگاه) اعمال شد".to_string());
                }
                Preset::Enterprise => {
                    self.license_type = "subscription".to_string();
                    self.max_devices = 10;
                    if let Some((y, mo, d)) = parse_jalali_date(&self.issued_jalali) {
                        let (ey, emo, ed) = add_jalali_months(y, mo, d, 24);
                        self.expires_jalali = format_jalali_date(ey, emo, ed);
                    }
                    self.select_all_features();
                    self.toast_message = Some("پریست لایسنس سازمانی (۲ سال، ۱۰ دستگاه) اعمال شد".to_string());
                }
                Preset::Perpetual => {
                    self.license_type = "perpetual".to_string();
                    self.max_devices = 5;
                    self.select_all_features();
                    self.toast_message = Some("پریست لایسنس دائمی و مادام‌العمر اعمال شد".to_string());
                }
            },
            Message::HwidTextChanged(text) => {
                self.hwid_text = text.clone();
                let (valid, invalid) = parse_bulk_hwids(&text);
                self.hwid_valid_count = valid.len();
                if !invalid.is_empty() {
                    self.hwid_error_msg = Some(format!(
                        "توجه: {} شناسه ورودی نامعتبر تشخیص داده شد",
                        invalid.len()
                    ));
                } else {
                    self.hwid_error_msg = None;
                }
            }
            Message::ImportDaqreqFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Daqiq Request", &["daqreq", "json"])
                    .pick_file()
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        match parse_daqreq(&content) {
                            Ok(req) => {
                                let mut current = self.hwid_text.trim().to_string();
                                if !current.is_empty() {
                                    current.push('\n');
                                }
                                current.push_str(&req.device_fingerprint);
                                self.hwid_text = current.clone();
                                let (valid, _) = parse_bulk_hwids(&current);
                                self.hwid_valid_count = valid.len();
                                if self.customer_name.is_empty() {
                                    self.customer_name = req.device_label.unwrap_or_default();
                                }
                                self.toast_message = Some("شناسه سخت‌افزاری از فایل .daqreq اضافه شد".to_string());
                            }
                            Err(e) => {
                                self.toast_message = Some(format!("خطا در خواندن فایل درخواست: {e}"));
                            }
                        }
                    }
                }
            }
            Message::FeatureToggled(key, val) => {
                self.features.insert(key, val);
            }
            Message::SelectAllFeatures => self.select_all_features(),
            Message::DeselectAllFeatures => {
                for v in self.features.values_mut() {
                    *v = false;
                }
            }
            Message::MinVersionChanged(v) => self.min_app_version = v,
            Message::MaxVersionChanged(v) => self.max_app_version = v,
            Message::GenerateLicenseClicked => self.generate_license(),
            Message::CopyGeneratedLicense => {
                if !self.generated_json.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&self.generated_json);
                        self.toast_message = Some("محتوای لایسنس به کلیپ‌بورد کپی شد".to_string());
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
                                self.toast_message = Some("خروجی اکسل/CSV با موفقیت ذخیره شد".to_string());
                            }
                        }
                    }
                    Err(e) => self.toast_message = Some(format!("خطا در تهیه خروجی CSV: {e}")),
                }
            }
            Message::CloneAndRenew(lic_id) => {
                if let Some(rec) = self.records.iter().find(|r| r.license_id == lic_id).cloned() {
                    self.customer_name = rec.customer_name;
                    self.license_id = generate_random_license_id();
                    self.license_type = rec.license_type;
                    self.max_devices = rec.max_devices;
                    self.hwid_text = rec.device_fingerprints.replace(',', "\n");
                    let (valid, _) = parse_bulk_hwids(&self.hwid_text);
                    self.hwid_valid_count = valid.len();
                    let (jy, jm, jd) = current_jalali_today();
                    self.issued_jalali = format_jalali_date(jy, jm, jd);
                    self.not_before_jalali = self.issued_jalali.clone();
                    let (ey, em, ed) = add_jalali_months(jy, jm, jd, 12);
                    self.expires_jalali = format_jalali_date(ey, em, ed);
                    self.active_tab = ActiveTab::Generator;
                    self.toast_message = Some("اطلاعات لایسنس جهت تمدید و صدور جدید بارگذاری شد".to_string());
                }
            }

            Message::InspectTextChanged(s) => self.inspect_json = s,
            Message::InspectFilePicked => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Daqiq License", &["daqlic", "json"])
                    .pick_file()
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.inspect_json = content;
                        self.verify_inspected_license();
                    }
                }
            }
            Message::CustomPubkeyChanged(k) => self.custom_public_key = k,
            Message::VerifyInspectClicked => self.verify_inspected_license(),
            Message::SimHwidChanged(s) => self.sim_hwid = s,
            Message::SimDateChanged(s) => self.sim_date_jalali = s,
            Message::SimVersionChanged(s) => self.sim_version = s,
            Message::RunSimulationClicked => self.run_simulation(),

            Message::UnlockPassphraseChanged(p) => self.unlock_passphrase = p,
            Message::UnlockVaultClicked => self.unlock_vault(),
            Message::LockVaultClicked => {
                self.vault = None;
                self.active_key_id = DEFAULT_KEY_ID.to_string();
                self.active_public_key = DEFAULT_PROD_PUBLIC_KEY.to_string();
                self.unlock_status_msg = Some((true, "مخزن کلیدها قفل گردید".to_string()));
            }
            Message::NewKeyIdChanged(k) => self.new_key_id = k,
            Message::NewPassphraseChanged(p) => self.new_passphrase = p,
            Message::CreateVaultClicked => self.create_new_vault(),
            Message::CopyPublicKeyForDaqiq => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&self.active_public_key);
                    self.toast_message = Some("کلید عمومی جهت درج در نرم‌افزار دقیق کپی شد".to_string());
                }
            }
            Message::DismissToast => self.toast_message = None,
        }
    }

    fn select_all_features(&mut self) {
        for v in self.features.values_mut() {
            *v = true;
        }
    }

    fn refresh_ledger(&mut self) {
        match self.db.list_licenses(100, 0) {
            Ok(list) => {
                self.records = list;
                self.ledger_error = None;
            }
            Err(e) => self.ledger_error = Some(e.to_string()),
        }
    }

    fn generate_license(&mut self) {
        // 1. Vault check
        let vault = match self.vault.as_ref() {
            Some(v) => v,
            None => {
                self.generation_error = Some("مخزن کلید امضا قفل است. لطفاً ابتدا از زبانه «مخزن کلیدها» قفل آن را باز نمایید.".to_string());
                return;
            }
        };

        // 2. Validate customer name
        if self.customer_name.trim().is_empty() {
            self.generation_error = Some("نام مشتری / فروشگاه نمی‌تواند خالی باشد.".to_string());
            return;
        }

        // 3. Validate device fingerprints
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

        // 4. Convert Jalali dates to UTC ISO
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

        // 5. Construct payload
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
        };

        // 6. Sign license
        match vault.sign_license(&payload) {
            Ok(envelope) => {
                let json_text = serde_json::to_string_pretty(&envelope).unwrap_or_default();
                let canon_bytes = canonicalize_payload(&envelope.payload).unwrap_or_default();
                let mut msg = Vec::new();
                msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
                msg.extend_from_slice(&canon_bytes);
                let mut hasher = Sha256::new();
                hasher.update(&msg);
                let digest = format!("{:x}", hasher.finalize());

                // 7. Save to SQLite database
                let _ = self.db.insert_license(&envelope);

                self.generated_envelope = Some(envelope);
                self.generated_json = json_text;
                self.generated_sha256 = digest;
                self.generation_error = None;
                self.toast_message = Some("لایسنس دیجیتال با موفقیت تولید و امضا شد!".to_string());
            }
            Err(e) => {
                self.generation_error = Some(format!("خطا در امضای دیجیتال لایسنس: {e}"));
            }
        }
    }

    fn verify_inspected_license(&mut self) {
        if self.inspect_json.trim().is_empty() {
            self.inspect_status = Some((false, "فایل لایسنس خالی است".to_string(), "لطفاً محتوای فایل .daqlic را وارد نمایید.".to_string()));
            return;
        }

        let envelope: LicenseEnvelope = match serde_json::from_str(&self.inspect_json) {
            Ok(env) => env,
            Err(e) => {
                self.inspect_status = Some((false, "فرمت JSON نامعتبر است".to_string(), e.to_string()));
                return;
            }
        };

        let pubkey = if !self.custom_public_key.trim().is_empty() {
            self.custom_public_key.trim().to_string()
        } else {
            self.active_public_key.clone()
        };

        let canon_bytes = match canonicalize_payload(&envelope.payload) {
            Ok(b) => b,
            Err(e) => {
                self.inspect_status = Some((false, "خطا در فرمت کانونیکال".to_string(), e.to_string()));
                return;
            }
        };
        let mut msg = Vec::new();
        msg.extend_from_slice(daqiq_lic_core::DOMAIN_PREFIX);
        msg.extend_from_slice(&canon_bytes);
        let mut hasher = Sha256::new();
        hasher.update(&msg);
        self.inspect_sha256 = format!("{:x}", hasher.finalize());

        match verify_license_envelope(&envelope, &pubkey) {
            Ok(true) => {
                self.inspect_status = Some((
                    true,
                    "امضای دیجیتال کاملاً معتبر است".to_string(),
                    format!(
                        "خریدار: {} | شناسه لایسنس: {} | کلید امضا: {}",
                        envelope.payload.customer.name,
                        envelope.payload.license_id,
                        envelope.key_id
                    ),
                ));
            }
            Ok(false) | Err(daqiq_lic_core::LicError::InvalidSignature) => {
                self.inspect_status = Some((
                    false,
                    "امضای دیجیتال نامعتبر است یا فایل دستکاری شده است".to_string(),
                    "محتوای لایسنس با امضای ثبت شده تطابق ندارد.".to_string(),
                ));
            }
            Err(e) => {
                self.inspect_status = Some((false, "خطا در اعتبارسنجی".to_string(), e.to_string()));
            }
        }
    }

    fn run_simulation(&mut self) {
        if self.inspect_json.trim().is_empty() {
            self.sim_status = Some((false, "لایسنس بارگذاری نشده".to_string(), "ابتدا یک فایل لایسنس در بخش بازرس باز کنید.".to_string()));
            return;
        }

        let envelope: LicenseEnvelope = match serde_json::from_str(&self.inspect_json) {
            Ok(env) => env,
            Err(e) => {
                self.sim_status = Some((false, "فایل لایسنس نامعتبر".to_string(), e.to_string()));
                return;
            }
        };

        let pubkey = if !self.custom_public_key.trim().is_empty() {
            self.custom_public_key.trim().to_string()
        } else {
            self.active_public_key.clone()
        };

        // 1. Signature
        if verify_license_envelope(&envelope, &pubkey).is_err() {
            self.sim_status = Some((
                false,
                "امضای دیجیتال نامعتبر".to_string(),
                "لایسنس به دلیل عدم تطابق امضای دیجیتال رد می‌شود.".to_string(),
            ));
            return;
        }

        let p = &envelope.payload;

        // 2. Hardware check
        if !self.sim_hwid.trim().is_empty() {
            let clean = clean_hwid_input(&self.sim_hwid);
            if !p.device_fingerprints.contains(&clean) {
                self.sim_status = Some((
                    false,
                    "دستگاه غیرمجاز (HWID نامطابق)".to_string(),
                    format!("شناسه سخت‌افزاری ({clean}) در لیست دستگاه‌های مجاز این لایسنس وجود ندارد."),
                ));
                return;
            }
        }

        // 3. Time check
        let sim_date_utc = match jalali_to_utc_iso_string(&self.sim_date_jalali, false) {
            Ok(iso) => chrono::DateTime::parse_from_rfc3339(&iso)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            Err(_) => Utc::now(),
        };

        if let Ok(nb) = chrono::DateTime::parse_from_rfc3339(&p.not_before) {
            if sim_date_utc < nb.with_timezone(&Utc) {
                self.sim_status = Some((
                    false,
                    "هنوز فعال نشده".to_string(),
                    format!("تاریخ شروع اعتبار این لایسنس {} می‌باشد.", p.not_before),
                ));
                return;
            }
        }

        if let Some(exp_str) = &p.expires_at {
            if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(exp_str) {
                if sim_date_utc > exp.with_timezone(&Utc) {
                    self.sim_status = Some((
                        false,
                        "منقضی شده".to_string(),
                        format!("اعتبار اشتراک لایسنس در تاریخ {exp_str} به پایان رسیده است."),
                    ));
                    return;
                }
            }
        }

        // 4. App version
        let app_v = self.sim_version.trim();
        if let Some(min_v) = &p.min_app_version {
            if app_v < min_v.as_str() {
                self.sim_status = Some((
                    false,
                    "نسخه نرم‌افزار قدیمی است".to_string(),
                    format!("حداقل نسخه مورد نیاز {min_v} است (نسخه فعلی: {app_v})."),
                ));
                return;
            }
        }
        if let Some(max_v) = &p.max_app_version {
            if app_v > max_v.as_str() {
                self.sim_status = Some((
                    false,
                    "نسخه نرم‌افزار بالاتر از حد مجاز است".to_string(),
                    format!("حداکثر نسخه مجاز {max_v} است (نسخه فعلی: {app_v})."),
                ));
                return;
            }
        }

        self.sim_status = Some((
            true,
            "لایسنس معتبر و فعال در دقیق v4.9.0".to_string(),
            "این لایسنس در تاریخ و سیستم مشخص شده، به طور کامل توسط نرم‌افزار دقیق پذیرفته می‌شود.".to_string(),
        ));
    }

    fn unlock_vault(&mut self) {
        if self.unlock_passphrase.trim().is_empty() {
            self.unlock_status_msg = Some((false, "رمز عبور نمی‌تواند خالی باشد".to_string()));
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Daqiq Vault", &["dqvault", "json"])
            .pick_file()
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<VaultEnvelope>(&content) {
                    Ok(env) => match KeyVault::load_from_envelope(&env, &self.unlock_passphrase) {
                        Ok(vault) => {
                            self.active_key_id = vault.key_id.clone();
                            self.active_public_key = vault.public_key_b64.clone();
                            self.vault = Some(vault);
                            self.unlock_status_msg = Some((
                                true,
                                format!("مخزن کلید با شناسه «{}» با موفقیت بازگشایی شد", self.active_key_id),
                            ));
                            self.toast_message = Some("قفل مخزن کلیدهای امضا باز شد".to_string());
                        }
                        Err(e) => {
                            self.unlock_status_msg = Some((false, format!("رمز عبور اشتباه است: {e}")));
                        }
                    },
                    Err(e) => {
                        self.unlock_status_msg = Some((false, format!("فرمت فایل مخزن نامعتبر است: {e}")));
                    }
                }
            }
        }
    }

    fn create_new_vault(&mut self) {
        if self.new_passphrase.trim().is_empty() {
            self.create_status_msg = Some((false, "رمز عبور مخزن جدید نمی‌تواند خالی باشد".to_string()));
            return;
        }
        if self.new_key_id.trim().is_empty() {
            self.create_status_msg = Some((false, "شناسه کلید جدید نمی‌تواند خالی باشد".to_string()));
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
                    let json_str = serde_json::to_string_pretty(&env).unwrap_or_default();
                    if std::fs::write(&path, json_str).is_ok() {
                        self.active_key_id = vault.key_id.clone();
                        self.active_public_key = vault.public_key_b64.clone();
                        self.vault = Some(vault);
                        self.create_status_msg = Some((
                            true,
                            format!("مخزن کلید جدید در {} ذخیره و فعال شد", path.display()),
                        ));
                        self.toast_message = Some("مخزن کلید جدید ایجاد و فعال شد".to_string());
                    }
                }
            }
            Err(e) => {
                self.create_status_msg = Some((false, format!("خطا در ساخت مخزن: {e}")));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_state_machine_reducer() {
        let mut state = AppState::new();
        let (vault, _) = KeyVault::create_vault("test-key", "TestPass123!").unwrap();
        state.active_key_id = vault.key_id.clone();
        state.active_public_key = vault.public_key_b64.clone();
        state.vault = Some(vault);

        state.update(Message::CustomerNameChanged("فروشگاه تست".to_string()));
        state.update(Message::HwidTextChanged(
            "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string(),
        ));
        state.update(Message::ApplyPreset(Preset::Pro));
        assert_eq!(state.max_devices, 3);

        state.update(Message::GenerateLicenseClicked);
        assert!(state.generated_envelope.is_some());
        assert!(!state.generated_json.is_empty());
        assert!(!state.generated_sha256.is_empty());

        // Test Ledger tab
        state.update(Message::SwitchTab(ActiveTab::Ledger));
        assert_eq!(state.active_tab, ActiveTab::Ledger);

        // Test Inspector tab
        state.update(Message::SwitchTab(ActiveTab::Inspector));
        state.inspect_json = state.generated_json.clone();
        state.update(Message::VerifyInspectClicked);
        assert_eq!(state.inspect_status.as_ref().map(|s| s.0), Some(true));

        // Test Simulator
        state.sim_hwid = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string();
        state.update(Message::RunSimulationClicked);
        assert_eq!(state.sim_status.as_ref().map(|s| s.0), Some(true));

        // Test Theme toggle
        let init_theme = state.theme_mode;
        state.update(Message::ToggleTheme);
        assert_ne!(state.theme_mode, init_theme);

        // Test Vault lock
        state.update(Message::SwitchTab(ActiveTab::Vault));
        state.update(Message::LockVaultClicked);
        assert!(state.vault.is_none());
    }
}
