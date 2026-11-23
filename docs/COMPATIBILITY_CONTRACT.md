# Daqiq Licensing Compatibility Contract (v1.4.0)

**Single Source of Truth:** `https://github.com/ouncommono/Daqiq/pull/39` (Daqiq v4.9.1)  
**Contract Version:** 1.4.0  
**Target Applications:**  
- Issuer: **Daqiq License Generator v1.4.0** (`daqiq.license_generator`)  
- Consumer: **Daqiq Desktop v4.9.1** (`daqiq.desktop`)  
- Device Request Utility: **Companion App A / In-App Exporter** (`daqiq.request_utility`)

---

## 1. Cryptographic Protocol & Envelope

### 1.1 Envelope Structure
The `.daqlic` file is an offline JSON document encoded in **UTF-8 without BOM**, limited to a maximum file size of **65,536 bytes (64 KiB)**.
Alternatively, the license can be pasted directly into Daqiq Desktop as a **Compact Text Token** (Base64-encoded UTF-8 string of the entire envelope JSON starting with `eyJ...`).

```json
{
  "envelope_version": 1,
  "algorithm": "Ed25519",
  "key_id": "daqiq-prod-v1",
  "payload": {
    "format_version": 1,
    "product_id": "daqiq.desktop",
    "license_id": "LIC-2026-000001",
    "key_id": "daqiq-prod-v1",
    "customer": {
      "contact": "+989120000000",
      "name": "نام مشتری یا فروشگاه",
      "organization": "مجموعه نمونه (اختیاری)"
    },
    "license_type": "subscription",
    "issued_at": "2026-11-22T00:00:00Z",
    "not_before": "2026-11-22T00:00:00Z",
    "expires_at": "2027-11-22T00:00:00Z",
    "max_devices": 1,
    "device_fingerprints": [
      "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"
    ],
    "match_policy": "standard",
    "grace_period_days": 7,
    "grace_mode": "readonly",
    "expiry_warning_days": 14,
    "max_clock_skew_hours": 24,
    "features": {
      "batches": true,
      "dashboard": true,
      "employees": true,
      "products": true,
      "reports": true,
      "sales": true,
      "services": true,
      "settings": true,
      "suppliers_payments": true,
      "upgrades": true
    },
    "min_app_version": "4.9.0",
    "max_app_version": null,
    "quotas": {},
    "metadata": {}
  },
  "signature": "<base64-standard-rfc4648-64-byte-ed25519-signature>"
}
```

### 1.2 Cryptographic Parameters
- **Algorithm:** Ed25519 (RFC 8032).
- **Public Key Representation:** Raw 32 bytes encoded in standard Base64 (e.g. `tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=`). In Daqiq Desktop, registered in `licensing/keys.py` within `DEFAULT_KEY_REGISTRY`.
- **Public Key Fingerprint:** Deterministic 12-hex character prefix of SHA-256 over raw public key bytes:
  $$\text{fingerprint} = \text{hex}(\text{SHA256}(\text{raw\_pub\_bytes}))[0..12]$$
- **Secret Signing Key Storage:** Never stored in plaintext. Encrypted in vendor vault files using AES-256-GCM with PBKDF2-HMAC-SHA256 (600,000 iterations). Memory zeroized on drop.
- **Domain Separation Prefix:**
  ```python
  DOMAIN_PREFIX = b"DAQIQ-LICENSE-V1\n"
  ```
- **Signature Encoding:** Standard Base64 (RFC 4648), exactly 64 raw bytes decoding from 88 Base64 characters (with `=` padding).

### 1.3 Canonical Serialization Algorithm
To guarantee byte-for-byte reproducibility across Python and Rust:
1. **Type Strictness:** Payload must contain only `dict`, `list`, `str`, `int`, `bool`, and `None`. Floats (`float`), `NaN`, and `Infinity` are strictly forbidden and rejected.
2. **Unicode Normalization:** Recursively normalize all dictionary keys and string values to **Unicode Normalization Form C (NFC)**.
3. **JSON Serialization:**
   ```python
   canonical_bytes = json.dumps(
       normalized_payload_dict,
       sort_keys=True,
       separators=(",", ":"),
       ensure_ascii=False,
   ).encode("utf-8")
   ```
4. **Field Omission vs Null Invariant:**
   - In `CustomerInfo`: `contact` and `organization` MUST serialize as `null` when unset.
   - In `LicensePayload`: `expires_at`, `min_app_version`, and `max_app_version` MUST serialize as `null` when unset.
   - Advanced PR #39 fields (`match_policy`, `grace_period_days`, `grace_mode`, `expiry_warning_days`, `max_clock_skew_hours`, `quotas`, `metadata`) are **omitted** from the dictionary when unset (matching `if self.field is not None:` in `LicensePayload.to_dict()`).
