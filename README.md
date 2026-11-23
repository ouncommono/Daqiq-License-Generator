# استودیو صدور لایسنس دقیق (Daqiq License Generator Studio v1.4.1)
### Commercial Offline License Studio for Daqiq Desktop ERP (v4.9.1+ — PR #39)
**100% Self-Contained Zero-Dependency Native Win32 Rust Desktop Application**

**Daqiq License Generator v1.4.1** یک نرم‌افزار تجاری رده سازمانی، کاملاً آفلاین (Air-Gapped) و بازنویسی‌شده به‌صورت **۱۰۰٪ Native Win32 Rust بدون هیچ‌گونه وابستگی به WebView2، Edge، Python یا Visual C++ Redistributable** است. این نسخه با **پیوند استاتیک کامل C-Runtime (`+crt-static`)**، رفع قطعی اشکالات چرخه حیات پنجره در ویندوز، ردیاب جامع مراحل راه‌اندازی (Startup Breadcrumb Logger) و **تطابق دقیق ۱:۱ ("تطابق کامل. نه کمتر نه بیشتر")** با ساختار اعتبارسنجی **نرم‌افزار دقیق (Daqiq Desktop ERP v4.9.1 — PR #39)** عرضه شده است.

---

## 🚀 ویژگی‌های کلیدی و اصلاحات نسخه ۱.۴.۱ (v1.4.1 Hotfix Highlights)

1. **پیوند استاتیک CRT و رفع کامل خطای فقدان DLL (Static CRT & Zero-Dependency Execution):**
   - پیوند کامل و استاتیک کتابخانه زمان اجرای مایکروسافت (`-C target-feature=+crt-static` در MSVC).
   - حذف کامل هرگونه وابستگی به `VCRUNTIME140.dll`، `MSVCP140.dll` و عدم نیاز به نصب Visual C++ Redistributable بر روی سیستم کاربر نهایی.
   - اجرای آنی و ۱۰۰٪ مستقل روی ویندوزهای تازه نصب‌شده، آفلاین و بدون ابزارهای جانبی.

2. **ردیاب گام‌به‌گام مراحل راه‌اندازی (Startup Breadcrumb Tracer):**
   - ثبت دقیق زمان و وضعیت تمامی ۱۴ مرحله راه‌اندازی در مسیر:
     `%LOCALAPPDATA%\DaqiqLicenseGenerator\startup.log`
   - زنجیره ثبت مراحل:
     `BOOT_START -> PANIC_HOOK_OK -> DATA_DIR_OK -> CONFIG_OK -> DB_OK -> COM_OK -> COMMCTRL_OK -> FONT_OK -> CLASS_REG_OK -> MAIN_HWND_OK -> PANELS_CREATED_OK -> LAYOUT_OK -> SHOW_WINDOW_OK -> ENTER_MSG_LOOP`

3. **سیستم هشدار بومی ویندوز برای خطاها (Native Win32 Error Reporting):**
   - در صورت بروز هرگونه خطای راه‌اندازی یا پنیک برنامه‌نویسی، یک پنجره بومی `MessageBoxW` با متن دوزبانه فارسی/انگلیسی، جزئیات دقیق فنی و کد خطای سیستم‌عامل (`GetLastError()`) نمایش داده می‌شود تا از خروج خاموش (Silent Exit) جلوگیری شود.

4. **اصلاح مهندسی چرخه حیات و هندسه پنجره (Safe Win32 Window Lifecycle):**
   - تفکیک صحیح و استاندارد `dwExStyle` (`WS_EX_APPWINDOW | WS_EX_CONTROLPARENT`) از `dwStyle` (`WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS`).
   - تخصیص صحیح `hInstance` ماژول به کلاس و توابع ساخت پنجره (`GetModuleHandleW`).
   - محاسبه خودکار و هوشمند مرکز صفحه نمایش بر اساس فضای کاری مانیتور کاربر (`SPI_GETWORKAREA`).
   - پشتیبانی از تم تاریک نوار عنوان با سازگاری برگشتی به نسخه‌های قدیمی‌تر ویندوز ۱۰ (DWM attributes 20 و 19).

5. **پایگاه داده پایدار با مهاجرت امن (Persistent SQLite Registry & Safe Migration):**
   - ذخیره‌سازی مطمئن تاریخچه لایسنس‌ها در مسیر دائمی `%LOCALAPPDATA%\DaqiqLicenseGenerator\registry.sqlite`.
   - بازیابی و پشتیبان‌گیری خودکار در صورت خرابی احتمالی فایل بدون ایجاد کرش در برنامه.

6. **تطابق کامل ۱:۱ با وریفایر Daqiq PR #39 (v4.9.1):**
   - تولید لایسنس‌های امضاشده با Ed25519 و سریال‌سازی استاندارد RFC 8785 (NFC Canonical JSON) با پیشوند تفکیک قلمرو `DAQIQ_LICENSE_SIG_V1:`.
   - توکن‌های فشرده فشرده‌سازی‌شده Base64 و اسکنر مستقیم باینری ۵۱۲ کیلوبایتی `Daqiq.exe`.

7. **شش دروازه آزمون و تایید انتشار (6 Release Gates Passed 100%):**
   - **Gate 1:** بازرسی هدر PE، ساب‌سیستم گرافیکی ۲ و ممنوعیت وابستگی‌های VC++ (`verify_pe.py`).
   - **Gate 2:** استعلام نسخه با پرچم `--version` (خروجی `1.4.1`).
   - **Gate 3:** آزمون متقابل سازگاری با وریفایر پایتون Daqiq PR #39 (۶ از ۶ آزمون موفق).
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
│           ├── startup_log.rs       # ردیاب گام‌به‌گام مراحل راه‌اندازی (startup.log)
│           ├── config.rs            # ذخیره و بارگذاری تنظیمات کاربر در AppData
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
│   └── verify_pe.py                 # بازرسی هدر PE، ساب‌سیستم و ممنوعیت VC++ Runtime
├── tests/
│   ├── test_cross_repo_compatibility.py # آزمون سازگاری متقابل با Daqiq PR #39
│   └── test_visible_launch.ps1      # دروازه ۶: آزمون لانچ مرئی و واقعی پنجره
├── docs/
│   ├── ROOT_CAUSE_V1_4_0.md         # گزارش تحلیل ریشه‌ای اشکالات راه‌اندازی نسخه ۱.۴.۰
│   ├── COMPATIBILITY_CONTRACT.md    # قرارداد رسمی تطابق بایت‌به‌بایت با Daqiq v4.9.1
│   └── ROOT_CAUSE_V1_3_0.md         # گزارش تحلیل ریشه‌ای اشکالات نسخه ۱.۳.۰
├── dist/                            # گزارش‌ها و خروجی‌های تست‌های خودکار
│   ├── selftest_report.json         # گزارش آزمون هسته سیستم (Gate 4)
│   └── ui_test_report.json          # گزارش آزمون رابط کاربری Win32 (Gate 5)
└── VERSION                          # شماره نسخه رسمی (1.4.1)
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

# Gate 2: استعلام نسخه رسمی (1.4.1)
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--version" -Wait -NoNewWindow

# Gate 3: آزمون متقابل سازگاری با کد پایتون Daqiq PR #39
python -m pytest tests/test_cross_repo_compatibility.py -v

# Gate 4: خودآزمایی هدلس هسته رمزنگاری
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--self-test","--report","dist\selftest_report.json" -Wait -NoNewWindow

# Gate 5: خودآزمایی هدلس رابط کاربری واقعی Win32
Start-Process target/release/daqiq-license-generator.exe -ArgumentList "--self-test-ui","--report","dist\ui_test_report.json" -Wait -NoNewWindow

# Gate 6: آزمون لانچ واقعی و مرئی پنجره و بررسی لاگ راه‌اندازی
powershell -ExecutionPolicy Bypass -File tests/test_visible_launch.ps1
```
