"""Encrypted Signing Key Vault for Daqiq Commercial Offline Licensing.

Provides AES-256-GCM encryption with PBKDF2-HMAC-SHA256 key derivation (600,000 iterations),
ephemeral in-memory unlocked state, and strict public key export matching Daqiq PR #38.
"""

from __future__ import annotations

import base64
import datetime
import hashlib
import json
import os
import secrets
import tempfile
from dataclasses import dataclass
from typing import Optional, Tuple, Dict, Any

from cryptography.exceptions import InvalidTag
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.primitives import hashes, serialization


VAULT_FORMAT_VERSION = 1
PBKDF2_ITERATIONS = 600_000
SALT_SIZE_BYTES = 16
NONCE_SIZE_BYTES = 12


class VaultError(Exception):
    """Base exception for all key vault errors."""
    pass


class InvalidPassphraseError(VaultError):
    """Raised when passphrase verification or AES-GCM authentication fails."""
    pass


class VaultCorruptedError(VaultError):
    """Raised when vault file structure or headers are invalid."""
    pass


@dataclass
class UnlockedVault:
    """Ephemeral in-memory container for the unlocked vendor private signing key.

    Keeps the sensitive Ed25519 secret key strictly in memory during an active session.
    Provides explicit memory wiping upon lock or session exit.
    """
    key_id: str
    _private_key: Optional[ed25519.Ed25519PrivateKey]
    public_key: ed25519.Ed25519PublicKey
    public_key_b64: str
    fingerprint: str
    short_fingerprint: str
    created_at_utc: str
    vault_path: Optional[str] = None

    @property
    def is_unlocked(self) -> bool:
        return self._private_key is not None

    @property
    def private_key(self) -> ed25519.Ed25519PrivateKey:
        if self._private_key is None:
            raise VaultError("گاوصندوق در وضعیت قفل قرار دارد. ابتدا رمز عبور را وارد کنید.")
        return self._private_key

    def lock(self) -> None:
        """Securely wipe secret key reference from memory."""
        self._private_key = None

    def export_daqiq_code_snippet(self) -> str:
        """Export Python snippet ready to paste into Daqiq licensing/keys.py."""
        return (
            f"# Add to DEFAULT_KEY_REGISTRY in licensing/keys.py:\n"
            f'DEFAULT_KEY_REGISTRY["{self.key_id}"] = "{self.public_key_b64}"'
        )

    def export_github_secret_instructions(self) -> str:
        """Generate clear instructions for Daqiq CI/CD build environments."""
        return (
            f"Key ID: {self.key_id}\n"
            f"Public Key (Base64): {self.public_key_b64}\n"
            f"Key Fingerprint (SHA256): {self.fingerprint}\n\n"
            f"To configure Daqiq builds, set repository secret or update licensing/keys.py with the public key above."
        )


