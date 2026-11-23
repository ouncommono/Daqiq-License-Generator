use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::error::LicError;

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
    pub license_type: String, // "perpetual", "subscription", or "trial"
    pub max_app_version: Option<String>,
    pub max_devices: u32,
    pub min_app_version: Option<String>,
    pub not_before: String,
    pub product_id: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_policy: Option<String>, // "strict", "standard", "relaxed"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_period_days: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_mode: Option<String>, // "readonly", "full"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry_warning_days: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_clock_skew_hours: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quotas: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseEnvelope {
    pub algorithm: String,
    pub envelope_version: u32,
    pub key_id: String,
    pub payload: LicensePayload,
    pub signature: String,
}

impl LicenseEnvelope {
    pub fn to_compact_token(&self) -> Result<String, LicError> {
        let json_bytes = serde_json::to_vec(self)
            .map_err(|e| LicError::Serialization(e.to_string()))?;
        Ok(BASE64_STANDARD.encode(json_bytes))
    }

    pub fn from_compact_token(token: &str) -> Result<Self, LicError> {
        let clean = token.trim();
        let json_bytes = if clean.starts_with('{') {
            clean.as_bytes().to_vec()
        } else {
            BASE64_STANDARD
                .decode(clean.as_bytes())
                .map_err(|e| LicError::Validation(format!("Invalid base64 token: {e}")))?
        };
        serde_json::from_slice(&json_bytes)
            .map_err(|e| LicError::Validation(format!("Invalid envelope JSON: {e}")))
    }
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
    pub license_type: String, // "perpetual", "subscription", or "trial"
    pub issued_at: String,
    pub not_before: String,
    pub expires_at: Option<String>,
    pub max_devices: u32,
    pub device_fingerprints: Vec<String>,
    pub features: BTreeMap<String, bool>,
    pub min_app_version: Option<String>,
    pub max_app_version: Option<String>,
    pub match_policy: Option<String>,
    pub grace_period_days: Option<u32>,
    pub grace_mode: Option<String>,
    pub expiry_warning_days: Option<u32>,
    pub max_clock_skew_hours: Option<u32>,
    pub quotas: Option<BTreeMap<String, serde_json::Value>>,
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
}
