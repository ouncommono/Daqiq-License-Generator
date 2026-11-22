use std::fs;
use std::path::Path;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

use crate::crypto::{canonicalize_payload, verify_license_envelope, DOMAIN_PREFIX};
use crate::error::LicError;
use crate::schema::{LicenseEnvelope, LicensePayload};

const VAULT_VERSION: u32 = 1;
const DEFAULT_ITERATIONS: u32 = 600_000;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEnvelope {
    pub vault_version: u32,
    pub key_id: String,
    pub public_key: String,
    pub kdf: String,
    pub iterations: u32,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub struct KeyVault {
    pub key_id: String,
    pub public_key_b64: String,
    private_key_bytes: Zeroizing<[u8; 32]>,
}

impl KeyVault {
    pub fn create_vault(key_id: &str, passphrase: &str) -> Result<(Self, VaultEnvelope), LicError> {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let vault = Self::from_signing_key_bytes(key_id, &seed);
        seed.zeroize();

        let envelope = vault.export_vault_envelope(passphrase)?;
        Ok((vault, envelope))
    }

    pub fn from_signing_key_bytes(key_id: &str, raw_bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(raw_bytes);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let public_key_b64 = BASE64_STANDARD.encode(verifying_key.as_bytes());

        let mut key_copy = [0u8; 32];
        key_copy.copy_from_slice(raw_bytes);

        Self {
            key_id: key_id.to_string(),
            public_key_b64,
            private_key_bytes: Zeroizing::new(key_copy),
        }
    }

    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.private_key_bytes)
    }

    pub fn export_vault_envelope(&self, passphrase: &str) -> Result<VaultEnvelope, LicError> {
        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        let mut derived_key = Zeroizing::new([0u8; 32]);
        pbkdf2::<Hmac<Sha256>>(
            passphrase.as_bytes(),
            &salt,
            DEFAULT_ITERATIONS,
            &mut *derived_key,
        )
        .map_err(|e| LicError::Vault(format!("Key derivation error: {e:?}")))?;

        let cipher = Aes256Gcm::new_from_slice(&*derived_key)
            .map_err(|e| LicError::Vault(format!("Cipher init error: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, self.private_key_bytes.as_slice())
            .map_err(|e| LicError::Vault(format!("Encryption error: {e}")))?;

        Ok(VaultEnvelope {
            vault_version: VAULT_VERSION,
            key_id: self.key_id.clone(),
            public_key: self.public_key_b64.clone(),
            kdf: "PBKDF2-HMAC-SHA256".to_string(),
            iterations: DEFAULT_ITERATIONS,
            salt: BASE64_STANDARD.encode(salt),
            nonce: BASE64_STANDARD.encode(nonce_bytes),
            ciphertext: BASE64_STANDARD.encode(ciphertext),
        })
    }

    pub fn load_from_envelope(envelope: &VaultEnvelope, passphrase: &str) -> Result<Self, LicError> {
        if envelope.vault_version != VAULT_VERSION {
            return Err(LicError::Vault(format!(
                "نسخه فایل مخزن کلید پشتیبانی نمی‌شود: {}",
                envelope.vault_version
            )));
        }

        let salt = BASE64_STANDARD
            .decode(&envelope.salt)
            .map_err(|e| LicError::Vault(format!("Invalid salt base64: {e}")))?;
        let nonce_bytes = BASE64_STANDARD
            .decode(&envelope.nonce)
            .map_err(|e| LicError::Vault(format!("Invalid nonce base64: {e}")))?;
        let ciphertext = BASE64_STANDARD
            .decode(&envelope.ciphertext)
            .map_err(|e| LicError::Vault(format!("Invalid ciphertext base64: {e}")))?;

        let mut derived_key = Zeroizing::new([0u8; 32]);
        pbkdf2::<Hmac<Sha256>>(
            passphrase.as_bytes(),
            &salt,
            envelope.iterations,
            &mut *derived_key,
        )
        .map_err(|e| LicError::Vault(format!("Key derivation error: {e:?}")))?;

        let cipher = Aes256Gcm::new_from_slice(&*derived_key)
            .map_err(|e| LicError::Vault(format!("Cipher init error: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted = cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|_| LicError::Vault("رمز عبور اشتباه است یا فایل کلید دستکاری شده است".to_string()))?;

        if decrypted.len() != 32 {
            return Err(LicError::Vault("Decrypted key must be exactly 32 bytes".to_string()));
        }

        let mut raw_arr = [0u8; 32];
        raw_arr.copy_from_slice(&decrypted);
        let vault = Self::from_signing_key_bytes(&envelope.key_id, &raw_arr);
        raw_arr.zeroize();

        if vault.public_key_b64 != envelope.public_key {
            return Err(LicError::Vault("کلید عمومی بازسازی‌شده با کلید ذخیره شده مغایرت دارد".to_string()));
        }

        Ok(vault)
    }

    pub fn save_to_file(&self, path: &Path, passphrase: &str) -> Result<(), LicError> {
        let envelope = self.export_vault_envelope(passphrase)?;
        let json_text = serde_json::to_string_pretty(&envelope)
            .map_err(|e| LicError::Serialization(e.to_string()))?;
        fs::write(path, json_text)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path, passphrase: &str) -> Result<Self, LicError> {
        let raw = fs::read_to_string(path)?;
        let envelope: VaultEnvelope = serde_json::from_str(&raw)
            .map_err(|e| LicError::Vault(format!("خطا در خواندن فایل مخزن کلید: {e}")))?;
        Self::load_from_envelope(&envelope, passphrase)
    }

    pub fn sign_license(&self, payload: &LicensePayload) -> Result<LicenseEnvelope, LicError> {
        let canonical_bytes = canonicalize_payload(payload)?;
        let mut message = Vec::with_capacity(DOMAIN_PREFIX.len() + canonical_bytes.len());
        message.extend_from_slice(DOMAIN_PREFIX);
        message.extend_from_slice(&canonical_bytes);

        let signing_key = self.signing_key();
        let signature = signing_key.sign(&message);
        let sig_b64 = BASE64_STANDARD.encode(signature.to_bytes());

        let envelope = LicenseEnvelope {
            algorithm: "Ed25519".to_string(),
            envelope_version: 1,
            key_id: payload.key_id.clone(),
            payload: payload.clone(),
            signature: sig_b64,
        };

        // Self-verify the newly signed license before emitting
        verify_license_envelope(&envelope, &self.public_key_b64)?;

        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_roundtrip() {
        let (vault, envelope) = KeyVault::create_vault("test-key-v1", "SecretPass123").unwrap();
        assert_eq!(vault.key_id, "test-key-v1");
        assert_eq!(envelope.key_id, "test-key-v1");

        // Unlock with correct passphrase
        let restored = KeyVault::load_from_envelope(&envelope, "SecretPass123").unwrap();
        assert_eq!(restored.key_id, vault.key_id);
        assert_eq!(restored.public_key_b64, vault.public_key_b64);

        // Unlock with wrong passphrase
        let failed = KeyVault::load_from_envelope(&envelope, "WrongPassword");
        assert!(failed.is_err());
    }
}
