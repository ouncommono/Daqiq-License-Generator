# استودیو صدور لایسنس دقیق (Daqiq License Generator Studio v1.4.0)
### Commercial Offline License Studio for Daqiq Desktop ERP (v4.9.1+ — PR #39)
**100% Self-Contained Zero-Lag Native Win32 Rust Desktop Application**

**Daqiq License Generator v1.4.0** یک نرم‌افزار تجاری رده سازمانی، کاملاً آفلاین (Air-Gapped) و بازنویسی‌شده به‌صورت **۱۰۰٪ Native Win32 Rust بدون هیچ‌گونه وابستگی به WebView2، Edge، iced یا درایورهای گرافیکی DirectX/Vulkan** است. این ابزار با **تطابق دقیق ۱:۱ ("تطابق کامل. نه کمتر نه بیشتر")** با ساختار اعتبارسنجی **نرم‌افزار دقیق (Daqiq Desktop ERP v4.9.1 — PR #39)**، لایسنس‌های نامتقارن رمزنگاری‌شده (`.daqlic` و توکن‌های فشرده متنی Base64) را با **تاخیر صفر سیستم‌عامل (0 ms Native OS Latency)** تولید، مدیریت، امضا و بازرسی می‌کند.

---

## 🚀 ویژگی‌های کلیدی نسخه ۱.۴.۰ (Key Highlights)

1. **رابط کاربری ۱۰۰٪ Native Win32 در راست خرد (Pure-Rust Win32 GUI):**
   - جایگزینی کامل لایه‌های سنگین یا ناپایدار با کنترل‌های بومی هسته ویندوز (`windows-sys` + Common Controls v6 + PerMonitorV2 HiDPI).
   - رندرینگ سریع و بدون تاخیر (0 ms Input & Scroll Latency) با نقاشی مستقیم دو بافره (Double-Buffered GDI).
   - جاسازی مستقیم قلم‌های فارسی **وزیرمتن (Vazirmatn)** و **جت‌برینز مونو (JetBrains Mono)** در حافظه با API رسمی `AddFontMemResourceEx`.
   - شکل‌دهی پیوسته و بومی حروف فارسی (Uniscribe/DirectWrite RTL Shaping) در تمامی کنترل‌های متنی.
   - تم دوگانه مدرن با تم زغالی ارگونومیک (`#1E222B` / `#262B36`) و تم روشن شفاف (`#F4F6FA`).

2. **رفع قطعی و دائمی خطای رد امضای دیجیتال (Definitive Signature Fix):**
   - تطابق بایت‌به‌بایت و ریاضیاتی فرآیند آماده‌سازی پیام و سریال‌سازی استاندارد RFC 8785 (NFC Unicode + حذف فیلدهای تهی) با کد منبع Daqiq PR #39 (`licensing/license_verifier.py`).
   - تضمین کامل عدم وقوع خطای `"امضای دیجیتال لایسنس نامعتبر است یا محتوای فایل دستکاری شده است."` در Daqiq v4.9.1.

3. **اسکنر تشخیصی تطابق باینری Daqiq.exe (In-App Binary Pairing Scanner):**
   - امکان انتخاب مستقیم فایل کامپایل‌شده `Daqiq.exe` یا فایل کد `licensing/keys.py` از داخل برنامه.
   - اسکن بافرینگ ۵۱۲ کیلوبایتی با همپوشانی ۱۲۸ بایت جهت جستجوی سریع کلید عمومی فعال در تمامی حالت‌های Base64 استاندارد، URL-safe، بایت‌های خام Ed25519 و کدهای هگز.
   - ارائه راهنمایی فارسی و اقدام عملی در صورت عدم تطابق کلید گاوصندوق با کلید تعبیه‌شده در نرم‌افزار دقیق.

4. **رفع کامل هرگونه کرش و ناپایداری (Zero-Crash Guarantee):**
   - پیش‌تخصیص کنترل‌های هر ۴ زبانه بدون وابستگی به موتورهای چیدمان پویا و پرخطا.
   - رفع ریشه‌ای کرش بخش تنظیمات و گاوصندوق.
   - ثبت خودکار جزئیات هرگونه خطای سیستمی در مسیر `%LOCALAPPDATA%\DaqiqLicenseGenerator\crash.log`.

5. **فوق‌العاده کم‌حجم و مستقل (Ultra-Compact Standalone .EXE):**
   - حجم نهایی فایل اجرایی: **۱۱.۸۶ مگابایت** (بسیار کمتر از سقف مجاز ۱۵ مگابایت).
   - ساب‌سیستم گرافیکی ویندوز (`IMAGE_SUBSYSTEM_WINDOWS_GUI`) بدون پرش پنجره سیاه CMD.
   - بدون نیاز به نصب VC++ Redistributable، بدون نیاز به اینترنت و بدون فایل‌های DLL خارجی.

6. **گاوصندوق رمزنگاری‌شده کلیدهای امضا (Encrypted Key Vault):**
   - ذخیره‌سازی کلیدهای اختصاصی با رمزنگاری **AES-256-GCM** و مشتق‌سازی با **PBKDF2-HMAC-SHA256 (۶۰۰٬۰۰۰ دور)**.
   - پاک‌سازی امن حافظه با `zeroize`.
   - عدم درج هیچ‌گونه کلید خصوصی در مخزن گیت.

7. **دفتر کل ثبت لایسنس‌ها با خروجی استاندارد اکسل (SQLite Ledger & Excel BOM):**
   - نگهداری تاریخچه و جستجوی پیشرفته بر اساس نام مشتری، شناسه لایسنس یا شماره تماس.
   - دکمه تمدید سریع (`Clone & Renew`).
   - خروجی فایل اکسل با فرمت CSV دارای نشانگر بایتی **UTF-8 BOM (`\xEF\xBB\xBF`)** برای نمایش صحیح و بدون به‌هم‌ریختگی متون فارسی.

8. **دروازه‌های پنج‌گانه آزمون و تایید انتشار (5 Release Gates Passed 100%):**
   - **Gate 1:** آزمون PE Header و وابستگی‌ها (`scripts/verify_pe.py`) با ساب‌سیستم ۲ و سایز ۱۱.۸۶ مگابایت.
   - **Gate 2:** استعلام نسخه بدون GUI با پرچم `--version` (خروجی `1.4.0`).
   - **Gate 3:** آزمون سازگاری متقابل زبانی Python/Rust در برابر وریفایر واقعی Daqiq PR #39 (۶ از ۶ آزمون موفق).
   - **Gate 4:** آزمون مستقل موتور با پرچم `--self-test --report dist\selftest_report.json` (۱۰۰٪ موفق).
   - **Gate 5:** آزمون رابط کاربری پنجره واقعی و چرخش کامل ۴ زبانه با پرچم `--self-test-ui --report dist\ui_test_report.json` (۱۰۰٪ موفق).

---

## 🛠️ ساختار مهندسی پروژه (System Architecture)

```
Daqiq-License-Generator/
├── Cargo.toml                       # پیکربندی ورک‌اسپیس Rust و بهینه‌سازی LTO
├── crates/
│   └── daqiq-lic-core/              # هسته محاسباتی، رمزنگاری و منطق کسب‌وکار
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── crypto.rs            # سریال‌سازی RFC 8785 و امضای Ed25519
│           ├── schema.rs            # مشخصات داده‌ای و توکن‌های PR #39
│           ├── hwid.rs              # اعتبارسنجی شناسه‌های سخت‌افزاری fp_v1
│           ├── jalali.rs            # تبدیل تاریخ شمسی و میلادی UTC
│           ├── vault.rs             # گاوصندوق، تطابق کلید و اسکنر باینری ۵۱۲KB
│           ├── registry.rs          # پایگاه داده SQLite و خروجی CSV با BOM
│           └── error.rs             # مدیریت جامع خطاهای سیستم
├── apps/
│   └── daqiq-lic-gui/               # رابط کاربری دسکتاپ (100% Native Win32 Rust)
│       ├── Cargo.toml               # windows-sys 0.59 + rfd + arboard
│       ├── build.rs                 # الحاق مانیفست Common Controls v6 و DPI
│       ├── assets/
│       │   ├── icon.ico             # آیکون وکتور اپلیکیشن
│       │   └── fonts/               # قلم‌های آفلاین TTF (Vazirmatn + JetBrains Mono)
│       └── src/
│           ├── main.rs              # نقطه ورود WinMain، پنجره اصلی و تست‌های هدلس
│           ├── config.rs            # ذخیره تنظیمات کاربر در AppData
│           ├── theme.rs             # پالت رنگی تیره زغالی و روشن شفاف
│           ├── rtl.rs               # محاسبات تقویم شمسی
│           ├── state.rs             # مدیریت وضعیت اپلیکیشن و منطق کاربری
│           ├── win32_util.rs        # ابزارهای کمکی Win32، قلم‌ها و کنترل‌ها
│           └── panels/
│               ├── mod.rs
│               ├── sidebar.rs       # سایدبار ناوبری و نشانگر تطابق کلید
│               ├── generator.rs     # زبانه ۱: صدور لایسنس تجاری
│               ├── ledger.rs        # زبانه ۲: دفتر کل لایسنس‌ها
│               ├── inspector.rs     # زبانه ۳: بازرسی، ارزیابی و شبیه‌ساز
│               └── vault.rs         # زبانه ۴: گاوصندوق و اسکنر باینری Daqiq.exe
├── scripts/
│   └── verify_pe.py                 # بازرسی هدر PE، ساب‌سیستم و وابستگی‌های DLL
├── tests/
│   └── test_cross_repo_compatibility.py # آزمون سازگاری متقابل با Daqiq PR #39
├── docs/
│   ├── COMPATIBILITY_CONTRACT.md    # قرارداد رسمی تطابق بایت‌به‌بایت با Daqiq v4.9.1
│   ├── ROOT_CAUSE_V1_3_0.md         # گزارش تحلیل ریشه‌ای اشکالات نسخه ۱.۳.۰
│   ├── ROOT_CAUSE_V1_2_0.md         # گزارش تحلیل ریشه‌ای اشکالات نسخه ۱.۲.۰
│   └── ROOT_CAUSE_V1_0_0.md         # گزارش تحلیل ریشه‌ای نسخه ۱.۰.۰
├── dist/                            # گزارش‌ها و خروجی‌های تست‌های خودکار
│   ├── selftest_report.json         # گزارش آزمون هسته سیستم (Gate 4)
│   └── ui_test_report.json          # گزارش آزمون رابط کاربری Win32 (Gate 5)
├── assets/                          # دارایی‌های تصویری و وکتور
└── VERSION                          # شماره نسخه رسمی (1.4.0)
```

---

## 💻 دستورالعمل اجرا و کامپایل محلی (Local Build & Execution)

### ۱. اجرای آزمون‌های هسته رمزنگاری:
```powershell
cargo test -p daqiq-lic-core --no-default-features
```

### ۲. کامپایل نسخه نهایی پرتابل:
```powershell
cargo build --release -p daqiq-license-generator --no-default-features
```

### ۳. اعتبارسنجی پنج دروازه ترخیص (Release Gates):
```powershell
# Gate 1: بازرسی هدر باینری و سقف حجم (< 15 MB)
python scripts/verify_pe.py target/release/daqiq-license-generator.exe 15.0

# Gate 2: استعلام نسخه رسمی
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--version" -Wait -NoNewWindow

# Gate 3: آزمون متقابل سازگاری با کد پایتون Daqiq PR #39
python -m pytest tests/test_cross_repo_compatibility.py -v

# Gate 4: خودآزمایی هدلس هسته رمزنگاری
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--self-test --report dist\selftest_report.json" -Wait -NoNewWindow

# Gate 5: خودآزمایی هدلس رابط کاربری واقعی Win32
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--self-test-ui --report dist\ui_test_report.json" -Wait -NoNewWindow
```