5. **Signing Payload Construction:**
   ```python
   message_to_sign = DOMAIN_PREFIX + canonical_bytes
   signature = private_key.sign(message_to_sign)
   signature_b64 = base64.b64encode(signature).decode("ascii")
   ```

---

## 2. Payload Field Specification (PR #39 Parity)

| Field Name | Type | Allowed Values | Null Behavior | Description |
|---|---|---|---|---|
| `format_version` | Integer | `1` | Required | Schema version identifier. |
| `product_id` | String | `"daqiq.desktop"` | Required | Target application identifier. |
| `license_id` | String | Non-empty ASCII/Unicode | Required | Unique vendor license number (e.g. `LIC-2026-000001`). |
| `key_id` | String | Non-empty ASCII | Required | Key ID matching `DEFAULT_KEY_REGISTRY` (`daqiq-prod-v1`). |
| `customer.name` | String | Non-empty Unicode NFC | Required | Customer name in Persian or Latin. |
| `customer.organization` | String / Null | Unicode NFC or `null` | Serializes as `null` | Organization name. |
| `customer.contact` | String / Null | ASCII/Unicode or `null` | Serializes as `null` | Support phone, email, or contact ID. |
| `license_type` | String | `"perpetual"`, `"subscription"`, `"trial"` | Required | License commercial model. |
| `issued_at` | String | ISO-8601 UTC (`...Z`) | Required | Creation timestamp. |
| `not_before` | String | ISO-8601 UTC (`...Z`) | Required | Start of validity. |
| `expires_at` | String / Null | ISO-8601 UTC or `null` | Serializes as `null` | Expiration timestamp (`null` for perpetual). |
| `max_devices` | Integer | $\ge 1$ | Required | Maximum allowed hardware nodes. |
| `device_fingerprints` | List[String] | Array of `fp_v1:...` | Required | Distinct hardware fingerprints ($1 \le N \le \text{max\_devices}$). |
| `features` | Dict[str, bool] | 10 boolean flags | Required | Feature flags for tabs 0 through 9. |
| `min_app_version` | String / Null | Semver or `null` | Serializes as `null` | Minimum compatible application version. |
| `max_app_version` | String / Null | Semver or `null` | Serializes as `null` | Maximum compatible application version. |
| `match_policy` | String / Null | `"strict"`, `"standard"`, `"relaxed"` | Omitted if unset | Node-lock matching tier policy. |
| `grace_period_days` | Integer / Null | $\ge 0$ | Omitted if unset | Grace period days after expiration. |
| `grace_mode` | String / Null | `"readonly"`, `"full"` | Omitted if unset | Behavior during grace period. |
| `expiry_warning_days` | Integer / Null | $\ge 0$ | Omitted if unset | Advance warning banner days before expiry. |
| `max_clock_skew_hours` | Integer / Null | $\ge 0$ | Omitted if unset | Backward clock-skew tolerance in hours. |
| `quotas` | Dict / Null | Object or omitted | Omitted if unset | Extensible numeric quotas. |
| `metadata` | Dict / Null | Object or omitted | Omitted if unset | Extensible key-value metadata. |

---

## 3. The 10 Commercial Application Features

| Feature Key | Application Tab | Persian Title | Default in Generator |
|---|---|---|---|
| `dashboard` | Tab 0 | 📊 داشبورد | `true` |
| `products` | Tab 1 | 📦 مدیریت کالاها | `true` |
| `batches` | Tab 2 | 🚚 ثبت خرید و محموله‌ها | `true` |
| `sales` | Tab 3 | 🛒 ثبت فروش | `true` |
| `services` | Tab 4 | 🛠️ میز خدمت | `true` |
| `upgrades` | Tab 5 | 🔧 ارتقای کالا | `true` |
| `suppliers_payments` | Tab 6 | 💳 تأمین‌کنندگان | `true` |
| `reports` | Tab 7 | 📈 گزارش‌های جامع | `true` |
| `employees` | Tab 8 | 👤 مدیریت کاربران | `true` |
| `settings` | Tab 9 | ⚙️ تنظیمات | `true` |

---

## 4. Hardware ID Format & Validation Rules

- **Format:** `fp_v1:<64-lowercase-hex>` (SHA-256 of `DAQIQ-HWFP-V1|<CANONICAL_STRING>`).
- **Sanitization:** Strip leading/trailing whitespace, quotes, and invisible Persian Unicode characters (LRM `\u200E`, RLM `\u200F`, ZWNJ `\u200C`, ZWJ `\u200D`).
- **Validation:** Case-insensitive prefix `fp_v1:` followed by exactly 64 hexadecimal characters.
- **Request File Format (`.daqreq`):**
  ```json
  {
    "request_version": 1,
    "product_id": "daqiq.desktop",
    "generated_at": "2026-11-22T00:00:00Z",
    "device_fingerprint": "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e",
    "device_label": "دفتر فروش - صندوق ۱",
    "app_version": "4.9.1",
    "hardware_tier": "tier1"
  }
  ```
