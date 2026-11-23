use iced::widget::{
    button, checkbox, column, container, radio, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

use crate::state::{AppState, Message, Preset, ThemeMode};
use crate::theme::{
    card_style, code_box_style, status_pill_style, subcard_style, warning_pill_style,
    FONT_BOLD, FONT_MONO, FONT_REGULAR,
};

pub fn view_generator(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // ================= RIGHT COLUMN =================
    // Card 1: Customer Details & Presets
    let card_customer = container(
        column![
            row![
                text("👤 مشخصات مشتری و شناسه لایسنس").font(FONT_BOLD).size(14),
                Space::new().width(Length::Fill),
            ]
            .align_y(Alignment::Center),

            // Presets row
            row![
                text("پریست‌های سریع:").font(FONT_REGULAR).size(11),
                button(text("آزمایشی").font(FONT_REGULAR).size(11))
                    .on_press(Message::ApplyPreset(Preset::Trial))
                    .padding([3, 7]),
                button(text("استاندارد").font(FONT_REGULAR).size(11))
                    .on_press(Message::ApplyPreset(Preset::Standard))
                    .padding([3, 7]),
                button(text("حرفه‌ای (۳ کاربر)").font(FONT_REGULAR).size(11))
                    .on_press(Message::ApplyPreset(Preset::Pro))
                    .padding([3, 7]),
                button(text("سازمانی (۱۰ کاربر)").font(FONT_REGULAR).size(11))
                    .on_press(Message::ApplyPreset(Preset::Enterprise))
                    .padding([3, 7]),
                button(text("دائمی").font(FONT_REGULAR).size(11))
                    .on_press(Message::ApplyPreset(Preset::Perpetual))
                    .padding([3, 7]),
            ]
            .spacing(5)
            .align_y(Alignment::Center),

            row![
                column![
                    text("نام مشتری یا فروشگاه *").font(FONT_REGULAR).size(12),
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
                            .padding([8, 10]),
                    ]
                    .spacing(4),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(10),

            row![
                column![
                    text("سازمان / مجموعه (اختیاری)").font(FONT_REGULAR).size(12),
                    text_input("مثال: شرکت داده‌پردازی سپهر", &state.customer_org)
                        .on_input(Message::CustomerOrgChanged)
                        .font(FONT_REGULAR)
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),

                column![
                    text("شماره تماس یا ایمیل (اختیاری)").font(FONT_REGULAR).size(12),
                    text_input("+989123456789", &state.customer_contact)
                        .on_input(Message::CustomerContactChanged)
                        .font(FONT_MONO)
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(10),
        ]
        .spacing(10),
    )
    .padding(14)
    .style(move |th| card_style(th, is_dark));

    // Card 2: Hardware ID Management
    let card_hwid = container(
        column![
            row![
                text("🖥️ قفل سخت‌افزاری و شناسه‌های دستگاه (HWID)").font(FONT_BOLD).size(14),
                Space::new().width(Length::Fill),
                // Live HWID badge
                container(
                    text(&state.cached_hwid_badge).font(FONT_REGULAR).size(11),
                )
                .padding([2, 8])
                .style(move |_th| {
                    if state.cached_hwid_has_error {
                        warning_pill_style(is_dark)
                    } else if state.cached_hwid_valid_count > 0 {
                        status_pill_style(true, is_dark)
                    } else {
                        status_pill_style(false, is_dark)
                    }
                }),
            ]
            .align_y(Alignment::Center),

            text("شناسه سخت‌افزاری تولید شده توسط Daqiq v4.9.1 (با پیشوند fp_v1:) را وارد کنید. چند دستگاه را با خط جدید جدا کنید:")
                .font(FONT_REGULAR)
                .size(11),

            text_input(
                "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e",
                &state.hwid_text,
            )
            .on_input(Message::HwidTextChanged)
            .font(FONT_MONO)
            .padding(8),

            row![
                button(
                    row![
                        text("📋").size(13),
                        text("الصاق از کلیپ‌بورد").font(FONT_REGULAR).size(11),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::PasteHwidFromClipboard)
                .padding([6, 12]),

                button(
                    row![
                        text("📥").size(13),
                        text("وارد کردن فایل درخواست (.daqreq)").font(FONT_REGULAR).size(11),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::ImportDaqreqFile)
                .padding([6, 12]),

                Space::new().width(Length::Fill),

                text("سقف دستگاه:").font(FONT_REGULAR).size(11),
                text_input("1", &state.max_devices.to_string())
                    .on_input(Message::MaxDevicesChanged)
                    .font(FONT_MONO)
                    .width(Length::Fixed(45.0))
                    .padding(4),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding(14)
    .style(move |th| card_style(th, is_dark));

    let right_column = column![
        card_customer,
        card_hwid,
    ]
    .spacing(12)
    .width(Length::FillPortion(1));

    // ================= LEFT COLUMN =================
    // Card 3: Subscription Model & Jalali Timing
    let is_perpetual = state.license_type == "perpetual";

    let license_type_radios = row![
        text("نوع لایسنس:").font(FONT_REGULAR).size(12),
        radio(
            "اشتراکی (مدت‌دار)",
            "subscription",
            Some(state.license_type.as_str()),
            |val| Message::LicenseTypeSelected(val.to_string()),
        )
        .font(FONT_REGULAR)
        .size(14),
        radio(
            "دائمی (مادام‌العمر)",
            "perpetual",
            Some(state.license_type.as_str()),
            |val| Message::LicenseTypeSelected(val.to_string()),
        )
        .font(FONT_REGULAR)
        .size(14),
        radio(
            "آزمایشی (Trial)",
            "trial",
            Some(state.license_type.as_str()),
            |val| Message::LicenseTypeSelected(val.to_string()),
        )
        .font(FONT_REGULAR)
        .size(14),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let duration_chips = row![
        text("مدت:").font(FONT_REGULAR).size(11),
        button(text("+۳۰ روز").font(FONT_REGULAR).size(11))
            .on_press(Message::QuickDurationMonths(1))
            .padding([3, 6]),
        button(text("+۹۰ روز").font(FONT_REGULAR).size(11))
            .on_press(Message::QuickDurationMonths(3))
            .padding([3, 6]),
        button(text("+۱ سال").font(FONT_REGULAR).size(11))
            .on_press(Message::QuickDurationMonths(12))
            .padding([3, 6]),
        button(text("+۲ سال").font(FONT_REGULAR).size(11))
            .on_press(Message::QuickDurationMonths(24))
            .padding([3, 6]),
        button(text("مادام‌العمر").font(FONT_REGULAR).size(11))
            .on_press(Message::QuickDurationMonths(-1))
            .padding([3, 6]),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let jalali_dates_row = row![
        column![
            text("تاریخ صدور").font(FONT_REGULAR).size(11),
            text_input("1405/09/01", &state.issued_jalali)
                .on_input(Message::IssuedJalaliChanged)
                .font(FONT_MONO)
                .padding(6),
        ]
        .spacing(3)
        .width(Length::FillPortion(1)),

        column![
            text("شروع اعتبار").font(FONT_REGULAR).size(11),
            text_input("1405/09/01", &state.not_before_jalali)
                .on_input(Message::NotBeforeJalaliChanged)
                .font(FONT_MONO)
                .padding(6),
        ]
        .spacing(3)
        .width(Length::FillPortion(1)),

        column![
            text(if is_perpetual { "تاریخ انقضا (نامحدود)" } else { "تاریخ انقضا *" })
                .font(FONT_REGULAR)
                .size(11),
            if is_perpetual {
                text_input("دائمی (بدون انقضا)", "—")
                    .font(FONT_REGULAR)
                    .padding(6)
            } else {
                text_input("1406/09/01", &state.expires_jalali)
                    .on_input(Message::ExpiresJalaliChanged)
                    .font(FONT_MONO)
                    .padding(6)
            },
        ]
        .spacing(3)
        .width(Length::FillPortion(1)),
    ]
    .spacing(8);

    let card_timing = container(
        column![
            row![
                text("📅 مدل لایسنس و تقویم شمسی").font(FONT_BOLD).size(14),
                Space::new().width(Length::Fill),
                text(&state.cached_jalali_summary).font(FONT_REGULAR).size(11),
            ]
            .align_y(Alignment::Center),

            license_type_radios,
            row![
                duration_chips,
                Space::new().width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            jalali_dates_row,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(move |th| card_style(th, is_dark));

    // Card 4: PR #39 Controls & Features
    let match_policy_row = row![
        text("سیاست انطباق:").font(FONT_REGULAR).size(11),
        radio("استاندارد (پیش‌فرض)", "standard", Some(state.match_policy.as_str()), |v| Message::MatchPolicyChanged(v.to_string())).font(FONT_REGULAR).size(13),
        radio("سخت‌گیرانه", "strict", Some(state.match_policy.as_str()), |v| Message::MatchPolicyChanged(v.to_string())).font(FONT_REGULAR).size(13),
        radio("منعطف", "relaxed", Some(state.match_policy.as_str()), |v| Message::MatchPolicyChanged(v.to_string())).font(FONT_REGULAR).size(13),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let advanced_timing_row = row![
        column![
            text("مهلت (روز)").font(FONT_REGULAR).size(11),
            text_input("7", &state.grace_period_days.to_string())
                .on_input(Message::GracePeriodDaysChanged)
                .font(FONT_MONO)
                .padding(5),
        ]
        .spacing(2)
        .width(Length::FillPortion(1)),

        column![
            text("حالت مهلت").font(FONT_REGULAR).size(11),
            row![
                radio("فقط‌خواندنی", "readonly", Some(state.grace_mode.as_str()), |v| Message::GraceModeChanged(v.to_string())).font(FONT_REGULAR).size(12),
                radio("کامل", "full", Some(state.grace_mode.as_str()), |v| Message::GraceModeChanged(v.to_string())).font(FONT_REGULAR).size(12),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(2)
        .width(Length::FillPortion(2)),

        column![
            text("هشدار (روز)").font(FONT_REGULAR).size(11),
            text_input("14", &state.expiry_warning_days.to_string())
                .on_input(Message::ExpiryWarningDaysChanged)
                .font(FONT_MONO)
                .padding(5),
        ]
        .spacing(2)
        .width(Length::FillPortion(1)),

        column![
            text("انحراف ساعت (س)").font(FONT_REGULAR).size(11),
            text_input("24", &state.max_clock_skew_hours.to_string())
                .on_input(Message::MaxClockSkewHoursChanged)
                .font(FONT_MONO)
                .padding(5),
        ]
        .spacing(2)
        .width(Length::FillPortion(1)),
    ]
    .spacing(8);

    // 10 Features
    let feature_labels: [(&str, &str); 10] = [
        ("dashboard", "📊 داشبورد"),
        ("products", "📦 مدیریت کالاها"),
        ("batches", "🚚 خرید و محموله‌ها"),
        ("sales", "🛒 ثبت فروش"),
        ("services", "🛠️ میز خدمت"),
        ("upgrades", "🔧 ارتقای کالا"),
        ("suppliers_payments", "💳 تأمین‌کنندگان"),
        ("reports", "📈 گزارش‌های جامع"),
        ("employees", "👤 مدیریت کاربران"),
        ("settings", "⚙️ تنظیمات"),
    ];

    let mut feat_col1 = column![].spacing(6);
    let mut feat_col2 = column![].spacing(6);
    for (i, (key, label)) in feature_labels.iter().enumerate() {
        let is_checked = *state.features.get(*key).unwrap_or(&true);
        let k = key.to_string();
        let item = row![
            checkbox(is_checked).on_toggle(move |c| Message::FeatureToggled(k.clone(), c)),
            text(*label).font(FONT_REGULAR).size(12),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        if i < 5 {
            feat_col1 = feat_col1.push(item);
        } else {
            feat_col2 = feat_col2.push(item);
        }
    }

    let card_features = container(
        column![
            row![
                text("⚙️ دسترسی‌های PR #39 و محدوده نسخه").font(FONT_BOLD).size(14),
                Space::new().width(Length::Fill),
                button(text("انتخاب همه ✓").font(FONT_REGULAR).size(10))
                    .on_press(Message::SelectAllFeatures)
                    .padding([2, 6]),
                button(text("لغو همه ✕").font(FONT_REGULAR).size(10))
                    .on_press(Message::DeselectAllFeatures)
                    .padding([2, 6]),
            ]
            .spacing(6)
            .align_y(Alignment::Center),

            match_policy_row,
            advanced_timing_row,

            row![
                feat_col1.width(Length::FillPortion(1)),
                feat_col2.width(Length::FillPortion(1)),
            ]
            .spacing(12),

            row![
                column![
                    text("حداقل نسخه مجاز برنامه").font(FONT_REGULAR).size(11),
                    text_input("4.9.0", &state.min_app_version)
                        .on_input(Message::MinVersionChanged)
                        .font(FONT_MONO)
                        .padding(5),
                ]
                .spacing(2)
                .width(Length::FillPortion(1)),

                column![
                    text("حداکثر نسخه مجاز (اختیاری)").font(FONT_REGULAR).size(11),
                    text_input("نامحدود", &state.max_app_version)
                        .on_input(Message::MaxVersionChanged)
                        .font(FONT_MONO)
                        .padding(5),
                ]
                .spacing(2)
                .width(Length::FillPortion(1)),
            ]
            .spacing(10),
        ]
        .spacing(10),
    )
    .padding(14)
    .style(move |th| card_style(th, is_dark));

    let left_column = column![
        card_timing,
        card_features,
    ]
    .spacing(12)
    .width(Length::FillPortion(1));

    // 2-Column Responsive Workspace
    let workspace = row![
        right_column,
        left_column,
    ]
    .spacing(14)
    .width(Length::Fill);

    let scrollable_content = scrollable(
        column![
            workspace,
            Space::new().height(Length::Fixed(10.0)),
        ]
        .spacing(10)
        .padding(16),
    )
    .height(Length::Fill);

    // Sticky Bottom Action Bar
    let validation_pill = container(
        row![
            text(if state.cached_validation_is_ok { "✓" } else { "⚠️" }).size(13),
            text(&state.cached_validation_status_text).font(FONT_REGULAR).size(11),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .style(move |_th| {
        if state.cached_validation_is_ok {
            status_pill_style(true, is_dark)
        } else {
            warning_pill_style(is_dark)
        }
    });

    let action_bar = container(
        row![
            validation_pill,

            Space::new().width(Length::Fill),

            button(
                text(if state.show_json_preview { "بستن پیش‌نمایش JSON ✕" } else { "پیش‌نمایش JSON 👁️" })
                    .font(FONT_REGULAR)
                    .size(12),
            )
            .on_press(Message::ToggleJsonPreview)
            .padding([8, 14]),

            button(
                row![
                    text("📋").size(14),
                    text("کپی توکن متنی (Base64)").font(FONT_REGULAR).size(12),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::CopyCompactToken)
            .padding([8, 14]),

            button(
                row![
                    text("💾").size(15),
                    text("تولید و ذخیره لایسنس (.daqlic)").font(FONT_BOLD).size(13),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .on_press(Message::GenerateLicenseClicked)
            .padding([9, 20]),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([10, 16])
    .style(move |th| subcard_style(th, is_dark));

    // Optional Collapsible JSON preview
    let mut main_view = column![scrollable_content];

    if state.show_json_preview && !state.generated_json.is_empty() {
        let preview_box = container(
            column![
                row![
                    text("خروجی کانویکال امضا شده (.daqlic):").font(FONT_BOLD).size(12),
                    Space::new().width(Length::Fill),
                    button(text("کپی JSON 📋").font(FONT_REGULAR).size(11))
                        .on_press(Message::CopyGeneratedLicense)
                        .padding([3, 8]),
                    button(text("ذخیره در فایل 💾").font(FONT_REGULAR).size(11))
                        .on_press(Message::SaveGeneratedLicense)
                        .padding([3, 8]),
                    text(format!("SHA-256 Digest: {}", state.generated_sha256)).font(FONT_MONO).size(11),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                container(
                    scrollable(text(&state.generated_json).font(FONT_MONO).size(11))
                        .height(Length::Fixed(180.0)),
                )
                .padding(10)
                .style(move |th| code_box_style(th, is_dark)),
            ]
            .spacing(6),
        )
        .padding(12)
        .style(move |th| card_style(th, is_dark));

        main_view = main_view.push(preview_box);
    }

    if let Some(err) = &state.generation_error {
        let err_box = container(
            row![
                text("❌").size(16),
                text(err).font(FONT_REGULAR).size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 14])
        .style(move |_th| status_pill_style(false, is_dark));

        main_view = main_view.push(err_box);
    }

    main_view.push(action_bar).into()
}
