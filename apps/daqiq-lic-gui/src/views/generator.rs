use iced::widget::{
    button, checkbox, column, container, radio, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

use crate::state::{AppState, Message, Preset, ThemeMode};
use crate::theme::{
    card_style, code_box_style, status_pill_style, FONT_BOLD, FONT_MONO, FONT_REGULAR,
};

pub fn view_generator(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // ================= CARD 1: Customer Info =================
    let card1_content = column![
        text("مشخصات خریدار و لایسنس").font(FONT_BOLD).size(15),
        row![
            column![
                text("نام مشتری / فروشگاه *").font(FONT_REGULAR).size(12),
                text_input("مثال: فروشگاه برادران حسینی", &state.customer_name)
                    .on_input(Message::CustomerNameChanged)
                    .font(FONT_REGULAR)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(2)),
            column![
                text("شناسه لایسنس *").font(FONT_REGULAR).size(12),
                row![
                    text_input("LIC-YYYYMMDD-XXXX", &state.license_id)
                        .on_input(Message::LicenseIdChanged)
                        .font(FONT_MONO)
                        .padding(8),
                    button(text("🔄").size(14))
                        .on_press(Message::RegenerateLicenseId)
                        .padding([8, 12]),
                ]
                .spacing(4),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
        row![
            column![
                text("سازمان / شعبه (اختیاری)").font(FONT_REGULAR).size(12),
                text_input("مثال: شعبه مرکزی تهران", &state.customer_org)
                    .on_input(Message::CustomerOrgChanged)
                    .font(FONT_REGULAR)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("اطلاعات تماس / تلفن (اختیاری)").font(FONT_REGULAR).size(12),
                text_input("+989123456789", &state.customer_contact)
                    .on_input(Message::CustomerContactChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
    ]
    .spacing(12);

    let card1 = container(card1_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    // ================= CARD 2: Subscription Model & Timing =================
    let is_perpetual = state.license_type == "perpetual";

    let presets_row = row![
        text("پریست‌های سریع:").font(FONT_REGULAR).size(12),
        button(text("آزمایشی (۱ ماه)").font(FONT_REGULAR).size(11))
            .on_press(Message::ApplyPreset(Preset::Trial))
            .padding([4, 8]),
        button(text("استاندارد (۱ سال)").font(FONT_REGULAR).size(11))
            .on_press(Message::ApplyPreset(Preset::Standard))
            .padding([4, 8]),
        button(text("حرفه‌ای (۳ کاربر)").font(FONT_REGULAR).size(11))
            .on_press(Message::ApplyPreset(Preset::Pro))
            .padding([4, 8]),
        button(text("سازمانی (۱۰ کاربر)").font(FONT_REGULAR).size(11))
            .on_press(Message::ApplyPreset(Preset::Enterprise))
            .padding([4, 8]),
        button(text("دائمی (مادام‌العمر)").font(FONT_REGULAR).size(11))
            .on_press(Message::ApplyPreset(Preset::Perpetual))
            .padding([4, 8]),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let card2_content = column![
        text("مدل اشتراک و تاریخ‌های اعتبار").font(FONT_BOLD).size(15),
        row![
            column![
                text("نوع لایسنس").font(FONT_REGULAR).size(12),
                row![
                    radio(
                        "اشتراکی (دارای انقضا)",
                        "subscription",
                        Some(state.license_type.as_str()),
                        |val| Message::LicenseTypeSelected(val.to_string()),
                    )
                    .font(FONT_REGULAR)
                    .size(13),
                    radio(
                        "مادام‌العمر (دائمی)",
                        "perpetual",
                        Some(state.license_type.as_str()),
                        |val| Message::LicenseTypeSelected(val.to_string()),
                    )
                    .font(FONT_REGULAR)
                    .size(13),
                ]
                .spacing(16),
            ]
            .spacing(4)
            .width(Length::FillPortion(2)),
            column![
                text("حداکثر دستگاه‌های مجاز").font(FONT_REGULAR).size(12),
                text_input("1", &state.max_devices.to_string())
                    .on_input(Message::MaxDevicesChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
        presets_row,
        row![
            column![
                text("تاریخ صدور (شمسی)").font(FONT_REGULAR).size(12),
                text_input("1405/09/01", &state.issued_jalali)
                    .on_input(Message::IssuedJalaliChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("شروع اعتبار (شمسی)").font(FONT_REGULAR).size(12),
                text_input("1405/09/01", &state.not_before_jalali)
                    .on_input(Message::NotBeforeJalaliChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text(if is_perpetual { "تاریخ انقضا (دائمی)" } else { "تاریخ انقضا (شمسی)" })
                    .font(FONT_REGULAR)
                    .size(12),
                text_input(
                    if is_perpetual { "بدون انقضا (دائمی)" } else { "1406/09/01" },
                    if is_perpetual { "" } else { &state.expires_jalali },
                )
                .on_input(Message::ExpiresJalaliChanged)
                .font(FONT_MONO)
                .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
        if !is_perpetual {
            row![
                text("افزودن سریع اعتبار:").font(FONT_REGULAR).size(12),
                button(text("+۱ ماه").font(FONT_REGULAR).size(11))
                    .on_press(Message::QuickDurationMonths(1))
                    .padding([4, 8]),
                button(text("+۳ ماه").font(FONT_REGULAR).size(11))
                    .on_press(Message::QuickDurationMonths(3))
                    .padding([4, 8]),
                button(text("+۶ ماه").font(FONT_REGULAR).size(11))
                    .on_press(Message::QuickDurationMonths(6))
                    .padding([4, 8]),
                button(text("+۱ سال").font(FONT_REGULAR).size(11))
                    .on_press(Message::QuickDurationMonths(12))
                    .padding([4, 8]),
                button(text("+۲ سال").font(FONT_REGULAR).size(11))
                    .on_press(Message::QuickDurationMonths(24))
                    .padding([4, 8]),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        } else {
            row![]
        },
    ]
    .spacing(12);

    let card2 = container(card2_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    // ================= CARD 3: Hardware IDs =================
    let hwid_badge = text(format!("{} دستگاه معتبر", state.hwid_valid_count))
        .font(FONT_REGULAR)
        .size(12);

    let hwid_header = row![
        text("شناسه‌های سخت‌افزاری دستگاه‌ها (HWID)").font(FONT_BOLD).size(15),
        Space::new().width(Length::Fill),
        container(hwid_badge)
            .padding([3, 8])
            .style(move |_| status_pill_style(state.hwid_valid_count > 0, is_dark)),
        button(text("📂 بارگذاری از فایل .daqreq").font(FONT_REGULAR).size(11))
            .on_press(Message::ImportDaqreqFile)
            .padding([4, 10]),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let card3_content = column![
        hwid_header,
        text("شناسه‌ها را از نرم‌افزار دقیق دریافت کرده و در کادر زیر وارد کنید (هر شناسه در یک سطر):")
            .font(FONT_REGULAR)
            .size(12),
        text_input(
            "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e",
            &state.hwid_text,
        )
        .on_input(Message::HwidTextChanged)
        .font(FONT_MONO)
        .padding(8),
        if let Some(err) = &state.hwid_error_msg {
            text(err).font(FONT_REGULAR).size(12)
        } else {
            text("")
        },
    ]
    .spacing(8);

    let card3 = container(card3_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    // ================= CARD 4: Feature Flags (10 Modules) =================
    let feature_header = row![
        text("ماژول‌ها و دسترسی‌های مجاز (۱۰ زبانه دقیق)").font(FONT_BOLD).size(15),
        Space::new().width(Length::Fill),
        button(text("انتخاب همه").font(FONT_REGULAR).size(11))
            .on_press(Message::SelectAllFeatures)
            .padding([4, 8]),
        button(text("لغو همه").font(FONT_REGULAR).size(11))
            .on_press(Message::DeselectAllFeatures)
            .padding([4, 8]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let make_feat_cb = |key: &'static str, label: &'static str| {
        let is_checked = *state.features.get(key).unwrap_or(&true);
        row![
            checkbox(is_checked)
                .on_toggle(move |val| Message::FeatureToggled(key.to_string(), val)),
            text(label).font(FONT_REGULAR).size(13),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    };

    let feat_col1 = column![
        make_feat_cb("dashboard", "📊 داشبورد و شاخص‌ها"),
        make_feat_cb("products", "📦 مدیریت کالاها و انبار"),
        make_feat_cb("batches", "🚚 ثبت خرید و محموله‌ها"),
        make_feat_cb("sales", "🛒 ثبت فروش و فاکتور"),
        make_feat_cb("services", "🛠️ میز خدمت و خدمات پس از فروش"),
    ]
    .spacing(8)
    .width(Length::FillPortion(1));

    let feat_col2 = column![
        make_feat_cb("upgrades", "🔧 ارتقای کالا و مونتاژ"),
        make_feat_cb("suppliers_payments", "💳 تأمین‌کنندگان و تسویه‌حساب"),
        make_feat_cb("reports", "📈 گزارش‌های جامع مالی"),
        make_feat_cb("employees", "👤 مدیریت کاربران و صندوقداران"),
        make_feat_cb("settings", "⚙️ تنظیمات و پشتیبان‌گیری"),
    ]
    .spacing(8)
    .width(Length::FillPortion(1));

    let card4_content = column![
        feature_header,
        row![feat_col1, feat_col2].spacing(16),
    ]
    .spacing(12);

    let card4 = container(card4_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    // ================= CARD 5: Version Bounds =================
    let card5_content = column![
        text("محدودیت نسخه برنامه (اختیاری)").font(FONT_BOLD).size(15),
        row![
            column![
                text("حداقل نسخه نرم‌افزار").font(FONT_REGULAR).size(12),
                text_input("4.9.0", &state.min_app_version)
                    .on_input(Message::MinVersionChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("حداکثر نسخه مجاز (اختیاری)").font(FONT_REGULAR).size(12),
                text_input("خالی برای نامحدود", &state.max_app_version)
                    .on_input(Message::MaxVersionChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
    ]
    .spacing(10);

    let card5 = container(card5_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    // Generate Button & Error
    let btn_generate = button(
        row![
            text("🛡️").size(16),
            text("تولید و امضای لایسنس دیجیتال (.daqlic)")
                .font(FONT_BOLD)
                .size(14),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .on_press(Message::GenerateLicenseClicked)
    .padding([12, 24]);

    let mut left_form = column![
        card1,
        card2,
        card3,
        card4,
        card5,
        btn_generate,
    ]
    .spacing(16)
    .width(Length::FillPortion(3));

    if let Some(err) = &state.generation_error {
        let err_box = container(
            row![
                text("⚠️").size(16),
                text(err).font(FONT_REGULAR).size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(12)
        .style(move |_| status_pill_style(false, is_dark));

        left_form = left_form.push(err_box);
    }

    // ================= RIGHT COLUMN: Output Preview =================
    let has_output = !state.generated_json.is_empty();

    let digest_box = container(
        column![
            text("هش SHA-256 محتوای امضا شده:").font(FONT_REGULAR).size(12),
            text(if has_output { &state.generated_sha256 } else { "—" })
                .font(FONT_MONO)
                .size(11),
        ]
        .spacing(4),
    )
    .padding(10)
    .style(move |_| code_box_style(is_dark));

    let json_preview = container(
        scrollable(
            text(if has_output {
                &state.generated_json
            } else {
                "محتوای فایل .daqlic پس از صدور در این بخش نمایش داده می‌شود..."
            })
            .font(FONT_MONO)
            .size(11),
        )
        .height(Length::Fill),
    )
    .padding(12)
    .height(Length::Fill)
    .style(move |_| code_box_style(is_dark));

    let mut copy_btn = button(text("📋 کپی لایسنس").font(FONT_REGULAR).size(13))
        .padding([8, 16])
        .width(Length::FillPortion(1));
    if has_output {
        copy_btn = copy_btn.on_press(Message::CopyGeneratedLicense);
    }

    let mut save_btn = button(text("💾 ذخیره فایل .daqlic").font(FONT_BOLD).size(13))
        .padding([8, 16])
        .width(Length::FillPortion(1));
    if has_output {
        save_btn = save_btn.on_press(Message::SaveGeneratedLicense);
    }

    let output_actions = row![copy_btn, save_btn].spacing(10);

    let right_card = container(
        column![
            row![
                text("خروجی لایسنس امضا شده").font(FONT_BOLD).size(15),
                Space::new().width(Length::Fill),
                container(
                    text(if has_output { "آماده تحویل" } else { "در انتظار تولید" })
                        .font(FONT_REGULAR)
                        .size(11),
                )
                .padding([2, 8])
                .style(move |_| status_pill_style(has_output, is_dark)),
            ]
            .align_y(Alignment::Center),
            digest_box,
            json_preview,
            output_actions,
        ]
        .spacing(12)
        .height(Length::Fill),
    )
    .padding(16)
    .height(Length::Fill)
    .style(move |th| card_style(th, is_dark));

    let right_column = column![right_card]
        .width(Length::FillPortion(2))
        .height(Length::Fill);

    let content_row = row![
        scrollable(left_form).height(Length::Fill),
        right_column,
    ]
    .spacing(20)
    .height(Length::Fill)
    .padding(20);

    content_row.into()
}
