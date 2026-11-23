# Forensic Investigation & Root Cause Analysis: Daqiq License Generator v1.4.1

## Executive Summary
In release **v1.4.1**, although the startup crash was permanently resolved via static CRT (`+crt-static`), users attempting to activate **Daqiq v4.9.2 (PR #40)** encountered the critical error:
> **"امضای دیجیتال لایسنس نامعتبر است یا محتوای فایل دستکاری شده است."** (The digital signature is invalid or the file content has been tampered with.)

This forensic audit investigates the exact mathematical, cryptographic, state management, and UI/UX causes of this failure, compares `Daqiq` PR #40 (`v4.9.2`) side-by-side with `Daqiq-License-Generator` v1.4.1, and establishes the definitive fixes implemented in **v1.4.2**.

---

## 1. Primary Root Cause: Ephemeral Random Key Generation on Startup & Missing Vault Persistence

### Finding
In `apps/daqiq-lic-gui/src/state.rs`:
```rust
impl AppState {
    pub fn new() -> Self {
        ...
        // Initialize vault with production key ID
        let (vault, _env) = KeyVault::create_vault(DEFAULT_KEY_ID, "daqiq").unwrap();
        let pubkey = vault.public_key_b64.clone();
        let fp = vault.fingerprint();
```
And in `crates/daqiq-lic-core/src/vault.rs`:
```rust
impl KeyVault {
    pub fn create_vault(key_id: &str, passphrase: &str) -> Result<(Self, VaultEnvelope), LicError> {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let vault = Self::from_signing_key_bytes(key_id, &seed);
        ...
```

### Mechanism of Failure
1. On **every single application launch**, `AppState::new()` called `KeyVault::create_vault`, which used the OS CSPRNG (`OsRng`) to generate a **completely random 32-byte Ed25519 private seed**.
2. This random private key was labeled with `DEFAULT_KEY_ID` (`daqiq-prod-v1`).
3. When the user filled out customer details and clicked **"تولید و ذخیره لایسنس"**, the license was signed using this newly minted, completely random private key.
4. The generated `.daqlic` envelope contained `key_id: "daqiq-prod-v1"`.
5. When imported into **`Daqiq v4.9.2`**:
   - `Daqiq`'s `LicenseVerifier` parsed the envelope and looked up `key_id: "daqiq-prod-v1"`.
   - `Daqiq` resolved its embedded production public key from native XOR-masked C shards (`_embedded_keys.py`), which is `tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=`.
   - `Daqiq` ran `matched_pub_key.verify(sig_bytes, data_to_verify)`.
   - Because the license was signed by a completely random ephemeral private key, the cryptographic signature check failed mathematically (`InvalidSignature`).
   - `Daqiq` raised `CryptoVerificationError("امضای دیجیتال لایسنس با هیچ‌یک از کلیدهای عمومی معتبر همخوانی ندارد.")`, mapping to `ValidationStatus.INVALID_SIGNATURE` with the exact Persian error string:
     `"امضای دیجیتال لایسنس نامعتبر است یا محتوای فایل دستکاری شده است."`
6. Furthermore, `config.json` had a field `last_vault_path`, but it was **never read or written** in `apps/daqiq-lic-gui`. Any custom private key imported by the user was kept strictly in volatile RAM and vanished upon closing the app.

---

## 2. Contributing Vector: Format Discrepancy in `Key FP` (Key Fingerprint)

### Finding
In `Daqiq` PR #40 (`v4.9.2`):
```python
def compute_key_fingerprint(raw_bytes: bytes) -> str:
    """Compute short human-readable SHA-256 fingerprint (XXXX-XXXX-XXXX)."""
    digest = hashlib.sha256(raw_bytes).hexdigest().upper()
    return f"{digest[0:4]}-{digest[4:8]}-{digest[8:12]}"
```
`Daqiq v4.9.2` displays the security key fingerprint in its Persian RTL UI as:
`شناسه کلید امنیتی (Key FP): daqiq-prod-v1 · 7925-3864-0164`
- Input: 32 raw bytes of the Ed25519 public key.
- Hash: SHA-256.
- Format: 12 uppercase hex characters grouped into 3 hyphen-separated segments of 4 characters (`XXXX-XXXX-XXXX`).
- Full display: `<key_id> · <fingerprint>` (e.g. `daqiq-prod-v1 · 7925-3864-0164`).

In `Daqiq-License-Generator v1.4.1`:
`crates/daqiq-lic-core/src/vault.rs` formatted the fingerprint as lowercase 12 characters without hyphens:
`792538640164`
Because the format did not match, the operator could not visually compare the fingerprint shown in the Generator with the fingerprint shown in Daqiq's activation window to confirm pairing.

---

## 3. Contributing Vector: Absence of Proactive Key Pairing Warnings

### Finding
In v1.4.1, the Generator provided no prominent warning when an unverified or newly generated ephemeral key was active.
The operator had no visual indicator telling them:
`"کلید امضای فعلی با کلید Daqiq v4.9.2 مطابقت ندارد — لایسنس‌های صادره پذیرفته نخواهند شد!"`

---

## 4. Contributing Vector: Visual & UI/UX Limitations of Raw Win32 in v1.4.1

### Finding
Although v1.4.1 was fast and stable, its raw Win32 presentation suffered from severe commercial aesthetic deficiencies:
1. **Raw Win32 Button Controls (`BS_PUSHBUTTON`):** Standard Windows 95/XP style rectangular buttons with harsh gray borders and dated hover states.
2. **Sunken Text Edits (`WS_EX_CLIENTEDGE` / `WS_BORDER`):** 3D sunken borders incompatible with modern high-end desktop design (Linear, Stripe, Raycast).
3. **Dropdown Controls:** Native combobox dropdowns displayed harsh unstyled white listboxes in Dark mode.
4. **Checkboxes:** Standard raw Win32 square check boxes with uneven text padding and blurry DPI rendering.
5. **Absence of Anti-Aliasing:** GDI lines and borders were aliased and pixelated.

---

## 5. Architectural Remedies in v1.4.2

| Problem Area | v1.4.1 Defect | v1.4.2 Permanent Fix |
| :--- | :--- | :--- |
| **Active Signing Key** | Ephemeral random key generated on every boot (`OsRng`) | Persistent encrypted vault at `%LOCALAPPDATA%\DaqiqLicenseGenerator\vault.dqkey`. Default production key initialized with clear pairing status. |
| **Key FP Calculation** | Lowercase 12-char hex (`792538640164`) | Exact PR #40 algorithm: `XXXX-XXXX-XXXX` uppercase (e.g. `7925-3864-0164`), full display `key_id · XXXX-XXXX-XXXX`. |
| **Pairing Diagnostics** | Hidden inside Tab 3; no active warnings | Live Key FP badge in Sidebar, Generator action bar, and Vault tab with immediate Green (Paired) / Amber (Mismatch) status. |
| **Rendering Engine** | Raw GDI / USER32 (`BS_PUSHBUTTON`, `WS_EX_CLIENTEDGE`) | Hardware-accelerated **Direct2D (`d2d1.dll`) + DirectWrite (`dwrite.dll`)** with automatic WARP fallback; anti-aliased 12px rounded cards, pill toggles, and vector icons. |
| **Input Fields** | Sunken Win32 border boxes | Borderless native Win32 `EDIT` controls seamlessly inset into Direct2D-rendered rounded containers (0ms native BiDi typing + modern aesthetics). |
| **Color Palette** | Plain gray and high contrast | Calm Graphite-Slate Dark (`#171A21` / `#1F242E`) and crisp Alabaster Light (`#F3F5F9` / `#FFFFFF`) conforming to WCAG AA+. |
