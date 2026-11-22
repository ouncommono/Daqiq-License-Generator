use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::state::{ActiveTab, AppState, Message, ThemeMode};
use crate::theme::{header_bar_style, nav_bar_style, status_pill_style, FONT_BOLD, FONT_REGULAR};

pub fn view_header(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // Brand & titles
    let brand_row = row![
        text("🛡️").size(24),
        text("استودیو صدور لایسنس دقیق")
            .font(FONT_BOLD)
            .size(18),
        text("نسخه ۱.۲.۰ (موتور امنیتی Pure Rust)")
            .font(FONT_REGULAR)
            .size(12),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    // Vault status pill
    let is_vault_open = state.vault.is_some();
    let vault_status_text = if is_vault_open {
        format!("مخزن کلید: باز ({})", state.active_key_id)
    } else {
        "مخزن کلید: قفل".to_string()
    };

    let vault_pill = button(
        container(
            row![
                text(if is_vault_open { "🔓" } else { "🔒" }).size(14),
                text(vault_status_text).font(FONT_REGULAR).size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .style(move |_| status_pill_style(is_vault_open, is_dark)),
    )
    .on_press(Message::SwitchTab(ActiveTab::Vault))
    .padding(0);

    // Theme toggle
    let theme_btn_label = if is_dark { "☀️ پوسته روشن" } else { "🌙 پوسته تاریک" };
    let theme_btn = button(text(theme_btn_label).font(FONT_REGULAR).size(12))
        .on_press(Message::ToggleTheme)
        .padding([6, 12]);

    let top_bar = row![
        brand_row,
        Space::new().width(Length::Fill),
        vault_pill,
        theme_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([12, 20]);

    // Nav tabs
    let make_tab_btn = |tab: ActiveTab, label: &'static str, icon: &'static str| {
        let is_active = state.active_tab == tab;
        let content = row![
            text(icon).size(15),
            text(label).font(if is_active { FONT_BOLD } else { FONT_REGULAR }).size(13),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        button(content)
            .on_press(Message::SwitchTab(tab))
            .padding([8, 16])
    };

    let nav_bar = row![
        make_tab_btn(ActiveTab::Generator, "صدور لایسنس جدید", "✨"),
        make_tab_btn(ActiveTab::Ledger, "دفتر فروش و بایگانی", "📊"),
        make_tab_btn(ActiveTab::Inspector, "بازرس و شبیه‌ساز دقیق", "🔍"),
        make_tab_btn(ActiveTab::Vault, "مخزن کلیدها و تنظیمات", "🔐"),
    ]
    .spacing(8)
    .padding([6, 20]);

    let mut full_header = iced::widget::column![
        container(top_bar).style(move |th| header_bar_style(th, is_dark)),
        container(nav_bar).style(move |th| nav_bar_style(th, is_dark)),
    ];

    if let Some(msg) = &state.toast_message {
        let toast = container(
            row![
                text("ℹ️").size(16),
                text(msg).font(FONT_REGULAR).size(13),
                Space::new().width(Length::Fill),
                button(text("✕").size(12)).on_press(Message::DismissToast).padding([2, 8]),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([8, 20])
        .style(move |_| status_pill_style(true, is_dark));

        full_header = full_header.push(toast);
    }

    full_header.into()
}
