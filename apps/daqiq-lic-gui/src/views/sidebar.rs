use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::state::{ActiveTab, AppState, Message, ThemeMode};
use crate::theme::{
    sidebar_style, status_pill_style, subcard_style, FONT_BOLD, FONT_MONO, FONT_REGULAR,
};

pub fn view_sidebar(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // 1. Top Brand Header
    let brand_header = column![
        row![
            text("🛡️").size(24),
            column![
                text("مولد لایسنس دقیق").font(FONT_BOLD).size(15),
                row![
                    text("v1.3.0").font(FONT_MONO).size(11),
                    container(text("Native Rust").font(FONT_REGULAR).size(10))
                        .padding([1, 6])
                        .style(move |_th| status_pill_style(true, is_dark)),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ]
            .spacing(2),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .padding([16, 14]);

    // 2. Navigation Items
    let make_nav_item = |tab: ActiveTab, label: &'static str, icon: &'static str| {
        let is_active = state.active_tab == tab;
        let content = row![
            text(icon).size(15),
            text(label).font(if is_active { FONT_BOLD } else { FONT_REGULAR }).size(13),
            Space::new().width(Length::Fill),
            if is_active {
                text("◀").size(11)
            } else {
                text("")
            },
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        button(content)
            .width(Length::Fill)
            .padding([10, 12])
            .on_press(Message::SwitchTab(tab))
    };

    let nav_items = column![
        make_nav_item(ActiveTab::Generator, "تولید لایسنس", "✨"),
        make_nav_item(ActiveTab::Ledger, "سوابق لایسنس‌ها", "📊"),
        make_nav_item(ActiveTab::Inspector, "بررسی و شبیه‌ساز", "🔍"),
        make_nav_item(ActiveTab::Vault, "گاوصندوق و تنظیمات", "🔐"),
    ]
    .spacing(4)
    .padding([8, 10]);

    // 3. Bottom Vault Status Card
    let is_vault_open = state.vault.is_some();
    let vault_status_card = container(
        column![
            row![
                text(if is_vault_open { "🔓" } else { "🔒" }).size(14),
                text(if is_vault_open { "مخزن کلید: فعال" } else { "مخزن کلید: قفل" })
                    .font(FONT_BOLD)
                    .size(11),
                Space::new().width(Length::Fill),
                button(text(if is_vault_open { "قفل" } else { "بازگشایی" }).font(FONT_REGULAR).size(10))
                    .on_press(if is_vault_open {
                        Message::LockVaultClicked
                    } else {
                        Message::SwitchTab(ActiveTab::Vault)
                    })
                    .padding([2, 6]),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            row![
                text("اثر انگشت:").font(FONT_REGULAR).size(10),
                text(&state.active_key_fingerprint).font(FONT_MONO).size(10),
            ]
            .spacing(4),
        ]
        .spacing(6),
    )
    .padding(10)
    .style(move |th| subcard_style(th, is_dark));

    // 4. Theme Switcher Pill
    let theme_btn_label = if is_dark { "☀️ پوسته روشن" } else { "🌙 پوسته تاریک" };
    let theme_btn = button(
        row![
            text(if is_dark { "☀️" } else { "🌙" }).size(13),
            text(theme_btn_label).font(FONT_REGULAR).size(12),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .on_press(Message::ToggleTheme)
    .padding([8, 12]);

    let bottom_section = column![
        vault_status_card,
        theme_btn,
    ]
    .spacing(8)
    .padding([10, 10]);

    let sidebar_layout = column![
        brand_header,
        nav_items,
        Space::new().height(Length::Fill),
        bottom_section,
    ]
    .width(Length::Fixed(220.0))
    .height(Length::Fill);

    container(sidebar_layout)
        .style(move |th| sidebar_style(th, is_dark))
        .into()
}
