use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL_SAFE;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicKeyMatchResult {
    Match,
    Mismatch {
        expected_fp: String,
        actual_fp: String,
    },
    Invalid(String),
}

/// Computes deterministic short key fingerprint matching Daqiq v4.9.2 PR #40:
/// SHA-256 over raw 32-byte Ed25519 public key, formatted as uppercase `XXXX-XXXX-XXXX` (e.g. `7925-3864-0164`).
pub fn compute_key_fingerprint(public_key_b64: &str) -> String {
    let clean = public_key_b64.trim().replace(&['\r', '\n', ' ', '\t'][..], "");
    if let Ok(raw) = BASE64_STANDARD.decode(&clean) {
        if raw.len() == 32 {
            let mut hasher = Sha256::new();
            hasher.update(&raw);
            let hex = format!("{:X}", hasher.finalize());
            if hex.len() >= 12 {
                return format!("{}-{}-{}", &hex[0..4], &hex[4..8], &hex[8..12]);
            }
        }
    }
    "نامعتبر".to_string()
}

/// Formats key fingerprint display string matching Daqiq PR #40: `f"{key_id} · {fingerprint}"`
pub fn compute_key_fingerprint_display(key_id: &str, public_key_b64: &str) -> String {
    format!("{} · {}", key_id, compute_key_fingerprint(public_key_b64))
}

