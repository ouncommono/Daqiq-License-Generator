# استودیو صدور لایسنس دقیق (Daqiq License Generator Studio v1.2.0)
### Commercial Offline License Studio for Daqiq Desktop ERP (v4.9.0+)
**100% Self-Contained Zero-WebView2 Native Rust Desktop Application**

**Daqiq License Generator v1.2.0** یک نرم‌افزار تجاری رده سازمانی، کاملاً آفلاین (Air-Gapped) و بازنویسی‌شده به‌صورت **۱۰۰٪ Native Rust خالص بدون هیچ‌گونه وابستگی به WebView2، مرورگر Edge یا مفسر پایتون** است. این ابزار برای تولید، مدیریت، امضا و بازرسی لایسنس‌های نامتقارن رمزنگاری‌شده (`.daqlic`) نرم‌افزار **دقیق (Daqiq Desktop ERP v4.9.0 - PR #38)** توسعه یافته است.

---

## 🚀 ویژگی‌های کلیدی نسخه ۱.۲.۰ (Key Features)

1. **حذف کامل WebView2 و وابستگی‌های خارجی (Zero WebView2 / Zero External Runtime):**
   - معماری رابط کاربری دسکتاپ بر پایه موتور گرافیکی بومی **`iced`** با رندرر نرم‌افزاری خودکفا **`tiny-skia`**.
   - اجرا و رندر پایدار روی تمام کامپیوترهای مبتنی بر ویندوز ۱۰ و ۱۱ (از جمله ماشین‌های مجازی، محیط‌های سروری، و سیستم‌های کاملاً آفلاین بدون درایور گرافیک یا GPU).
   - حذف کامل کتابخانه‌های `WebView2Loader.dll`، Edge و فایل‌های اضافی فرانت‌اند وب.
2. **فایل اجرایی مستقل فوق سبک و بدون پنجره کنسول (Standalone Native Binary < 25 MB):**
   - حجم فایل پرتابل نهایی کامپایل‌شده تنها **۵.۵ مگابایت** است (هدف: کمتر از ۲۵ مگابایت).
   - پیوند به ساب‌سیستم گرافیکی ویندوز (`Subsystem = 2 / IMAGE_SUBSYSTEM_WINDOWS_GUI`) بدون کوچک‌ترین پرش یا نمایش پنجره CMD.
   - شروع به کار فوق سریع (زیر ۳۰۰ میلی‌ثانیه) با مصرف حافظه رم حداقلی.
3. **تطابق بایت‌به‌بایت با موتور امنیتی Daqiq PR #38:**
   - امضای دیجیتال استاندارد **Ed25519 (RFC 8032)** منطبق بر کلید رسمی Daqiq (`daqiq-prod-v1: tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=`).
   - پیشوند تفکیک دامنه رمزنگاری: `b"DAQIQ-LICENSE-V1\n"`.
   - سریال‌سازی قطعی (Canonical JSON): نرمال‌سازی نویسه‌ها به فرم **Unicode NFC**، مرتب‌سازی بازگشتی کلیدها (`BTreeMap`)، حفظ فیلدهای اختیاری با مقدار `null` و جداسازهای استاندارد بدون فاصله `(,)` و `(:)`.
   - قفل سخت‌افزاری بر اساس فرمت استاندارد **`fp_v1:<64-hex>`** با پالایش خودکار نویسه‌های نامرئی، نیم‌فاصله‌ها و کاراکترهای جهت متن (RLM/LRM) و پشتیبانی از بارگذاری فایل درخواست `.daqreq`.
4. **گاوصندوق رمزنگاری‌شده کلیدهای امضا (Encrypted Key Vault):**
   - ذخیره‌سازی کلیدهای اختصاصی منحصراً به فرمت رمزنگاری‌شده `.dqvault` با الگوریتم **AES-256-GCM** و مشتق‌سازی کلید با **PBKDF2-HMAC-SHA256 (۶۰۰٬۰۰۰ دور تکرار)**.
   - پاک‌سازی امن حافظه با کتابخانه `zeroize`.
   - خط‌مشی سخت‌گیرانه عدم ثبت هیچ کلید خصوصی در مخزن گیت.
5. **رابط کاربری مدرن فارسی با اتصال صحیح حروف و کنتراست بالا (WCAG AA+):**
   - قلم‌های محلی **وزیرمتن (Vazirmatn)** و **جت‌برینز مونو (JetBrains Mono)** به صورت باینری با دستور `include_bytes!` درون فایل اجرایی تعبیه شده‌اند.
   - رندر و شکل‌دهی صحیح حروف متصل زبان فارسی به صورت بومی با موتور `cosmic-text` / `rustybuzz`.
   - پشتیبانی از تم‌های تیره (Dark) و روشن (Light) منطبق بر استانداردهای کنتراست **WCAG AA+**.
6. **دفتر ثبت فروش و بایگانی لایسنس‌ها (Bundled SQLite Ledger):**
   - پایگاه داده سبک و محلی SQLite برای جستجو، فیلتر و نگهداری سوابق لایسنس‌های صادر شده.
   - دکمه تمدید و صدور جدید سریع (`Clone & Renew`).
   - خروجی استاندارد CSV جهت وارد کردن به نرم‌افزارهای حسابداری و اکسل.
7. **بازرس و شبیه‌ساز دقیق (Inspector & Simulator):**
   - بارگذاری هر فایل `.daqlic` با دیالوگ استاندارد Win32، راستی‌آزمایی امضا و محاسبه هش SHA-256.
   - شبیه‌ساز کامل رفتار ارزیابی نرم‌افزار دقیق برای بررسی تاریخ‌های انقضا، نسخه‌های مجاز و شناسه‌های سخت‌افزاری.
8. **اجرای خودکار آزمون‌های هِدلس و گزارش سلامت:**
   - پرچم `--self-test --report <path>` جهت تست داخلی تمام ماژول‌های رمزنگاری و شبیه‌ساز وضعیت بدون باز شدن پنجره گرافیکی.
   - پرچم `--version` جهت استعلام نسخه سریع.

---

## 🛠️ معماری سیستم (System Architecture)

```
Daqiq-License-Generator/
├── Cargo.toml                       # پیکربندی ورک‌اسپیس Rust و بهینه‌سازی سایز LTO
├── crates/
│   └── daqiq-lic-core/              # هسته محاسباتی، رمزنگاری و منطق کسب‌وکار (Pure Rust)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── crypto.rs            # سریال‌سازی NFC، هش و امضای Ed25519
│           ├── schema.rs            # تعاریف داده‌ای ۱۴ فیلد استاندارد لایسنس
│           ├── hwid.rs              # پالایش و اعتبارسنجی شناسه‌های سخت‌افزاری fp_v1
│           ├── jalali.rs            # تبدیل دقیق تاریخ شمسی و میلادی UTC
│           ├── vault.rs             # گاوصندوق رمزنگاری‌شده کلید با AES-256-GCM
│           ├── registry.rs          # پایگاه داده داخلی SQLite و خروجی CSV
│           └── error.rs             # مدیریت جامع خطاهای سیستم
├── apps/
│   └── daqiq-lic-gui/               # رابط کاربری بومی دسکتاپ (Pure Native Rust - بدون WebView)
│       ├── Cargo.toml               # استفاده از iced با tiny-skia و بدون wgpu/webview
│       ├── build.rs                 # الحاق آیکون و فراداده ویندوز (VERSIONINFO 1.2.0)
│       ├── assets/
│       │   ├── icon.ico             # آیکون وکتور اپلیکیشن
│       │   └── fonts/               # قلم‌های آفلاین TTF (Vazirmatn + JetBrains Mono)
│       └── src/
│           ├── main.rs              # نقطه ورود GUI، ساب‌سیستم ویندوز و پرچم --self-test
│           ├── theme.rs             # تم‌های تیره و روشن با کنتراست بالا (WCAG AA+)
│           ├── rtl.rs               # ابزارهای تقویم شمسی و قالب‌بندی فارسی
│           ├── state.rs             # مدیریت وضعیت بر پایه معماری Elm (AppState & Messages)
│           └── views/
│               ├── mod.rs
│               ├── header.rs        # نوار بالای وضعیت، وضعیت گاوصندوق و دکمه تم
│               ├── generator.rs     # زبانه ۱: استودیو تولید لایسنس
│               ├── registry.rs      # زبانه ۲: دفتر فروش و سوابق
│               ├── inspector.rs     # زبانه ۳: بازرس و شبیه‌ساز دقیق
│               └── vault.rs         # زبانه ۴: گاوصندوق کلیدها و تنظیمات
├── scripts/
│   └── verify_pe.py                 # اسکریپت بازرسی هدر PE، ساب‌سیستم و وابستگی‌های DLL
├── tests/
│   └── test_cross_repo_compatibility.py # آزمون انطباق متقابل با وریفایر Daqiq PR #38
├── docs/
│   ├── COMPATIBILITY_CONTRACT.md    # سند رسمی سازگاری بایت‌به‌بایت با دقیق
│   └── ROOT_CAUSE_V1_0_0.md         # گزارش تحلیل ریشه‌ای اشکالات نسخه ۱.۰.۰
├── assets/                          # آیکون‌های برنامه (.ico, .png, .svg)
└── VERSION                          # شناسه نسخه (1.2.0)
```

---

## ⚙️ دستورات ساخت و تست (Build & Verification)

### اجرای آزمون‌های ورک‌اسپیس Rust
```bash
cargo test --workspace --verbose
```

### اجرای تست سازگاری متقابل با Daqiq PR #38
```bash
python -m pytest tests/test_cross_repo_compatibility.py -v
```

### ساخت نسخه نهایی پرتابل (Standalone Release EXE)
```bash
cargo build --release -p daqiq-license-generator
```
فایل اجرایی در مسیر `target/release/daqiq-license-generator.exe` با حجم حدود **۵.۵ مگابایت** تولید می‌شود.

### بازرسی خودکار هدر PE و عدم وابستگی به WebView2
```bash
python scripts/verify_pe.py target/release/daqiq-license-generator.exe 25.0
```

### اجرای آزمون سلامت هِدلس (Headless Self-Test)
```bash
.\target\release\daqiq-license-generator.exe --self-test --report selftest_report.json
```

---

## 🔒 امنیت و گواهی انطباق (Security Compliance)

- **Air-Gapped & 100% Offline:** صفر اتصال شبکه‌ای در زمان اجرا؛ تمام قلم‌ها و دارایی‌ها مستقیماً درون باینری تعبیه شده‌اند.
- **Zero Secrets in Git:** هیچ کلید خصوصی متن آشکاری در مخزن وجود ندارد؛ آزمون‌ها با کلیدهای گذرا اجرا می‌شوند.
- **PE GUI Subsystem 2:** بدون وابستگی به کنسول ویندوز یا خط فرمان.
