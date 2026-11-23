use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::state::{AppState, Message, ThemeMode};
use crate::theme::{card_style, status_pill_style, FONT_BOLD, FONT_MONO, FONT_REGULAR};

pub fn view_inspector(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // ================= RIGHT COLUMN: Inspector =================
    let inspect_header = row![
        text("بررسی و اعتبارسنجی لایسنس یا توکن متنی").font(FONT_BOLD).size(15),
        Space::new().width(Length::Fill),
        button(text("📂 باز کردن فایل .daqlic").font(FONT_REGULAR).size(12))
            .on_press(Message::InspectFilePicked)
            .padding([6, 12]),
    ]
    .align_y(Alignment::Center);

    let mut left_content = column![
        inspect_header,
        text("محتوای JSON یا توکن متنی Base64 را در کادر زیر وارد کنید:")
            .font(FONT_REGULAR)
            .size(12),
        text_input(
            "محتوای فایل .daqlic یا توکن متنی Base64 (eyJ...)",
            &state.inspect_text,
        )
        .on_input(Message::InspectTextChanged)
        .font(FONT_MONO)
        .padding(8),
        column![
            text("کلید عمومی جهت بررسی (اختیاری - پیش‌فرض: کلید فعال مخزن)").font(FONT_REGULAR).size(12),
            text_input("خالی برای کلید فعال مخزن", &state.custom_public_key)
                .on_input(Message::CustomPubkeyChanged)
                .font(FONT_MONO)
                .padding(8),
        ]
        .spacing(4),
        button(
            row![
                text("🔍").size(16),
                text("اعتبارسنجی امضای دیجیتال").font(FONT_BOLD).size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::VerifyInspectClicked)
        .padding([8, 16]),
    ]
    .spacing(12);

    if let Some((is_valid, title, detail)) = &state.inspect_status {
        let status_box = container(
            column![
                row![
                    text(if *is_valid { "✅" } else { "❌" }).size(18),
                    text(title).font(FONT_BOLD).size(14),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(detail).font(FONT_REGULAR).size(12),
                if !state.inspect_sha256.is_empty() {
                    text(format!("هش SHA-256: {}", state.inspect_sha256))
                        .font(FONT_MONO)
                        .size(11)
                } else {
                    text("")
                },
            ]
            .spacing(6),
        )
        .padding(14)
        .style(move |_th| status_pill_style(*is_valid, is_dark));

        left_content = left_content.push(status_box);
    }

    let left_card = container(left_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    // ================= LEFT COLUMN: Simulator =================
    let mut right_content = column![
        text("شبیه‌ساز رفتار نرم‌افزار دقیق v4.9.1").font(FONT_BOLD).size(15),
        text("ارزیابی کنید که آیا نرم‌افزار دقیق در تاریخ، نسخه و سیستم مشخص، این لایسنس را می‌پذیرد یا خیر:")
            .font(FONT_REGULAR)
            .size(12),
        column![
            text("شناسه سخت‌افزاری دستگاه مشتری (HWID)").font(FONT_REGULAR).size(12),
            text_input(
                "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e",
                &state.sim_hwid,
            )
            .on_input(Message::SimHwidChanged)
            .font(FONT_MONO)
            .padding(8),
        ]
        .spacing(4),
        row![
            column![
                text("تاریخ فرضی ارزیابی (شمسی)").font(FONT_REGULAR).size(12),
                text_input("1405/09/01", &state.sim_date_jalali)
                    .on_input(Message::SimDateChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("نسخه فرضی برنامه").font(FONT_REGULAR).size(12),
                text_input("4.9.1", &state.sim_version)
                    .on_input(Message::SimVersionChanged)
                    .font(FONT_MONO)
                    .padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
        button(
            row![
                text("⚡").size(16),
                text("اجرای شبیه‌سازی ورود مشتری").font(FONT_BOLD).size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::RunSimulationClicked)
        .padding([8, 16]),
    ]
    .spacing(12);

    if let Some((is_ok, title, detail)) = &state.sim_status {
        let sim_box = container(
            column![
                row![
                    text(if *is_ok { "✅" } else { "⚠️" }).size(18),
                    text(title).font(FONT_BOLD).size(14),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(detail).font(FONT_REGULAR).size(12),
            ]
            .spacing(6),
        )
        .padding(14)
        .style(move |_th| status_pill_style(*is_ok, is_dark));

        right_content = right_content.push(sim_box);
    }

    let right_card = container(right_content)
        .padding(16)
        .style(move |th| card_style(th, is_dark));

    let content_row = row![
        left_card.width(Length::FillPortion(1)),
        right_card.width(Length::FillPortion(1)),
    ]
    .spacing(16)
    .width(Length::Fill);

    container(scrollable(content_row)).padding(16).into()
}