/// Compares the active vault public key against an expected Daqiq public key (Base64 or PEM)
pub fn compare_public_key(active_pub_b64: &str, candidate_b64_or_pem: &str) -> PublicKeyMatchResult {
    let clean_candidate = candidate_b64_or_pem.trim();
    if clean_candidate.is_empty() {
        return PublicKeyMatchResult::Invalid("کلید عمومی خالی است.".to_string());
    }

    let clean_b64 = if clean_candidate.contains("-----BEGIN") {
        clean_candidate
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("")
    } else {
        clean_candidate.replace(&['\r', '\n', ' ', '\t'][..], "")
    };

    let active_clean = active_pub_b64.trim().replace(&['\r', '\n', ' ', '\t'][..], "");
    let active_bytes = match BASE64_STANDARD.decode(&active_clean) {
        Ok(b) if b.len() == 32 => b,
        _ => return PublicKeyMatchResult::Invalid("کلید عمومی فعال مخزن نامعتبر است.".to_string()),
    };

    let cand_bytes = match BASE64_STANDARD.decode(&clean_b64) {
        Ok(b) => b,
        Err(e) => {
            return PublicKeyMatchResult::Invalid(format!("فرمت Base64 کلید وارد شده نامعتبر است: {e}"))
        }
    };

    if cand_bytes.len() != 32 {
        return PublicKeyMatchResult::Invalid(format!(
            "طول کلید عمومی باید دقیقاً ۳۲ بایت باشد؛ دریافت شد: {} بایت.",
            cand_bytes.len()
        ));
    }

    if active_bytes == cand_bytes {
        PublicKeyMatchResult::Match
    } else {
        let expected_fp = compute_key_fingerprint(&clean_b64);
        let actual_fp = compute_key_fingerprint(&active_clean);
        PublicKeyMatchResult::Mismatch {
            expected_fp,
            actual_fp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryPairingResult {
    Paired {
        matched_encoding: String,
        offset: u64,
        public_key_fingerprint: String,
    },
    Mismatch {
        detected_keys: Vec<String>,
        active_fingerprint: String,
        guidance_fa: String,
    },
    FileNotFound,
    IoError(String),
    InvalidActiveKey(String),
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_encode_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Scans a compiled Daqiq executable or Python source file to determine if the active vault's
/// public key is embedded in it (checking raw bytes, standard Base64, URL-safe Base64, and Hex).
pub fn verify_daqiq_binary_pairing(
    file_path: &Path,
    active_public_key_b64: &str,
) -> BinaryPairingResult {
    let clean_pub = active_public_key_b64
        .trim()
        .replace(&['\r', '\n', ' ', '\t'][..], "");
    let raw_bytes = match BASE64_STANDARD.decode(&clean_pub) {
        Ok(b) if b.len() == 32 => b,
        _ => return BinaryPairingResult::InvalidActiveKey("کلید عمومی فعال باید ۳۲ بایت Base64 معتبر باشد.".to_string()),
    };

    let active_fp = compute_key_fingerprint(&clean_pub);

    // Form search targets
    let std_b64 = clean_pub.as_bytes().to_vec();
    let url_b64 = BASE64_URL_SAFE.encode(&raw_bytes).into_bytes();
    let hex_lower = hex_encode(&raw_bytes).into_bytes();
    let hex_upper = hex_encode_upper(&raw_bytes).into_bytes();

    let file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return BinaryPairingResult::FileNotFound,
        Err(e) => return BinaryPairingResult::IoError(format!("خطا در باز کردن فایل: {e}")),
    };

    let mut reader = BufReader::with_capacity(512 * 1024, file);
    let mut chunk = vec![0u8; 512 * 1024];
    let mut total_offset: u64 = 0;
    let mut overlap = Vec::new();
    let overlap_size = 128;

    let default_prod_b64 = b"tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";
    let mut found_default_key = false;

    loop {
        let bytes_read = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => return BinaryPairingResult::IoError(format!("خطا در خواندن فایل: {e}")),
        };

        let mut buffer = Vec::with_capacity(overlap.len() + bytes_read);
        buffer.extend_from_slice(&overlap);
        buffer.extend_from_slice(&chunk[..bytes_read]);

        let base_offset = total_offset.saturating_sub(overlap.len() as u64);

        if let Some(pos) = find_subslice(&buffer, &std_b64) {
            return BinaryPairingResult::Paired {
                matched_encoding: "Base64 (Standard)".to_string(),
                offset: base_offset + pos as u64,
                public_key_fingerprint: active_fp,
            };
        }
        if let Some(pos) = find_subslice(&buffer, &raw_bytes) {
            return BinaryPairingResult::Paired {
                matched_encoding: "Raw 32-byte Ed25519".to_string(),
                offset: base_offset + pos as u64,
                public_key_fingerprint: active_fp,
            };
        }
        if let Some(pos) = find_subslice(&buffer, &url_b64) {
            return BinaryPairingResult::Paired {
                matched_encoding: "Base64 (URL-Safe)".to_string(),
                offset: base_offset + pos as u64,
                public_key_fingerprint: active_fp,
            };
        }
        if let Some(pos) = find_subslice(&buffer, &hex_lower) {
            return BinaryPairingResult::Paired {
                matched_encoding: "Hex (Lowercase)".to_string(),
                offset: base_offset + pos as u64,
                public_key_fingerprint: active_fp,
            };
        }
        if let Some(pos) = find_subslice(&buffer, &hex_upper) {
            return BinaryPairingResult::Paired {
                matched_encoding: "Hex (Uppercase)".to_string(),
                offset: base_offset + pos as u64,
                public_key_fingerprint: active_fp,
            };
        }

        if !found_default_key && find_subslice(&buffer, default_prod_b64).is_some() {
            found_default_key = true;
        }

        total_offset += bytes_read as u64;

        if bytes_read >= overlap_size {
            overlap.clear();
            overlap.extend_from_slice(&chunk[bytes_read - overlap_size..bytes_read]);
        } else {
            overlap.clear();
        }
    }

    let mut detected_keys = Vec::new();
    if found_default_key {
        detected_keys.push(format!("daqiq-prod-v1: {}", std::str::from_utf8(default_prod_b64).unwrap_or("")));
    }

    let guidance_fa = if found_default_key {
        format!(
            "فایل انتخاب‌شده حاوی کلید پیش‌فرض رسمی Daqiq ({}) است، اما کلید فعال گاوصندوق شما ({}) با آن تطابق ندارد! برای رفع خطای امضا، یا کلید خصوصی متناظر را در گاوصندوق وارد کنید یا کلید عمومی فعال گاوصندوق را در فایل licensing/keys.py یا متغیر محیطی DAQIQ_PUBLIC_KEY در پروژه Daqiq قرار داده و برنامه را مجدداً کامپایل نمایید.",
            compute_key_fingerprint(std::str::from_utf8(default_prod_b64).unwrap_or("")),
            active_fp
        )
    } else {
        format!(
            "هیچ نشانه‌ای از کلید فعال مخزن ({}) در فایل انتخابی یافت نشد. لطفاً مطمئن شوید که فایل انتخاب‌شده مربوط به نسخه ۴.۹.۱ نرم‌افزار Daqiq است.",
            active_fp
        )
    };

    BinaryPairingResult::Mismatch {
        detected_keys,
        active_fingerprint: active_fp,
        guidance_fa,
    }
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

    pub fn import_private_key(
        key_id: &str,
        raw_bytes: &[u8; 32],
        passphrase: &str,
    ) -> Result<(Self, VaultEnvelope), LicError> {
        let vault = Self::from_signing_key_bytes(key_id, raw_bytes);
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

    pub fn fingerprint(&self) -> String {
        compute_key_fingerprint(&self.public_key_b64)
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
    fn test_exact_pr40_fingerprint_algorithm() {
        let prod_pub = "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";
        let fp = compute_key_fingerprint(prod_pub);
        assert_eq!(fp, "7925-3864-0164");

        let display = compute_key_fingerprint_display("daqiq-prod-v1", prod_pub);
        assert_eq!(display, "daqiq-prod-v1 · 7925-3864-0164");
    }

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

    #[test]
    fn test_public_key_pairing_checker() {
        let (vault, _env) = KeyVault::create_vault("daqiq-prod-v1", "SecretPass123").unwrap();
        let active_pub = &vault.public_key_b64;

        // Exact match
        let res = compare_public_key(active_pub, active_pub);
        assert_eq!(res, PublicKeyMatchResult::Match);

        // Different public key
        let diff_pub = "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";
        if active_pub != diff_pub {
            let res = compare_public_key(active_pub, diff_pub);
            match res {
                PublicKeyMatchResult::Mismatch { expected_fp, actual_fp } => {
                    assert_eq!(expected_fp, compute_key_fingerprint(diff_pub));
                    assert_eq!(actual_fp, vault.fingerprint());
                }
                _ => panic!("Expected Mismatch"),
            }
        }

        // Invalid key
        let res_invalid = compare_public_key(active_pub, "not-a-valid-base64-key");
        assert!(matches!(res_invalid, PublicKeyMatchResult::Invalid(_)));
    }

    #[test]
    fn test_verify_daqiq_binary_pairing_test() {
        let (vault, _env) = KeyVault::create_vault("daqiq-prod-v1", "SecretPass123").unwrap();
        let temp_dir = tempfile::tempdir().unwrap();

        // 1. Matched Base64 file
        let match_file = temp_dir.path().join("mock_daqiq_matched.exe");
        let content = format!("some binary prefix ... {} ... suffix", vault.public_key_b64);
        fs::write(&match_file, content).unwrap();

        let res = verify_daqiq_binary_pairing(&match_file, &vault.public_key_b64);
        assert!(matches!(res, BinaryPairingResult::Paired { .. }));

        // 2. Mismatch file containing default Daqiq key
        let mismatch_file = temp_dir.path().join("mock_daqiq_default.exe");
        let def_key = "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=";
        fs::write(&mismatch_file, format!("prefix {def_key} suffix")).unwrap();

        if vault.public_key_b64 != def_key {
            let res_mis = verify_daqiq_binary_pairing(&mismatch_file, &vault.public_key_b64);
            assert!(matches!(res_mis, BinaryPairingResult::Mismatch { .. }));
        }

        // 3. File not found
        let not_found_res = verify_daqiq_binary_pairing(
            &temp_dir.path().join("does_not_exist.exe"),
            &vault.public_key_b64,
        );
        assert_eq!(not_found_res, BinaryPairingResult::FileNotFound);
    }
}
