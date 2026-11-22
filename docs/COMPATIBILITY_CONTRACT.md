# Daqiq Licensing Compatibility Contract (v1.0.0)

**Single Source of Truth:** `https://github.com/ouncommono/Daqiq/pull/38` (Daqiq v4.9.0)  
**Contract Version:** 1.0.0  
**Target Applications:**  
- Issuer: **Daqiq License Generator v1.0.0** (`daqiq.license_generator`)  
- Consumer: **Daqiq Desktop v4.9.0** (`daqiq.desktop`)  
- Device Request Utility: **Companion App A** (`daqiq.request_utility`)

---

## 1. Cryptographic Protocol & Envelope

### 1.1 Envelope Structure
The `.daqlic` file is an offline JSON document encoded in **UTF-8 without BOM**, limited to a maximum file size of **65,536 bytes (64 KiB)**.

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
      "name": "نام مشتری یا فروشگاه",
      "organization": "مجموعه نمونه (اختیاری)",
      "contact": "+989120000000"
    },
    "license_type": "subscription",
    "issued_at": "2026-11-22T00:00:00Z",
    "not_before": "2026-11-22T00:00:00Z",
    "expires_at": "2027-11-22T00:00:00Z",
    "max_devices": 1,
    "device_fingerprints": [
      "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e"
    ],
    "features": {
      "dashboard": true,
      "products": true,
      "batches": true,
      "sales": true,
      "services": true,
      "upgrades": true,
      "suppliers_payments": true,
      "reports": true,
      "employees": true,
      "settings": true
    },
    "min_app_version": "4.9.0",
    "max_app_version": null
  },
  "signature": "<base64-standard-rfc4648-64-byte-ed25519-signature>"
}
```

### 1.2 Cryptographic Parameters
- **Algorithm:** Ed25519 (Edwards-curve Digital Signature Algorithm over Curve25519, RFC 8032).
- **Public Key Representation:** Raw 32 bytes encoded in standard Base64 (e.g. `tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=`). In Daqiq Desktop, registered in `licensing/keys.py` within `DEFAULT_KEY_REGISTRY`.
- **Private Key Storage:** Never stored in plaintext. Encrypted in vendor vault files using AES-256-GCM with PBKDF2-HMAC-SHA256 (600,000 iterations) or Scrypt key derivation.
- **Domain Separation Prefix:**
  ```python
  DOMAIN_PREFIX = b"DAQIQ-LICENSE-V1\n"
  ```
- **Signature Encoding:** Standard Base64 (RFC 4648), exactly 64 raw bytes decoding from 88 Base64 characters (with `=` padding).

### 1.3 Canonical Serialization Algorithm
To guarantee byte-for-byte reproducibility across all platforms:
1. **Type Strictness:** Payload must contain only `dict`, `list`, `str`, `int`, `bool`, and `None`. Floats (`float`), `NaN`, and `Infinity` are strictly forbidden and rejected.
2. **Unicode Normalization:** Recursively normalize all dictionary keys and string values to **Unicode Normalization Form C (NFC)** via `unicodedata.normalize("NFC", text)`.
3. **JSON Serialization:**
   ```python
   canonical_bytes = json.dumps(
       normalized_payload_dict,
       sort_keys=True,
       separators=(",", ":"),
       ensure_ascii=False,
   ).encode("utf-8")
   ```
4. **Signing Payload Construction:**
   ```python
   message_to_sign = DOMAIN_PREFIX + canonical_bytes
   signature = private_key.sign(message_to_sign)
   signature_b64 = base64.b64encode(signature).decode("ascii")
   ```
5. **Daqiq Verification Parity:**
   In Daqiq Desktop (`licensing/license_verifier.py`), the verified payload dictionary is constructed strictly from `LicensePayload.to_dict()`, ensuring the exact 14 payload fields are serialized during verification.

---

## 2. Payload Field Specification (14 Strict Fields)

| Field Name | Type | Allowed Values | Description |
|---|---|---|---|
| `format_version` | Integer | `1` | Schema version identifier. |
| `product_id` | String | `"daqiq.desktop"` | Target application identifier. |
| `license_id` | String | Non-empty ASCII/Unicode | Unique vendor license number (e.g. `DQL-1405-0001` or `LIC-2026-000001`). |
| `key_id` | String | Non-empty ASCII | Identifier of the Ed25519 public key registered in Daqiq's `KeyRegistry`. |
| `customer.name` | String | Non-empty string | Customer or store name in Persian/English (NFC normalized). |
| `customer.organization` | String / Null | Optional string | Organization or commercial branch name. |
| `customer.contact` | String / Null | Optional string | Support phone, email, or contact ID. |
| `license_type` | String | `"perpetual"` or `"subscription"` | Licensing model. |
| `issued_at` | String | ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`) | License issuance timestamp. |
| `not_before` | String | ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`) | Validity start timestamp. |
| `expires_at` | String / Null | `null` for perpetual; ISO-8601 UTC for subscription | Expiration timestamp (`expires_at > not_before`). |
| `max_devices` | Integer | Integer `>= 1` | Maximum authorized hardware devices. |
| `device_fingerprints` | List[String] | Array of `fp_v1:<64-hex>` | List of unique hardware IDs. `1 <= len <= max_devices`. |
| `features` | Dict[str, bool] | 10 boolean flags | Map of feature identifiers to boolean authorization. |
| `min_app_version` | String / Null | e.g. `"4.9.0"` or `null` | Minimum required version of Daqiq Desktop. |
| `max_app_version` | String / Null | e.g. `"5.0.0"` or `null` | Maximum allowed version of Daqiq Desktop. |

---

## 3. Hardware Fingerprint Specification (`fp_v1`)

### 3.1 Syntax & Format
- **Pattern:** `^fp_v1:[0-9a-f]{64}$`
- **Prefix:** `fp_v1:`
- **Hash Algorithm:** SHA-256 over `DAQIQ-HWFP-V1|<CANONICAL_STRING>`
- **Normalization:** Uppercase ASCII, single-space collapsed, trimmed, NFC normalized.

### 3.2 Canonical Component Construction
- **Tier 1 (UUID + Motherboard Serial):**  
  `V1|UUID=<UUID>|BOARD=<SERIAL>`
- **Tier 2 (UUID + Motherboard Model):**  
  `V1|UUID=<UUID>|BOARD_MODEL=<MFG>:<PROD>`
- **Tier 3 (Motherboard Serial + Motherboard Model):**  
  `V1|BOARD=<SERIAL>|BOARD_MODEL=<MFG>:<PROD>`

### 3.3 OEM Placeholder Blacklist
- **UUIDs Rejected:** Empty, `00000000-0000-0000-0000-000000000000`, `ffffffff-ffff-ffff-ffff-ffffffffffff`, `03000200-0400-0500-0006-000700080009`, repeated characters.
- **Board Serials Rejected:** Empty, `none`, `n/a`, `na`, `not applicable`, `not specified`, `default string`, `to be filled by o.e.m.`, `system serial number`, `chassis serial number`, `base board serial number`, `oem`, `invalid`, `unknown`, `0`, `00000000`, `123456789`, repeated single characters.

---

## 4. Feature Entitlements & Tab Mapping (10 Modules)

| Feature Key | Daqiq Module / Tab | Description |
|---|---|---|
| `dashboard` | 📊 داشبورد | Overview dashboard, KPIs, quick metrics |
| `products` | 📦 مدیریت کالاها | Inventory and product catalog |
| `batches` | 🚚 ثبت خرید و محموله‌ها | Procurement batches and suppliers |
| `sales` | 🛒 ثبت فروش | Point-of-sale, invoicing, barcode checkouts |
| `services` | 🛠️ میز خدمت | Repair services, warranty tracking |
| `upgrades` | 🔧 ارتقای کالا | Assembly, hardware upgrades, parts consumption |
| `suppliers_payments` | 💳 تأمین‌کنندگان | Supplier balance and settlement ledger |
| `reports` | 📈 گزارش‌های جامع | Financial reports, sales history, audit logs |
| `employees` | 👤 مدیریت کاربران | User roles, permissions, cashier accounts |
| `settings` | ⚙️ تنظیمات | App configuration, backup/restore, tax settings |

---

## 5. Device Request Contract (`.daqreq`)

When a client exports system hardware specifications:
```json
{
  "request_version": 1,
  "product_id": "daqiq.desktop",
  "generated_at": "2026-11-22T00:00:00Z",
  "device_fingerprint": "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e",
  "device_label": "صندوق اصلی فروشگاه (Main Terminal)",
  "app_version": "4.9.0",
  "hardware_tier": "tier1"
}
```

---

## 6. Official Golden Test Vector (Spec Section 3.2 Parity)

- **Test Public Key (Base64 raw 32 bytes):**  
  `tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=`
- **Domain Separated Message SHA-256 Digest:**  
  `52bdf69ad45d565685bae87f75c03d4a5baa43a3acafe785499b16ee5618366a`
- **Ed25519 Signature (Base64):**  
  `kgrF5f89h9JBIHiQ55H5J+jZul3ZwC6NdeEDXtrlxMDWWlBr892vliqLsTxPD0q+RlEJaXsIQZUSjaevKSGjCA==`
