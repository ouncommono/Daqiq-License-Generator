use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ALL_FEATURE_KEYS: [&str; 10] = [
    "batches",
    "dashboard",
    "employees",
    "products",
    "reports",
    "sales",
    "services",
    "settings",
    "suppliers_payments",
    "upgrades",
];

pub const DEFAULT_PRODUCT_ID: &str = "daqiq.desktop";
pub const DEFAULT_KEY_ID: &str = "daqiq-prod-v1";
pub const DEFAULT_PROD_PUBLIC_KEY: &str = "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomerInfo {
    pub name: String,
    pub organization: Option<String>,
    pub contact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicensePayload {
    pub customer: CustomerInfo,
    pub device_fingerprints: Vec<String>,
    pub expires_at: Option<String>,
    pub features: BTreeMap<String, bool>,
    pub format_version: u32,
    pub issued_at: String,
    pub key_id: String,
    pub license_id: String,
    pub license_type: String,
    pub max_app_version: Option<String>,
    pub max_devices: u32,
    pub min_app_version: Option<String>,
    pub not_before: String,
    pub product_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseEnvelope {
    pub algorithm: String,
    pub envelope_version: u32,
    pub key_id: String,
    pub payload: LicensePayload,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceRequest {
    pub request_version: u32,
    pub product_id: String,
    pub generated_at: String,
    pub device_fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_tier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSpec {
    pub license_id: String,
    pub key_id: String,
    pub customer_name: String,
    pub customer_organization: Option<String>,
    pub customer_contact: Option<String>,
    pub license_type: String, // "perpetual" or "subscription"
    pub issued_at: String,
    pub not_before: String,
    pub expires_at: Option<String>,
    pub max_devices: u32,
    pub device_fingerprints: Vec<String>,
    pub features: BTreeMap<String, bool>,
    pub min_app_version: Option<String>,
    pub max_app_version: Option<String>,
}
