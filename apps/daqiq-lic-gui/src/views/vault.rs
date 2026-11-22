use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use crate::state::{AppState, Message, ThemeMode};
use crate::theme::{card_style, code_box_style, status_pill_style, FONT_BOLD, FONT_MONO, FONT_REGULAR};

pub fn view_vault(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // ================= LEFT COLUMN: Vault Info & Unlock =================
    let info_box = container(
        column![
            row![
                text("شناسه کلید فعال:").font(FONT_REGULAR).size(12),
                text(&state.active_key_id).font(FONT_BOLD).size(13),
            ]
            .spacing(8),
            row![
                text("کلید عمومی فعال:").font(FONT_REGULAR).size(12),
                text(&state.active_public_key)
                    .font(FONT_MONO)
                    .size(11),
            ]
            .spacing(8),
            button(
                row![
                    text("📋").size(14),
                    text("کپی کلید عمومی برای نرم‌افزار دقیق").font(FONT_REGULAR).size(12),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::CopyPublicKeyForDaqiq)
            .padding([6, 12]),
        ]
        .spacing(10),
    )
    .padding(14)
    .style(move |_| code_box_style(is_dark));

    let unlock_form = column![
        text("بازگشایی قفل مخزن موجود (.dqvault)").font(FONT_BOLD).size(14),
        text("فایل مخزن کلیدهای رمزنگاری شده خود را انتخاب کرده و رمز عبور را وارد نمایید:")
            .font(FONT_REGULAR)
            .size(12),
        column![
            text("رمز عبور مخزن").font(FONT_REGULAR).size(12),
            text_input("رمز عبور ورود به مخزن کلید", &state.unlock_passphrase)
                .on_input(Message::UnlockPassphraseChanged)
                .secure(true)
                .font(FONT_MONO)
                .padding(8),
        ]
        .spacing(4),
        row![
            button(
                row![
                    text("🔓").size(16),
                    text("انتخاب فایل و بازگشایی قفل").font(FONT_BOLD).size(13),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .on_press(Message::UnlockVaultClicked)
            .padding([8, 16]),
            button(
                row![
                    text("🔒").size(16),
                    text("قفل کردن مخزن").font(FONT_REGULAR).size(13),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .on_press(Message::LockVaultClicked)
            .padding([8, 16]),
        ]
        .spacing(10),
    ]
    .spacing(12);

    let mut left_content = column![
        text("مخزن کلیدهای امضا (Ed25519 Secure Vault)").font(FONT_BOLD).size(15),
        info_box,
        unlock_form,
    ]
    .spacing(16);

    if let Some((is_ok, msg)) = &state.unlock_status_msg {
        let msg_box = container(
            row![
                text(if *is_ok { "✅" } else { "⚠️" }).size(16),
                text(msg).font(FONT_REGULAR).size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(12)
        .style(move |_| status_pill_style(*is_ok, is_dark));

        left_content = left_content.push(msg_box);
    }

    let left_card = container(left_content)
        .padding(16)
        .height(Length::Fill)
        .style(move |th| card_style(th, is_dark));

    // ================= RIGHT COLUMN: Create Vault & About =================
    let create_form = column![
        text("ایجاد جفت کلید جدید و صدور مخزن امن").font(FONT_BOLD).size(15),
        column![
            text("شناسه کلید جدید (Key ID)").font(FONT_REGULAR).size(12),
            text_input("daqiq-custom-v1", &state.new_key_id)
                .on_input(Message::NewKeyIdChanged)
                .font(FONT_MONO)
                .padding(8),
        ]
        .spacing(4),
        column![
            text("رمز عبور امن جهت رمزنگاری مخزن (AES-256-GCM)").font(FONT_REGULAR).size(12),
            text_input("حداقل ۸ کاراکتر ترکیبی", &state.new_passphrase)
                .on_input(Message::NewPassphraseChanged)
                .secure(true)
                .font(FONT_MONO)
                .padding(8),
        ]
        .spacing(4),
        button(
            row![
                text("🔑").size(16),
                text("ایجاد کلید جدید و ذخیره فایل .dqvault").font(FONT_BOLD).size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::CreateVaultClicked)
        .padding([8, 16]),
    ]
    .spacing(12);

    let notice_box = container(
        column![
            text("📌 راهنمای تطابق با نرم‌افزار دقیق:").font(FONT_BOLD).size(12),
            text("کلید رسمی پیش‌فرض شرکت در نسخه Daqiq v4.9.0 برابر daqiq-prod-v1 می‌باشد. جهت صدور لایسنس برای نسخه‌های رسمی بدون تغییر کد، باید از کلید daqiq-prod-v1 استفاده شود.")
                .font(FONT_REGULAR)
                .size(11),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(move |_| code_box_style(is_dark));

    let about_box = container(
        column![
            text("درباره استودیو صدور لایسنس دقیق").font(FONT_BOLD).size(13),
            text("نسخه ۱.۲.۰ — بازطراحی کامل با معماری ۱۰۰٪ Native Rust (بدون WebView2)")
                .font(FONT_REGULAR)
                .size(11),
            text("• الگوریتم رمزنگاری: Ed25519 Asymmetric Cryptography")
                .font(FONT_MONO)
                .size(10),
            text("• مخزن کلید: AES-256-GCM + PBKDF2 (600,000 تکرار) با Zeroize")
                .font(FONT_MONO)
                .size(10),
            text("• رندر گرافیکی: موتور نرم‌افزاری خودکفا Tiny-Skia (بدون نیاز به کارت گرافیک یا درایور)")
                .font(FONT_REGULAR)
                .size(10),
            text("• ۱۰۰٪ آفلاین و کاملاً ایمن (Air-Gapped) با فونت محلی وزیرمتن")
                .font(FONT_REGULAR)
                .size(10),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(move |th| card_style(th, is_dark));

    let mut right_content = column![
        create_form,
        notice_box,
        about_box,
    ]
    .spacing(16);

    if let Some((is_ok, msg)) = &state.create_status_msg {
        let msg_box = container(
            row![
                text(if *is_ok { "✅" } else { "⚠️" }).size(16),
                text(msg).font(FONT_REGULAR).size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(12)
        .style(move |_| status_pill_style(*is_ok, is_dark));

        right_content = right_content.push(msg_box);
    }

    let right_card = container(right_content)
        .padding(16)
        .height(Length::Fill)
        .style(move |th| card_style(th, is_dark));

    let content_row = row![
        scrollable(left_card).width(Length::FillPortion(1)).height(Length::Fill),
        scrollable(right_card).width(Length::FillPortion(1)).height(Length::Fill),
    ]
    .spacing(20)
    .height(Length::Fill)
    .padding(20);

    content_row.into()
}
