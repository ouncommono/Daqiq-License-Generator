use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use daqiq_lic_core::PublicKeyMatchResult;

use crate::state::{AppState, Message, ThemeMode};
use crate::theme::{
    card_style, code_box_style, status_pill_style, subcard_style, warning_pill_style,
    FONT_BOLD, FONT_MONO, FONT_REGULAR,
};

pub fn view_vault(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // 1. Active Key Info Card
    let is_open = state.vault.is_some();
    let key_info_card = container(
        column![
            row![
                text("🔐 کلید رمزنگاری فعال").font(FONT_BOLD).size(15),
                Space::new().width(Length::Fill),
                container(
                    text(if is_open { "وضعیت: باز و فعال" } else { "وضعیت: قفل" })
                        .font(FONT_REGULAR)
                        .size(11),
                )
                .padding([3, 8])
                .style(move |_th| status_pill_style(is_open, is_dark)),
            ]
            .align_y(Alignment::Center),

            row![
                text("شناسه کلید:").font(FONT_REGULAR).size(12),
                text(&state.active_key_id).font(FONT_MONO).size(12),
                Space::new().width(Length::Fixed(20.0)),
                text("اثر انگشت (Fingerprint):").font(FONT_REGULAR).size(12),
                text(&state.active_key_fingerprint).font(FONT_MONO).size(12),
            ]
            .align_y(Alignment::Center),

            text("کلید عمومی فعال (Ed25519 Base64):").font(FONT_REGULAR).size(12),
            container(text(&state.active_public_key).font(FONT_MONO).size(12))
                .padding(8)
                .style(move |th| code_box_style(th, is_dark)),

            row![
                button(
                    row![
                        text("📋").size(14),
                        text("کپی کلید عمومی برای فایل constants.py در Daqiq").font(FONT_REGULAR).size(12),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::CopyPublicKeyForDaqiq)
                .padding([8, 14]),

                Space::new().width(Length::Fill),

                if is_open {
                    button(text("🔒 قفل کردن مخزن فعلی").font(FONT_REGULAR).size(12))
                        .on_press(Message::LockVaultClicked)
                        .padding([8, 14])
                } else {
                    button(text("قفل است").font(FONT_REGULAR).size(12)).padding([8, 14])
                },
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding(16)
    .style(move |th| card_style(th, is_dark));

    // 2. Public Key Pairing Verifier Card (NEW in v1.3.0)
    let pairing_card = container(
        column![
            row![
                text("🛡️ ابزار بررسی تطابق با کلید عمومی Daqiq v4.9.1").font(FONT_BOLD).size(15),
                Space::new().width(Length::Fill),
            ]
            .align_y(Alignment::Center),

            text("جهت اطمینان از اینکه فایل‌های لایسنس صادر شده با خطای «امضای دیجیتال لایسنس نامعتبر است» مواجه نمی‌شوند، کلید عمومی کامپایل شده در Daqiq را در زیر الصاق کنید تا وضعیت تطابق فورا بررسی شود:")
                .font(FONT_REGULAR)
                .size(12),

            row![
                text_input(
                    "کلید عمومی Daqiq (مثال: tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=)",
                    &state.pairing_input_pubkey,
                )
                .on_input(Message::PairingInputPubkeyChanged)
                .font(FONT_MONO)
                .padding(8),

                button(text("بررسی تطابق 🔍").font(FONT_REGULAR).size(12))
                    .on_press(Message::CheckPairingClicked)
                    .padding([8, 16]),
            ]
            .spacing(10),

            if let Some(status) = &state.pairing_status {
                match status {
                    PublicKeyMatchResult::Match => container(
                        row![
                            text("✓").size(16),
                            text("تطابق کامل (MATCH): کلید امضای فعال شما دقیقاً منطبق بر کلید عمومی Daqiq v4.9.1 است. لایسنس‌ها ۱۰۰٪ معتبر خواهند بود.").font(FONT_REGULAR).size(12),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 14])
                    .style(move |_th| status_pill_style(true, is_dark)),

                    PublicKeyMatchResult::Mismatch { expected_fp, actual_fp } => container(
                        row![
                            text("⚠️").size(16),
                            text(format!(
                                "عدم تطابق (MISMATCH): کلید فعال شما ({actual_fp}) با کلید Daqiq ({expected_fp}) یکسان نیست! لایسنس‌ها رد خواهند شد."
                            ))
                            .font(FONT_REGULAR)
                            .size(12),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 14])
                    .style(move |_th| warning_pill_style(is_dark)),

                    PublicKeyMatchResult::Invalid(msg) => container(
                        row![
                            text("❌").size(16),
                            text(format!("کلید ورودی نامعتبر است: {msg}")).font(FONT_REGULAR).size(12),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 14])
                    .style(move |_th| status_pill_style(false, is_dark)),
                }
            } else {
                container(
                    text("کلید عمومی را وارد کرده و دکمه بررسی را بزنید.").font(FONT_REGULAR).size(11),
                )
                .padding([4, 10])
                .style(move |th| subcard_style(th, is_dark))
            },
        ]
        .spacing(10),
    )
    .padding(16)
    .style(move |th| card_style(th, is_dark));

    // 3. Vault Operations (Unlock / Create)
    let unlock_card = container(
        column![
            text("بازگشایی مخزن کلید موجود (.dqvault)").font(FONT_BOLD).size(14),
            text("فایل گاوصندوق کلید اختصاصی خود را انتخاب کرده و رمز عبور آن را وارد کنید:")
                .font(FONT_REGULAR)
                .size(12),
            row![
                text_input("رمز عبور مخزن کلید", &state.unlock_passphrase)
                    .on_input(Message::UnlockPassphraseChanged)
                    .secure(true)
                    .font(FONT_MONO)
                    .padding(8),
                button(text("انتخاب فایل و بازگشایی 📂").font(FONT_REGULAR).size(12))
                    .on_press(Message::UnlockVaultClicked)
                    .padding([8, 14]),
            ]
            .spacing(10),
            if let Some((ok, msg)) = &state.unlock_status_msg {
                let is_ok = *ok;
                container(text(msg).font(FONT_REGULAR).size(12))
                    .padding([6, 12])
                    .style(move |_th| status_pill_style(is_ok, is_dark))
            } else {
                container(text("").size(0))
            },
        ]
        .spacing(8),
    )
    .padding(16)
    .style(move |th| card_style(th, is_dark));

    let create_card = container(
        column![
            text("ساخت مخزن کلید جدید (New Key Vault)").font(FONT_BOLD).size(14),
            text("یک جفت کلید Ed25519 امن تولید کرده و آن را با رمز عبور قوی رمزنگاری نمایید:")
                .font(FONT_REGULAR)
                .size(12),
            row![
                column![
                    text("شناسه کلید (Key ID)").font(FONT_REGULAR).size(11),
                    text_input("daqiq-prod-v1", &state.new_key_id)
                        .on_input(Message::NewKeyIdChanged)
                        .font(FONT_MONO)
                        .padding(8),
                ]
                .spacing(3)
                .width(Length::FillPortion(1)),
                column![
                    text("رمز عبور قوی (حداقل ۶ نویسه)").font(FONT_REGULAR).size(11),
                    text_input("رمز عبور محافظت از مخزن", &state.new_passphrase)
                        .on_input(Message::NewPassphraseChanged)
                        .secure(true)
                        .font(FONT_MONO)
                        .padding(8),
                ]
                .spacing(3)
                .width(Length::FillPortion(1)),
                button(text("تولید و ذخیره مخزن 🔐").font(FONT_REGULAR).size(12))
                    .on_press(Message::CreateVaultClicked)
                    .padding([8, 14]),
            ]
            .spacing(10)
            .align_y(Alignment::End),
            if let Some((ok, msg)) = &state.create_status_msg {
                let is_ok = *ok;
                container(text(msg).font(FONT_REGULAR).size(12))
                    .padding([6, 12])
                    .style(move |_th| status_pill_style(is_ok, is_dark))
            } else {
                container(text("").size(0))
            },
        ]
        .spacing(8),
    )
    .padding(16)
    .style(move |th| card_style(th, is_dark));

    // 4. About & Invariants
    let about_card = container(
        column![
            row![
                text("ℹ️ درباره استودیو صدور لایسنس دقیق").font(FONT_BOLD).size(14),
                Space::new().width(Length::Fill),
                text("نسخه ۱.۳.۰ — Pure Native Rust").font(FONT_MONO).size(11),
            ]
            .align_y(Alignment::Center),

            text("این نرم‌افزار به صورت ۱۰۰٪ آفلاین و بدون وابستگی به WebView2، پایتون یا کتابخانه‌های جانبی اجرا می‌شود. استانداردهای رمزنگاری لایسنس بر پایه Ed25519 و کانویکال‌سازی UTF-8 کاملاً با Daqiq v4.9.1 تطابق ۱:۱ دارند.")
                .font(FONT_REGULAR)
                .size(12),
        ]
        .spacing(6),
    )
    .padding(16)
    .style(move |th| card_style(th, is_dark));

    let content = column![
        key_info_card,
        pairing_card,
        row![
            unlock_card.width(Length::FillPortion(1)),
            create_card.width(Length::FillPortion(1)),
        ]
        .spacing(12),
        about_card,
        Space::new().height(Length::Fixed(20.0)),
    ]
    .spacing(14)
    .padding(16);

    scrollable(content).height(Length::Fill).into()
}
