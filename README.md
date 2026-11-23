# استودیو صدور لایسنس دقیق (Daqiq License Generator Studio v1.4.2)
### Commercial Offline License Studio for Daqiq Desktop ERP (v4.9.2+ — PR #40)
**100% Self-Contained Zero-Dependency Native Win32 Rust Desktop Application**

**Daqiq License Generator v1.4.2** یک نرم‌افزار تجاری رده سازمانی، کاملاً آفلاین (Air-Gapped) و بازنویسی‌شده به‌صورت **۱۰۰٪ Native Win32 Rust بدون هیچ‌گونه وابستگی به WebView2، Edge، Python یا Visual C++ Redistributable** است. این نسخه با **پیوند استاتیک کامل C-Runtime (`+crt-static`)**، **تطابق دقیق ۱:۱ ("تطابق کامل. نه کمتر نه بیشتر")** با ساختار اعتبارسنجی **نرم‌افزار دقیق (Daqiq Desktop ERP v4.9.2 — PR #40)**، محاسبه و نمایش اثرانگشت کلید بر اساس الگوریتم رسمی **Key FP**، گاوصندوق کلید پایدار دائمی (`vault.dqkey`) و بازطراحی بصری رابط کاربری در تراز بین‌المللی (کیفیت Linear و Stripe) عرضه شده است.

---

## 🚀 ویژگی‌های کلیدی و نوآوری‌های نسخه ۱.۴.۲ (v1.4.2 Commercial Highlights)

1. **تطابق کامل ۱:۱ با وریفایر Daqiq PR #40 (v4.9.2) و رفع قطعی خطای امضا:**
   - رفع دائمی خطای «امضای دیجیتال لایسنس نامعتبر است یا محتوای فایل دستکاری شده است».
   - پشتیبانی از تمام فیلدهای استاندارد PR #40 اعم از سیاست تطابق سخت‌افزار (`match_policy`)، مهلت ارفاق (`grace_period_days`, `grace_mode`)، هشدار انقضا، خطای ساعت، سهمیه‌ها (`quotas`) و متادیتا.
   - تولید لایسنس‌های امضاشده با Ed25519 و سریال‌سازی استاندارد RFC 8785 (NFC Canonical JSON) با پیشوند تفکیک قلمرو `DAQIQ_LICENSE_SIG_V1:`.

2. **الگوریتم رسمی اثرانگشت کلید (Official Key FP Parity):**
   - پیاده‌سازی دقیق فرمت اثرانگشت کلید عمومی مطابق با الگوریتم وریفایر Daqiq PR #40:
     $$\text{Key FP} = \text{hex}(\text{SHA256}(\text{raw\_pub\_bytes})).\text{upper}()[0..4] + \text{'-'} + [4..8] + \text{'-'} + [8..12]$$
     (۱۴ نویسه، مانند `7925-3864-0164` برای کلید رسمی `daqiq-prod-v1`).
   - فرمت نمایشی استاندارد: `daqiq-prod-v1 · 7925-3864-0164`.
   - نشانگر زنده وضعیت جفت‌شدگی (Pairing Status Badges) با نرم‌افزار رسمی دقیق در سایدبار ناوبری، نوار عملیات صدور و زبانه گاوصندوق.

3. **گاوصندوق امن و پایدار دائمی (Persistent Key Vault Engine):**
   - ذخیره‌سازی و بارگذاری پایدار جفت‌کلیدها در مسیر امن:
     `%LOCALAPPDATA%\DaqiqLicenseGenerator\vault.dqkey`
   - کلیدهای ایجادشده یا واردشده توسط صادرکننده هرگز با بستن برنامه یا راه‌اندازی مجدد سیستم از بین نمی‌روند.
   - رمزنگاری فایل گاوصندوق با الگوریتم AES-256-GCM و مشتق‌سازی کلید با PBKDF2-HMAC-SHA256 (۶۰۰,۰۰۰ دور).

4. **بازطراحی بصری مدرن و تراز بین‌المللی (International-Grade UI/UX Redesign):**
   - طراحی تخت، مینیمال و دقیق با دقت پیکسلی مشابه Linear، Raycast و Stripe در هر دو تم **Dark** (زغالی تیره عمیق `#171A21`/`#1F242E`) و **Light** (روشن کریستالی `#F3F5F9`/`#FFFFFF`).
   - فیلدهای ورودی تخت، بدون حاشیه برجسته ۳D قدیمی ویندوز، با تایپوگرافی تمیز و فاصله استاندارد.
   - رندرینگ فوق‌العاده روان 0ms Native بدون کوچک‌ترین لگ یا مصرف منابع گرافیکی سنگین.

5. **پیوند استاتیک CRT و عدم نیاز به پیش‌نیازها (Static CRT & Zero External Dependencies):**
   - پیوند کامل و استاتیک کتابخانه زمان اجرای مایکروسافت (`-C target-feature=+crt-static` در MSVC).
   - حذف کامل هرگونه وابستگی به `VCRUNTIME140.dll`، `MSVCP140.dll` و عدم نیاز به نصب Visual C++ Redistributable.
   - اجرای آنی و ۱۰۰٪ مستقل روی ویندوزهای تازه نصب‌شده، آفلاین و ایزوله (Air-Gapped).
   - ساب‌سیستم گرافیکی ۲ ویندوز (`Subsystem == 2`) با تضمین عدم نمایش پنجره سیاه CMD.

6. **ردیاب گام‌به‌گام مراحل راه‌اندازی و اعلام خطای بومی:**
   - ثبت دقیق زمان و وضعیت تمامی ۱۴ مرحله راه‌اندازی در:
     `%LOCALAPPDATA%\DaqiqLicenseGenerator\startup.log`
   - زنجیره ثبت مراحل:
     `BOOT_START -> PANIC_HOOK_OK -> DATA_DIR_OK -> CONFIG_OK -> DB_OK -> COM_OK -> COMMCTRL_OK -> FONT_OK -> CLASS_REG_OK -> MAIN_HWND_OK -> PANELS_CREATED_OK -> LAYOUT_OK -> SHOW_WINDOW_OK -> ENTER_MSG_LOOP`
   - پنجره بومی `MessageBoxW` دوزبانه در صورت وقوع خطای بحرانی با نمایش کد خطای سیستم‌عامل (`GetLastError()`).

7. **شش دروازه آزمون و تایید انتشار (6 Release Gates Passed 100%):**
   - **Gate 1:** بازرسی هدر PE، ساب‌سیستم گرافیکی ۲ و ممنوعیت وابستگی‌های VC++ (`verify_pe.py`). حجم باینری: **۱۱.۸۹ مگابایت** (زیر سقف ۱۵ مگابایت).
   - **Gate 2:** استعلام نسخه با پرچم `--version` (خروجی دقیق `1.4.2`).
   - **Gate 3:** آزمون متقابل سازگاری با وریفایر پایتون Daqiq PR #40 (۷ از ۷ آزمون موفق شامل آزمون Key FP).
   - **Gate 4:** خودآزمایی هدلس هسته رمزنگاری (`--self-test --report dist\selftest_report.json`).
   - **Gate 5:** خودآزمایی هدلس رابط کاربری واقعی Win32 با ۴ زبانه (`--self-test-ui --report dist\ui_test_report.json`).
   - **Gate 6:** آزمون شبیه‌سازی واقعی دابل‌کلیک کاربر بدون پرچم (`tests/test_visible_launch.ps1`) همراه با تایید نمایش پنجره، ابعاد هندسی و ثبت کامل ۱۴ مرحله در `startup.log`.

---

## 🛠️ ساختار مهندسی پروژه (System Architecture)

```
Daqiq-License-Generator/
├── .cargo/
│   └── config.toml                  # فعال‌سازی پیوند استاتیک CRT برای MSVC (+crt-static)
├── Cargo.toml                       # پیکربندی ورک‌اسپیس Rust و بهینه‌سازی سایز
├── crates/
│   └── daqiq-lic-core/              # هسته محاسباتی، رمزنگاری و منطق کسب‌وکار
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── crypto.rs            # سریال‌سازی RFC 8785 و امضای Ed25519
│           ├── schema.rs            # مشخصات داده‌ای و توکن‌های PR #40 و کلیدهای پیش‌فرض
│           ├── hwid.rs              # اعتبارسنجی شناسه‌های سخت‌افزاری fp_v1
│           ├── jalali.rs            # تبدیل تاریخ شمسی و میلادی UTC
│           ├── vault.rs             # گاوصندوق، الگوریتم Key FP، ذخیره دیسک و اسکنر باینری ۵۱۲KB
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
│           ├── startup_log.rs       # ردیاب گام‌به‌گام مراحل راه‌اندازی (startup.log)
│           ├── config.rs            # ذخیره و بارگذاری تنظیمات کاربر در AppData
│           ├── theme.rs             # پالت رنگی مدرن تیره زغالی و روشن شفاف
│           ├── rtl.rs               # محاسبات تقویم شمسی
│           ├── state.rs             # مدیریت وضعیت اپلیکیشن، ماندگاری گاوصندوق و منطق کاربری
│           ├── win32_util.rs        # کنترل‌های مدرن تخت Win32، قلم‌ها و توابع ترسیم
│           └── panels/
│               ├── mod.rs
│               ├── sidebar.rs       # سایدبار ناوبری و نشانگر تطابق Key FP
│               ├── generator.rs     # زبانه ۱: صدور لایسنس تجاری و نشانگر جفت‌شدگی
│               ├── ledger.rs        # زبانه ۲: دفتر کل لایسنس‌ها
│               ├── inspector.rs     # زبانه ۳: بازرسی، ارزیابی و شبیه‌ساز PR #40
│               └── vault.rs         # زبانه ۴: گاوصندوق، ماندگاری دیسک و اسکنر باینری Daqiq.exe
├── scripts/
│   └── verify_pe.py                 # بازرسی هدر PE، ساب‌سیستم و ممنوعیت VC++ Runtime
├── tests/
│   ├── test_cross_repo_compatibility.py # آزمون سازگاری متقابل با Daqiq PR #40 و Key FP
│   └── test_visible_launch.ps1      # دروازه ۶: آزمون لانچ مرئی و واقعی پنجره
├── docs/
│   ├── ROOT_CAUSE_V1_4_1.md         # گزارش تحلیل ریشه‌ای اشکالات راه‌اندازی و معماری v1.4.1
│   ├── COMPATIBILITY_CONTRACT.md    # قرارداد رسمی تطابق بایت‌به‌بایت با Daqiq v4.9.2 (PR #40)
│   ├── ROOT_CAUSE_V1_4_0.md         # گزارش تحلیل ریشه‌ای اشکالات راه‌اندازی نسخه ۱.۴.۰
│   └── ROOT_CAUSE_V1_3_0.md         # گزارش تحلیل ریشه‌ای اشکالات نسخه ۱.۳.۰
├── dist/                            # گزارش‌ها و خروجی‌های تست‌های خودکار
│   ├── selftest_report.json         # گزارش آزمون هسته سیستم (Gate 4)
│   └── ui_test_report.json          # گزارش آزمون رابط کاربری Win32 (Gate 5)
└── VERSION                          # شماره نسخه رسمی (1.4.2)
```

---

## 💻 دستورالعمل اجرا و کامپایل محلی (Local Build & Execution)

### ۱. اجرای آزمون‌های هسته رمزنگاری:
```powershell
cargo test -p daqiq-lic-core --no-default-features
```

### ۲. کامپایل نسخه نهایی پرتابل با پیوند استاتیک CRT:
```powershell
cargo build --release -p daqiq-license-generator --no-default-features
```

### ۳. اعتبارسنجی شش دروازه ترخیص (6 Release Gates):
```powershell
# Gate 1: بازرسی هدر باینری و سقف حجم (< 15 MB) بدون وابستگی‌های زمان اجرا
python scripts/verify_pe.py target/release/daqiq-license-generator.exe 15.0

# Gate 2: استعلام نسخه رسمی (1.4.2)
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--version" -Wait -NoNewWindow

# Gate 3: آزمون متقابل سازگاری با کد پایتون Daqiq PR #40 (شامل Key FP)
python -m pytest tests/test_cross_repo_compatibility.py -v

# Gate 4: خودآزمایی هدلس هسته رمزنگاری
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--self-test","--report","dist\selftest_report.json" -Wait -NoNewWindow

# Gate 5: خودآزمایی هدلس رابط کاربری واقعی Win32
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--self-test-ui","--report","dist\ui_test_report.json" -Wait -NoNewWindow

# Gate 6: آزمون لانچ واقعی و مرئی پنجره و بررسی لاگ راه‌اندازی
powershell -ExecutionPolicy Bypass -File tests/test_visible_launch.ps1
```
