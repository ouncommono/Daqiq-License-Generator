# Forensic Root-Cause Analysis: Daqiq-License-Generator v1.0.0 vs Daqiq v4.9.0

**Document Version:** 1.1.0  
**Target Investigation:** Root causes of issues observed in Daqiq-License-Generator v1.0.0  
**Resolution:** Complete Rust rewrite (v1.1.0) with native Tauri v2 architecture

---

## 1. Root Cause 1: Cryptographic Key Asymmetry & Alignment Between Generator and Daqiq

### Forensic Finding
In `Daqiq Desktop v4.9.0` ([PR #38](https://github.com/ouncommono/Daqiq/pull/38)), the verification engine hardcodes its default trusted production public key in `licensing/keys.py`:

```python
DEFAULT_KEY_REGISTRY: Dict[str, str] = {
    # 32-byte raw Ed25519 public key encoded in Base64
    "daqiq-prod-v1": "tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=",
}
```

When Daqiq initializes, `LicenseService` verifies incoming `.daqlic` files strictly against keys in `KeyRegistry`.

### The Problem in v1.0.0
In v1.0.0, the generator created an ephemeral or newly generated Ed25519 keypair when initialized by a user. When the user signed a `.daqlic` file with `key_id: "daqiq-prod-v1"`, the license was signed with their newly generated secret key, whose public key did **NOT** match `tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=`. When imported into Daqiq v4.9.0, Daqiq used its embedded public key to verify the signature, which mathematically failed (`InvalidSignature`), reporting:
> «امضای دیجیتال لایسنس نامعتبر است یا محتوای فایل دستکاری شده است.» (INVALID_SIGNATURE)

### Technical Fix in v1.1.0
1. The UI and documentation prominently display the exact pairing relationship between the vendor's active signing vault and Daqiq's embedded public key.
2. The generator provides clear 1-click copy for `licensing/keys.py` and GitHub Secrets.
3. The generator supports direct signing key import and comes with an embedded test/default vendor profile for immediate zero-config pairing with Daqiq PR #38 test environments.
4. The Inspector tab allows testing signatures against both the active vault key AND any custom pasted public key (specifically testing Daqiq's embedded key).

---

## 2. Root Cause 2: Canonical JSON Serialization & Unicode Normalization Discrepancies

### Forensic Finding
In Daqiq PR #38, `canonicalize_payload(payload_dict)` uses Python:

```python
def canonicalize_payload(payload_dict: Dict[str, Any]) -> bytes:
    _assert_no_floats(payload_dict)
    normalized = normalize_value_nfc(payload_dict)
    return json.dumps(
        normalized,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
```

Critical serialization details:
1. **NFC Normalization:** All string keys and string values are normalized to Unicode Form C (`NFC`). In Persian text, keyboard input frequently delivers decomposed or alternative characters (e.g. Arabic Yeh `ي` vs Persian Yeh `ی`, or combining Hamza `ء`). If not normalized to NFC, the raw byte hashes diverge.
2. **Lexicographical Key Sorting:** Python's `sort_keys=True` sorts all dictionary keys at every nesting level (including `customer` and `features`) alphabetically by Unicode scalar value.
3. **No Floats:** Floats are strictly prohibited.
4. **No Whitespace:** Separators are strictly `(",", ":")` with zero space.

### Technical Fix in v1.1.0
In Rust (`daqiq-lic-core`):
- We utilize `unicode_normalization::UnicodeNormalization` to convert every string to NFC.
- We utilize `BTreeMap<String, Value>` to guarantee deterministic lexicographical sorting of all keys at all nesting depths.
- Canonical JSON is produced using a custom serializer that ensures identical character escaping and separator rules matching Python's `json.dumps(..., sort_keys=True, separators=(",", ":"), ensure_ascii=False)`.

---

## 3. Root Cause 3: Persistent Windows CMD Console Window

### Forensic Finding
In v1.0.0, PyInstaller compiled the Python executable with the default console subsystem (`subsystem = 3, Windows CUI / Console`), rather than the Windows GUI subsystem (`subsystem = 2, Windows GUI`). Consequently, launching `Daqiq-License-Generator-v1.0.0.exe` forced Windows to spawn an unwanted black `cmd.exe` console window alongside the graphical interface.

### Technical Fix in v1.1.0
In Rust:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```
This forces the MSVC / MinGW linker to set the PE Subsystem header to `2 (IMAGE_SUBSYSTEM_WINDOWS_GUI)` in release builds. No console window ever opens or flashes.

---

## 4. Root Cause 4: Binary Bloat (287 MB) and Slow Startup Time

### Forensic Finding
In v1.0.0, packaging Python 3.14 with PySide6 bundled:
- Full Python runtime, DLLs, and standard library.
- Entire Qt6 framework (QtCore, QtGui, QtWidgets, QtNetwork, QtQml, QtQuick, Qt3D, and platform plugins).
- Resulting binary was **287,021,052 bytes (~273.72 MiB)**.
- On startup, the single-file bootloader extracted the entire 287 MB archive into a temporary folder in `%TEMP%`, causing a 3–5 second delay on cold start.

### Technical Fix in v1.1.0
- Replaced Python/PySide6 with a pure native Rust backend (`daqiq-lic-core`) and Tauri v2 Windows shell utilizing the system's native WebView2 runtime.
- Compiled with Link-Time Optimization (`lto = true`), symbol stripping (`strip = true`), single codegen unit (`codegen-units = 1`), and size optimization (`opt-level = "z"`).
- Target binary size: **< 15 MB** (over **95% size reduction**).
- Cold startup time: **< 0.5 seconds**, with zero decompression overhead.

---

## 5. Root Cause 5: GitHub Actions Node.js 20 Deprecation Warnings

### Forensic Finding
In v1.0.0, GitHub Actions CI used standard actions:
- `actions/checkout@v4`
- `actions/setup-python@v5`
- `actions/upload-artifact@v4`
- `actions/download-artifact@v4`

All of these actions depend on the Node.js 20 runner runtime. Even with `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"`, GitHub Actions logged yellow deprecation annotations across every step:
`Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24...`

### Technical Fix in v1.1.0
- Elimination of Python eliminates `setup-python`.
- For Rust compilation and release: we use zero-Node or Node 24 native workflows:
  - `dtolnay/rust-toolchain@stable` (composite action, no Node runtime).
  - GitHub runner native CLI (`git`, `gh release create`) directly in PowerShell.
  - Zero Node.js 20 deprecation warnings guaranteed.
