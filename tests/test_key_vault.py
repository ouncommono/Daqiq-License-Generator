"""Unit tests for Encrypted Key Vault (core/key_vault.py)."""

import base64
import json
import os
import pytest
from cryptography.hazmat.primitives import serialization
from core.key_vault import (
    KeyVault,
    UnlockedVault,
    VaultError,
    InvalidPassphraseError,
    VaultCorruptedError,
)


def test_keypair_generation():
    """Verify Ed25519 keypair generation and properties."""
    priv, pub, pub_b64, fp = KeyVault.generate_keypair()
    assert priv is not None
    assert pub is not None
    raw_pub = base64.b64decode(pub_b64)
    assert len(raw_pub) == 32
    assert len(fp) == 64
    assert fp.isalnum()


def test_vault_create_and_unlock_roundtrip(tmp_path):
    """Verify encrypted vault save, atomic persistence, and unlocking."""
    vault_file = str(tmp_path / "test_vendor.dqvault")
    passphrase = "VendorSecretPassphrase!1405"
    key_id = "daqiq-test-vault-v1"

    unlocked, vault_json = KeyVault.create_vault(
        key_id=key_id,
        passphrase=passphrase,
        file_path=vault_file,
    )

    assert unlocked.key_id == key_id
    assert unlocked.is_unlocked is True
    assert os.path.isfile(vault_file)

    # Verify no plaintext secret key appears in the serialized JSON
    with open(vault_file, "r", encoding="utf-8") as f:
        file_text = f.read()

    assert ("BEGIN" + " " + "PRIVATE" + " " + "KEY") not in file_text
    raw_priv = unlocked.private_key.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    raw_priv_b64 = base64.b64encode(raw_priv).decode("ascii")
    assert raw_priv_b64 not in file_text

    # Unlock from file
    loaded_vault = KeyVault.unlock_vault_from_file(vault_file, passphrase)
    assert loaded_vault.key_id == key_id
    assert loaded_vault.public_key_b64 == unlocked.public_key_b64
    assert loaded_vault.fingerprint == unlocked.fingerprint
    assert loaded_vault.is_unlocked is True

    # Lock test
    loaded_vault.lock()
    assert loaded_vault.is_unlocked is False
    with pytest.raises(VaultError, match="قفل"):
        _ = loaded_vault.private_key


def test_wrong_passphrase_fails_cleanly(tmp_path):
    """Verify entering wrong passphrase raises InvalidPassphraseError without crash."""
    vault_file = str(tmp_path / "test_vendor.dqvault")
    KeyVault.create_vault(
        key_id="key-1",
        passphrase="CorrectPassword123",
        file_path=vault_file,
    )

    with pytest.raises(InvalidPassphraseError, match="رمز عبور"):
        KeyVault.unlock_vault_from_file(vault_file, "WrongPassword!!!")


def test_tampered_ciphertext_fails_authentication(tmp_path):
    """Verify AES-GCM authentication tag rejects tampered files."""
    vault_file = str(tmp_path / "test_vendor.dqvault")
    KeyVault.create_vault(
        key_id="key-1",
        passphrase="SecretPassword123",
        file_path=vault_file,
    )

    with open(vault_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    # Corrupt ciphertext byte
    ct_bytes = bytearray(base64.b64decode(data["ciphertext"]))
    ct_bytes[5] ^= 0xFF
    data["ciphertext"] = base64.b64encode(bytes(ct_bytes)).decode("ascii")

    with open(vault_file, "w", encoding="utf-8") as f:
        json.dump(data, f)

    with pytest.raises(InvalidPassphraseError, match="دستکاری"):
        KeyVault.unlock_vault_from_file(vault_file, "SecretPassword123")


def test_import_existing_raw_and_pem_private_key():
    """Verify importing existing Ed25519 secret keys (raw 32-byte and PEM)."""
    priv, pub, pub_b64, fp = KeyVault.generate_keypair()
    raw_priv = priv.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    raw_b64 = base64.b64encode(raw_priv).decode("ascii")

    # 1. Import via raw Base64
    unlocked1, _ = KeyVault.import_raw_private_key(
        raw_key_data=raw_b64,
        key_id="imported-key-1",
        passphrase="Pass",
    )
    assert unlocked1.public_key_b64 == pub_b64
    assert unlocked1.fingerprint == fp

    # 2. Import via PEM
    pem_bytes = priv.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )
    unlocked2, _ = KeyVault.import_raw_private_key(
        raw_key_data=pem_bytes.decode("utf-8"),
        key_id="imported-key-2",
        passphrase="Pass",
    )
    assert unlocked2.public_key_b64 == pub_b64
    assert unlocked2.fingerprint == fp


def test_public_key_export_format_matches_daqiq():
    """Verify exported public key matches Daqiq DEFAULT_KEY_REGISTRY format."""
    unlocked, _ = KeyVault.create_vault(key_id="daqiq-prod-v1", passphrase="Pass")
    snippet = unlocked.export_daqiq_code_snippet()
    assert 'DEFAULT_KEY_REGISTRY["daqiq-prod-v1"] = "' in snippet
    assert unlocked.public_key_b64 in snippet

    raw_pub = base64.b64decode(unlocked.public_key_b64)
    assert len(raw_pub) == 32
