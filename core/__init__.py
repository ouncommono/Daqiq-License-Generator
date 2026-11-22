"""Core cryptographic and licensing engine for Daqiq License Generator."""

from .key_vault import KeyVault, UnlockedVault, VaultError
from .hwid_parser import (
    HWIDParser,
    DeviceEntry,
    ParsedHWIDBatch,
    validate_hwid_string,
    clean_hwid_string,
)
from .license_builder import (
    LicenseBuilder,
    LicenseSpec,
    GeneratedLicense,
    LicenseValidationError,
    canonicalize_payload,
)
from .registry_db import RegistryDB, LicenseRecord

__all__ = [
    "KeyVault",
    "UnlockedVault",
    "VaultError",
    "HWIDParser",
    "DeviceEntry",
    "ParsedHWIDBatch",
    "validate_hwid_string",
    "clean_hwid_string",
    "LicenseBuilder",
    "LicenseSpec",
    "GeneratedLicense",
    "LicenseValidationError",
    "canonicalize_payload",
    "RegistryDB",
    "LicenseRecord",
]
