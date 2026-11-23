use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::state::{ActiveTab, AppState, Message, ThemeMode};
use crate::theme::{status_pill_style, top_header_style, FONT_BOLD, FONT_REGULAR};

pub fn view_header(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    let tab_title = match state.active_tab {
        ActiveTab::Generator => "استودیو صدور لایسنس (.daqlic)",
        ActiveTab::Ledger => "دفتر فروش و سوابق لایسنس‌ها",
        ActiveTab::Inspector => "بازرس و شبیه‌ساز اعتبارسنجی",
        ActiveTab::Vault => "گاوصندوق کلیدها و تطابق با Daqiq",
    };

    let title_row = row![
        text(tab_title).font(FONT_BOLD).size(16),
        container(
            text("سازگار با Daqiq v4.9.1 (PR #39)")
                .font(FONT_REGULAR)
                .size(11),
        )
        .padding([2, 8])
        .style(move |_th| status_pill_style(true, is_dark)),
        Space::new().width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let mut header_col = iced::widget::column![
        container(title_row)
            .padding([12, 20])
            .style(move |th| top_header_style(th, is_dark)),
    ];

    if let Some(msg) = &state.toast_message {
        let toast = container(
            row![
                text("ℹ️").size(15),
                text(msg).font(FONT_REGULAR).size(12),
                Space::new().width(Length::Fill),
                button(text("✕").size(11))
                    .on_press(Message::DismissToast)
                    .padding([2, 6]),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([6, 16])
        .style(move |_th| status_pill_style(true, is_dark));

        header_col = header_col.push(toast);
    }

    header_col.into()
}
