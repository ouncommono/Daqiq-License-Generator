pub mod crypto;
pub mod error;
pub mod hwid;
pub mod jalali;
pub mod registry;
pub mod schema;
pub mod vault;

pub use crypto::{
    canonicalize_json_value, canonicalize_payload, verify_license_envelope, CanonicalValue,
    Keypair, DOMAIN_PREFIX,
};
pub use error::LicError;
pub use hwid::{clean_hwid_input, parse_bulk_hwids, parse_daqreq, validate_hwid};
pub use jalali::{
    gregorian_to_jalali, is_jalali_leap, jalali_str_to_utc_iso, jalali_to_gregorian,
    utc_iso_to_jalali_str,
};
pub use registry::{RegistryDb, RegistryRecord};
pub use schema::{
    CustomerInfo, DeviceRequest, LicenseEnvelope, LicensePayload, LicenseSpec,
    ALL_FEATURE_KEYS, DEFAULT_KEY_ID, DEFAULT_PRODUCT_ID, DEFAULT_PROD_PUBLIC_KEY,
};
pub use vault::{KeyVault, VaultEnvelope};
