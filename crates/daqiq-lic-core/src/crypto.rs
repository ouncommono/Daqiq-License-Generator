use std::collections::BTreeMap;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::error::LicError;
use crate::schema::{LicenseEnvelope, LicensePayload};

pub const DOMAIN_PREFIX: &[u8] = b"DAQIQ-LICENSE-V1\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CanonicalValue {
    Null,
    Bool(bool),
    Int(i64),
    Uint(u64),
    String(String),
    Array(Vec<CanonicalValue>),
    Object(BTreeMap<String, CanonicalValue>),
}

pub fn json_to_canonical_value(val: &serde_json::Value) -> Result<CanonicalValue, LicError> {
    match val {
        serde_json::Value::Null => Ok(CanonicalValue::Null),
        serde_json::Value::Bool(b) => Ok(CanonicalValue::Bool(*b)),
        serde_json::Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                Ok(CanonicalValue::Int(i))
            } else if let Some(u) = num.as_u64() {
                Ok(CanonicalValue::Uint(u))
            } else {
                Err(LicError::Validation(
                    "مقدار اعشاری (float) در لایسنس مجاز نیست".to_string(),
                ))
            }
        }
        serde_json::Value::String(s) => {
            let normalized: String = s.nfc().collect();
            Ok(CanonicalValue::String(normalized))
        }
        serde_json::Value::Array(arr) => {
            let mut list = Vec::with_capacity(arr.len());
            for item in arr {
                list.push(json_to_canonical_value(item)?);
            }
            Ok(CanonicalValue::Array(list))
        }
        serde_json::Value::Object(obj) => {
            let mut map = BTreeMap::new();
            for (k, v) in obj {
                let norm_k: String = k.nfc().collect();
                map.insert(norm_k, json_to_canonical_value(v)?);
            }
            Ok(CanonicalValue::Object(map))
        }
    }
}

pub fn canonicalize_payload(payload: &LicensePayload) -> Result<Vec<u8>, LicError> {
    let json_val = serde_json::to_value(payload)
        .map_err(|e| LicError::Serialization(e.to_string()))?;
    let canonical = json_to_canonical_value(&json_val)?;
    let canonical_str = serde_json::to_string(&canonical)
        .map_err(|e| LicError::Serialization(e.to_string()))?;
    Ok(canonical_str.into_bytes())
}

pub fn canonicalize_json_value(val: &serde_json::Value) -> Result<Vec<u8>, LicError> {
    let canonical = json_to_canonical_value(val)?;
    let canonical_str = serde_json::to_string(&canonical)
        .map_err(|e| LicError::Serialization(e.to_string()))?;
    Ok(canonical_str.into_bytes())
}

pub struct Keypair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Keypair {
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn public_key_b64(&self) -> String {
        BASE64_STANDARD.encode(self.verifying_key.as_bytes())
    }

    pub fn sign_license(&self, payload: &LicensePayload) -> Result<LicenseEnvelope, LicError> {
        let canonical_bytes = canonicalize_payload(payload)?;
        let mut message = Vec::with_capacity(DOMAIN_PREFIX.len() + canonical_bytes.len());
        message.extend_from_slice(DOMAIN_PREFIX);
        message.extend_from_slice(&canonical_bytes);

        let signature = self.signing_key.sign(&message);
        let sig_b64 = BASE64_STANDARD.encode(signature.to_bytes());

        Ok(LicenseEnvelope {
            envelope_version: 1,
            algorithm: "Ed25519".to_string(),
            key_id: payload.key_id.clone(),
            payload: payload.clone(),
            signature: sig_b64,
        })
    }
}

pub fn verify_license_envelope(
    envelope: &LicenseEnvelope,
    public_key_b64: &str,
) -> Result<bool, LicError> {
    if envelope.envelope_version != 1 {
        return Err(LicError::Validation(format!(
            "نسخه نامعتبر envelope: {}",
            envelope.envelope_version
        )));
    }
    if envelope.algorithm != "Ed25519" {
        return Err(LicError::Validation(format!(
            "الگوریتم پشتیبانی نشده: {}",
            envelope.algorithm
        )));
    }
    if envelope.key_id != envelope.payload.key_id {
        return Err(LicError::Validation(
            "شناسه کلید در پوشه و محتوا همخوانی ندارد".to_string(),
        ));
    }

    let pubkey_bytes = BASE64_STANDARD
        .decode(public_key_b64.trim())
        .map_err(|e| LicError::Crypto(format!("Invalid public key base64: {e}")))?;

    if pubkey_bytes.len() != 32 {
        return Err(LicError::Crypto("Public key must be 32 bytes".to_string()));
    }

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&pubkey_bytes);
    let verifying_key = VerifyingKey::from_bytes(&key_arr)
        .map_err(|e| LicError::Crypto(format!("Invalid Ed25519 public key: {e}")))?;

    let sig_bytes = BASE64_STANDARD
        .decode(envelope.signature.trim())
        .map_err(|e| LicError::Crypto(format!("Invalid signature base64: {e}")))?;

    if sig_bytes.len() != 64 {
        return Err(LicError::Crypto("Signature must be 64 bytes".to_string()));
    }

    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|e| LicError::Crypto(format!("Invalid signature structure: {e}")))?;

    let canonical_bytes = canonicalize_payload(&envelope.payload)?;
    let mut message = Vec::with_capacity(DOMAIN_PREFIX.len() + canonical_bytes.len());
    message.extend_from_slice(DOMAIN_PREFIX);
    message.extend_from_slice(&canonical_bytes);

    verifying_key
        .verify_strict(&message, &signature)
        .map_err(|_| LicError::InvalidSignature)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::CustomerInfo;
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    #[test]
    fn test_official_golden_test_vector_parity() {
        let raw_pub_b64 = "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";
        let expected_sha = "52bdf69ad45d565685bae87f75c03d4a5baa43a3acafe785499b16ee5618366a";
        let sig_b64 = "kgrF5f89h9JBIHiQ55H5J+jZul3ZwC6NdeEDXtrlxMDWWlBr892vliqLsTxPD0q+RlEJaXsIQZUSjaevKSGjCA==";

        let mut features = BTreeMap::new();
        features.insert("batches".to_string(), true);
        features.insert("dashboard".to_string(), true);
        features.insert("employees".to_string(), true);
        features.insert("products".to_string(), true);
        features.insert("reports".to_string(), true);
        features.insert("sales".to_string(), true);
        features.insert("services".to_string(), true);
        features.insert("settings".to_string(), true);
        features.insert("suppliers_payments".to_string(), true);
        features.insert("upgrades".to_string(), true);

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

        let canon_bytes = canonicalize_payload(&payload).expect("Canonicalization should succeed");
        let mut msg = Vec::new();
        msg.extend_from_slice(DOMAIN_PREFIX);
        msg.extend_from_slice(&canon_bytes);

        let mut hasher = Sha256::new();
        hasher.update(&msg);
        let actual_sha = format!("{:x}", hasher.finalize());

        assert_eq!(
            actual_sha, expected_sha,
            "Domain-separated message SHA-256 must match golden vector exactly!"
        );

        let envelope = LicenseEnvelope {
            algorithm: "Ed25519".to_string(),
            envelope_version: 1,
            key_id: "daqiq-test-v1".to_string(),
            payload,
            signature: sig_b64.to_string(),
        };

        let valid = verify_license_envelope(&envelope, raw_pub_b64)
            .expect("Verification must succeed on golden test vector");
        assert!(valid);
    }
}