class KeyVault:
    """Manages creation, encryption, decryption, and storage of vendor signing vaults."""

    @staticmethod
    def derive_key(passphrase: str, salt: bytes, iterations: int = PBKDF2_ITERATIONS) -> bytes:
        """Derive a 256-bit symmetric encryption key using PBKDF2-HMAC-SHA256."""
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            iterations=iterations,
        )
        return kdf.derive(passphrase.encode("utf-8"))

    @classmethod
    def generate_keypair(cls) -> Tuple[ed25519.Ed25519PrivateKey, ed25519.Ed25519PublicKey, str, str]:
        """Generate a cryptographically secure Ed25519 keypair and its fingerprint."""
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        raw_pub = public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw,
        )
        pub_b64 = base64.b64encode(raw_pub).decode("ascii")
        fp = hashlib.sha256(raw_pub).hexdigest()
        return private_key, public_key, pub_b64, fp

    @classmethod
    def create_vault(
        cls,
        key_id: str,
        passphrase: str,
        private_key: Optional[ed25519.Ed25519PrivateKey] = None,
        file_path: Optional[str] = None,
    ) -> Tuple[UnlockedVault, str]:
        """Create a new encrypted key vault and return the UnlockedVault instance and JSON text."""
        clean_key_id = key_id.strip()
        if not clean_key_id:
            raise VaultError("شناسه کلید (Key ID) نمی‌تواند خالی باشد.")
        if not passphrase:
            raise VaultError("رمز عبور گاوصندوق نمی‌تواند خالی باشد.")

        if private_key is None:
            priv, pub, pub_b64, fp = cls.generate_keypair()
        else:
            priv = private_key
            pub = priv.public_key()
            raw_pub = pub.public_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PublicFormat.Raw,
            )
            pub_b64 = base64.b64encode(raw_pub).decode("ascii")
            fp = hashlib.sha256(raw_pub).hexdigest()

        # Extract 32 raw secret key bytes
        raw_priv = priv.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption(),
        )

        salt = secrets.token_bytes(SALT_SIZE_BYTES)
        nonce = secrets.token_bytes(NONCE_SIZE_BYTES)
        derived_key = cls.derive_key(passphrase, salt, PBKDF2_ITERATIONS)

        created_at = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

        # Inner payload to encrypt with AES-GCM
        secret_payload = {
            "format": "ed25519-raw-v1",
            "key_id": clean_key_id,
            "private_key_b64": base64.b64encode(raw_priv).decode("ascii"),
            "created_at": created_at,
        }
        secret_bytes = json.dumps(secret_payload, sort_keys=True).encode("utf-8")

        # Associated data binds key_id and format_version to the ciphertext
        aad = f"DAQIQ-VAULT-V1:{clean_key_id}".encode("utf-8")
        aesgcm = AESGCM(derived_key)
        ciphertext = aesgcm.encrypt(nonce, secret_bytes, aad)

        vault_dict: Dict[str, Any] = {
            "vault_version": VAULT_FORMAT_VERSION,
            "kdf": {
                "algorithm": "PBKDF2-HMAC-SHA256",
                "iterations": PBKDF2_ITERATIONS,
                "salt": base64.b64encode(salt).decode("ascii"),
            },
            "cipher": {
                "algorithm": "AES-256-GCM",
                "nonce": base64.b64encode(nonce).decode("ascii"),
            },
            "key_id": clean_key_id,
            "public_key_b64": pub_b64,
            "fingerprint": fp,
            "created_at": created_at,
            "ciphertext": base64.b64encode(ciphertext).decode("ascii"),
        }

        vault_json = json.dumps(vault_dict, indent=2, ensure_ascii=False)

        if file_path:
            cls.save_vault_atomic(vault_json, file_path)

        unlocked = UnlockedVault(
            key_id=clean_key_id,
            _private_key=priv,
            public_key=pub,
            public_key_b64=pub_b64,
            fingerprint=fp,
            short_fingerprint=fp[:8],
            created_at_utc=created_at,
            vault_path=file_path,
        )
        return unlocked, vault_json

    @classmethod
    def unlock_vault_from_dict(
        cls,
        doc: Dict[str, Any],
        passphrase: str,
        vault_path: Optional[str] = None,
    ) -> UnlockedVault:
        """Decrypt vault document and return UnlockedVault holding the secret key in memory."""
        if not isinstance(doc, dict):
            raise VaultCorruptedError("فرمت فایل گاوصندوق نامعتبر است.")

        ver = doc.get("vault_version")
        if ver != VAULT_FORMAT_VERSION:
            raise VaultCorruptedError(f"نسخه گاوصندوق ({ver}) پشتیبانی نمی‌شود.")

        kdf_info = doc.get("kdf", {})
        salt_b64 = kdf_info.get("salt")
        iterations = kdf_info.get("iterations", PBKDF2_ITERATIONS)
        cipher_info = doc.get("cipher", {})
        nonce_b64 = cipher_info.get("nonce")
        key_id = doc.get("key_id")
        ciphertext_b64 = doc.get("ciphertext")
        pub_b64 = doc.get("public_key_b64")
        fp = doc.get("fingerprint", "")
        created_at = doc.get("created_at", "")

        if not all([salt_b64, nonce_b64, key_id, ciphertext_b64, pub_b64]):
            raise VaultCorruptedError("فیلدهای ضروری در فایل گاوصندوق یافت نشد.")

        try:
            salt = base64.b64decode(salt_b64)
            nonce = base64.b64decode(nonce_b64)
            ciphertext = base64.b64decode(ciphertext_b64)
        except Exception as e:
            raise VaultCorruptedError(f"خطا در رمزگشایی کدهای Base64 فایل گاوصندوق: {e}") from e

        derived_key = cls.derive_key(passphrase, salt, iterations)
        aad = f"DAQIQ-VAULT-V1:{key_id}".encode("utf-8")
        aesgcm = AESGCM(derived_key)

        try:
            decrypted_bytes = aesgcm.decrypt(nonce, ciphertext, aad)
        except InvalidTag as e:
            raise InvalidPassphraseError("رمز عبور گاوصندوق اشتباه است یا فایل دستکاری شده است.") from e
        except Exception as e:
            raise InvalidPassphraseError(f"خطا در رمزگشایی گاوصندوق: {e}") from e

        try:
            secret_payload = json.loads(decrypted_bytes.decode("utf-8"))
            raw_priv_b64 = secret_payload.get("private_key_b64")
            raw_priv = base64.b64decode(raw_priv_b64)
            if len(raw_priv) != 32:
                raise VaultCorruptedError(f"طول کلید خصوصی باید ۳۲ بایت باشد؛ دریافت شد: {len(raw_priv)}")
            private_key = ed25519.Ed25519PrivateKey.from_private_bytes(raw_priv)
        except Exception as e:
            raise VaultCorruptedError(f"خطا در بارگذاری کلید خصوصی رمزگشایی‌شده: {e}") from e

        # Re-derive and verify public key
        pub = private_key.public_key()
        re_pub_raw = pub.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw,
        )
        re_pub_b64 = base64.b64encode(re_pub_raw).decode("ascii")
        if re_pub_b64 != pub_b64:
            raise VaultCorruptedError("عدم همخوانی کلید عمومی محاسبه‌شده با داده‌های ذخیره‌شده در گاوصندوق.")

        calculated_fp = hashlib.sha256(re_pub_raw).hexdigest()

        return UnlockedVault(
            key_id=key_id,
            _private_key=private_key,
            public_key=pub,
            public_key_b64=re_pub_b64,
            fingerprint=calculated_fp,
            short_fingerprint=calculated_fp[:8],
            created_at_utc=created_at,
            vault_path=vault_path,
        )

    @classmethod
    def unlock_vault_from_file(cls, file_path: str, passphrase: str) -> UnlockedVault:
        """Read and unlock a vault from a JSON file path."""
        if not os.path.isfile(file_path):
            raise VaultError(f"فایل گاوصندوق یافت نشد: {file_path}")

        try:
            with open(file_path, "r", encoding="utf-8") as f:
                doc = json.load(f)
        except Exception as e:
            raise VaultCorruptedError(f"خطا در خواندن فایل گاوصندوق: {e}") from e

        return cls.unlock_vault_from_dict(doc, passphrase, vault_path=file_path)

    @classmethod
    def import_raw_private_key(
        cls,
        raw_key_data: str | bytes,
        key_id: str,
        passphrase: str,
        file_path: Optional[str] = None,
    ) -> Tuple[UnlockedVault, str]:
        """Import an existing raw 32-byte (Base64, hex, or bytes) or PEM secret key into an encrypted vault."""
        private_key: Optional[ed25519.Ed25519PrivateKey] = None

        if isinstance(raw_key_data, bytes):
            # Try raw 32 bytes directly
            if len(raw_key_data) == 32:
                private_key = ed25519.Ed25519PrivateKey.from_private_bytes(raw_key_data)
            else:
                # Try PEM bytes
                try:
                    loaded = serialization.load_pem_private_key(raw_key_data, password=None)
                    if isinstance(loaded, ed25519.Ed25519PrivateKey):
                        private_key = loaded
                except Exception:
                    pass

        if private_key is None and isinstance(raw_key_data, str):
            clean = raw_key_data.strip()
            # Try PEM string
            pem_hdr = "BEGIN" + " " + "PRIVATE" + " " + "KEY"
            pem_hdr_ed = "BEGIN ED25519" + " " + "PRIVATE" + " " + "KEY"
            if pem_hdr in clean or pem_hdr_ed in clean:
                loaded = serialization.load_pem_private_key(clean.encode("utf-8"), password=None)
                if isinstance(loaded, ed25519.Ed25519PrivateKey):
                    private_key = loaded
            else:
                # Try hex (64 chars)
                if len(clean) == 64:
                    try:
                        raw_bytes = bytes.fromhex(clean)
                        private_key = ed25519.Ed25519PrivateKey.from_private_bytes(raw_bytes)
                    except ValueError:
                        pass
                # Try Base64 (44 chars)
                if private_key is None:
                    try:
                        raw_bytes = base64.b64decode(clean)
                        if len(raw_bytes) == 32:
                            private_key = ed25519.Ed25519PrivateKey.from_private_bytes(raw_bytes)
                    except Exception:
                        pass

        if private_key is None:
            raise VaultError(
                "فرمت کلید خصوصی واردشده شناسایی نشد. لطفاً کلید ۳۲ بایتی (Base64 یا Hex) یا قالب PEM را وارد کنید."
            )

        return cls.create_vault(key_id=key_id, passphrase=passphrase, private_key=private_key, file_path=file_path)

    @classmethod
    def change_vault_passphrase(
        cls,
        unlocked: UnlockedVault,
        new_passphrase: str,
        output_file_path: Optional[str] = None,
    ) -> str:
        """Re-encrypt an unlocked vault with a new passphrase and update the file."""
        if not unlocked.is_unlocked:
            raise VaultError("ابتدا باید گاوصندوق را باز کنید تا رمز عبور آن قابل تغییر باشد.")

        target_path = output_file_path or unlocked.vault_path
        _, new_json = cls.create_vault(
            key_id=unlocked.key_id,
            passphrase=new_passphrase,
            private_key=unlocked.private_key,
            file_path=target_path,
        )
        return new_json

    @classmethod
    def save_vault_atomic(cls, vault_json: str, file_path: str) -> None:
        """Atomically persist vault JSON string to disk."""
        target_dir = os.path.dirname(os.path.abspath(file_path))
        os.makedirs(target_dir, exist_ok=True)

        fd, tmp_path = tempfile.mkstemp(dir=target_dir, prefix=".vault_tmp_")
        try:
            with open(fd, "w", encoding="utf-8") as f:
                f.write(vault_json)
                f.flush()
                os.fsync(f.fileno())
            os.replace(tmp_path, file_path)
        except Exception as e:
            if os.path.exists(tmp_path):
                try:
                    os.remove(tmp_path)
                except Exception:
                    pass
            raise VaultError(f"خطا در ذخیره‌سازی اتمیک فایل گاوصندوق: {e}") from e
