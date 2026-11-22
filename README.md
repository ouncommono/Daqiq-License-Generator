# مولد لایسنس دقیق — Daqiq License Generator v1.0.0
### Commercial Offline License Studio for Daqiq Desktop (v4.9.0+)

**Daqiq License Generator** is an enterprise-grade, 100% offline desktop application engineered for generating, managing, and verifying asymmetric cryptographic licenses (`.daqlic`) for **Daqiq Desktop ERP** ([PR #38](https://github.com/ouncommono/Daqiq/pull/38)).

---

## 🔒 ویژگی‌های کلیدی و امنیتی (Key Features)

1. **سازگاری صد در صد با پروتکل لایسنس دقیق (PR #38):**
   - امضای دیجیتال نامتقارن با الگوریتم **Ed25519 (RFC 8032)**.
   - سریال‌سازی استاندارد و یکسان بر مبنای استاندارد **Unicode NFC** و مرتب‌سازی کلیدها (`sort_keys=True`).
   - پشتیبانی کامل از شناسه سخت‌افزاری **`fp_v1`** (گره‌های سخت‌افزاری پایدار مادربورد و بایوس).
2. **گاوصندوق رمزنگاری‌شده کلیدهای امضا (Encrypted Key Vault):**
   - ذخیره‌سازی کلیدهای خصوصی منحصراً به صورت رمزنگاری‌شده با **AES-256-GCM** و استخراج کلید با **PBKDF2-HMAC-SHA256 (۶۰۰٬۰۰۰ دور)**.
   - حفظ کلید خصوصی منحصراً در حافظه رم (RAM) و امکان قفل فوری با یک کلیک.
   - عدم ذخیره هیچ کلید محرمانه‌ای در تاریخچه Git یا دیسک به صورت متن واضح (Plaintext).
3. **پشتیبانی کامل از چند دستگاهی (Multi-Device Binding):**
   - ثبت یک یا چند شناسه سخت‌افزاری با اعتبارسنجی زنده ساختار `fp_v1`.
   - امکان افزودن گروهی شناسه‌ها (Bulk Paste) و حذف نویسه‌های نامرئی، فاصله‌ها و کاراکترهای جهت متن (RLM/LRM).
   - بارگذاری مستقیم فایل‌های درخواست سیستم (`.daqreq`).
4. **زمان‌بندی تقویم جلالی (Jalali-First):**
   - انتخاب‌گر تقویم شمسی برای تاریخ‌های شروع و پایان لایسنس با تبدیل دقیق به UTC ISO-8601.
   - دکمه‌های سریع برای دوره‌های پرکاربرد (۷ روز، ۱ ماه، ۳ ماه، ۶ ماه، ۱ سال، ۲ سال، دائمی).
5. **دفتر کل سوابق لایسنس‌ها (Offline License Registry):**
   - پایگاه داده محلی SQLite با قابلیت جستجوی آنی مشتری، شماره تماس و شناسه لایسنس.
   - امکان «تمدید سریع (Clone & Renew)» با یک کلیک، ذخیره مجدد فایل و خروجی CSV/Excel.
6. **بازرس و شبیه‌ساز لایسنس (License Inspector & Simulator):**
   - بررسی صحت رمزنگاری و نمایش اجزای فایل‌های `.daqlic`.
   - شبیه‌ساز رفتار لایسنس در تاریخ‌های مختلف و با شناسه‌های سخت‌افزاری گوناگون در Daqiq.
7. **طراحی حرفه‌ای راست‌به‌چپ (Persian RTL) با PySide6:**
   - دو تم مدرن روشن (Light) و تیره (Dark) هماهنگ با هویت بصری نرم‌افزار دقیق.

---

## 🛠️ ساختار ماژول‌ها و معماری (Architecture)

```
Daqiq-License-Generator/
├── core/
│   ├── key_vault.py        # گاوصندوق رمزنگاری کلیدهای امضا (AES-256-GCM)
│   ├── hwid_parser.py      # تحلیل، پاک‌سازی و اعتبارسنجی شناسه‌های سخت‌افزاری
│   ├── license_builder.py  # ساخت پاکت لایسنس، امضا و خود-اعتبارسنجی اتمیک
│   └── registry_db.py      # پایگاه داده محلی سوابق لایسنس‌های صادرشده
├── ui/
│   ├── theme.py            # استایل‌های تیره و روشن، آیکون‌ها و تایپوگرافی
│   ├── jalali_picker.py    # ویجت تعاملی انتخاب تاریخ جلالی
│   ├── tab_generator.py    # تب ۱: استودیوی تولید لایسنس
│   ├── tab_ledger.py       # تب ۲: دفتر کل سوابق لایسنس‌ها
│   ├── tab_inspector.py    # تب ۳: بازرس و شبیه‌ساز لایسنس
│   ├── tab_vault.py        # تب ۴: مدیریت گاوصندوق کلیدها و تنظیمات
│   └── main_window.py      # پنجره اصلی برنامه با چیدمان راست‌به‌چپ
├── docs/
│   └── COMPATIBILITY_CONTRACT.md # سند رسمی سازگاری با Daqiq PR #38
├── tests/                  # آزمون‌های خودکار هِدلس
└── main.py                 # نقطه ورود برنامه و اجرای رابط کاربری
```

---

## 🚀 نحوه نصب و اجرا (Quick Start)

### ۱. نصب وابستگی‌ها
```bash
pip install -r requirements.txt
pip install -r requirements-build.txt
```

### ۲. اجرای برنامه
```bash
python main.py
```

### ۳. اجرای آزمون‌های هِدلس (Headless Automated Tests)
```bash
# ویندوز (PowerShell)
$env:QT_QPA_PLATFORM="offscreen"
pytest -v

# لینوکس / CI
QT_QPA_PLATFORM=offscreen pytest -v
```

---

## 🔑 سازگاری با کد Daqiq Desktop

کلید عمومی تولیدشده در تب «مدیریت کلیدها» مستقیماً در فایل `licensing/keys.py` نرم‌افزار Daqiq ثبت می‌شود:

```python
DEFAULT_KEY_REGISTRY: Dict[str, str] = {
    "daqiq-prod-v1": "<YOUR_BASE64_PUBLIC_KEY>",
}
```

---

## 📄 مجوز (License)

Commercial Proprietary — تمامی حقوق برای توسعه‌دهندگان Daqiq محفوظ است.
