use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::rtl::utc_iso_to_jalali_display;
use crate::state::{AppState, Message, ThemeMode};
use crate::theme::{card_style, code_box_style, FONT_BOLD, FONT_MONO, FONT_REGULAR};

pub fn view_registry(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme_mode == ThemeMode::Dark;

    // Stat cards
    let total_count = state.records.len();
    let perpetual_count = state
        .records
        .iter()
        .filter(|r| r.license_type == "perpetual")
        .count();
    let subscription_count = total_count.saturating_sub(perpetual_count);

    let make_stat_card = |label: &'static str, value: usize, icon: &'static str| {
        container(
            row![
                text(icon).size(22),
                column![
                    text(value.to_string()).font(FONT_BOLD).size(18),
                    text(label).font(FONT_REGULAR).size(12),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([12, 16])
        .width(Length::FillPortion(1))
        .style(move |th| card_style(th, is_dark))
    };

    let stats_row = row![
        make_stat_card("کل لایسنس‌های ثبت شده", total_count, "📋"),
        make_stat_card("لایسنس‌های اشتراکی", subscription_count, "⏳"),
        make_stat_card("لایسنس‌های دائمی", perpetual_count, "♾️"),
    ]
    .spacing(16);

    // Search & Actions bar
    let search_bar = row![
        text_input(
            "جستجو بر اساس نام، شناسه لایسنس، تلفن، HWID...",
            &state.ledger_search,
        )
        .on_input(Message::LedgerSearchChanged)
        .font(FONT_REGULAR)
        .padding(8)
        .width(Length::FillPortion(3)),
        button(text("📊 خروجی اکسل / CSV").font(FONT_REGULAR).size(13))
            .on_press(Message::ExportLedgerCsv)
            .padding([8, 16]),
        button(text("🔄 به‌روزرسانی").font(FONT_REGULAR).size(13))
            .on_press(Message::RefreshLedger)
            .padding([8, 16]),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    // Table Header
    let table_header = container(
        row![
            text("شناسه لایسنس").font(FONT_BOLD).size(12).width(Length::FillPortion(2)),
            text("نام مشتری / سازمان").font(FONT_BOLD).size(12).width(Length::FillPortion(3)),
            text("نوع").font(FONT_BOLD).size(12).width(Length::FillPortion(1)),
            text("تاریخ صدور").font(FONT_BOLD).size(12).width(Length::FillPortion(2)),
            text("تاریخ انقضا").font(FONT_BOLD).size(12).width(Length::FillPortion(2)),
            text("دستگاه‌ها").font(FONT_BOLD).size(12).width(Length::FillPortion(1)),
            text("عملیات").font(FONT_BOLD).size(12).width(Length::FillPortion(2)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([10, 16])
    .style(move |_| code_box_style(is_dark));

    // Table Rows
    let mut table_rows = column![].spacing(6);

    if state.records.is_empty() {
        let empty_msg = container(
            text("هیچ لایسنسی در پایگاه داده محلی یافت نشد.")
                .font(FONT_REGULAR)
                .size(13),
        )
        .padding(30)
        .center_x(Length::Fill);

        table_rows = table_rows.push(empty_msg);
    } else {
        for rec in &state.records {
            let lic_id = rec.license_id.clone();
            let is_perp = rec.license_type == "perpetual";
            let exp_display = if is_perp {
                "مادام‌العمر".to_string()
            } else if let Some(exp) = &rec.expires_at {
                utc_iso_to_jalali_display(exp)
            } else {
                "—".to_string()
            };

            let row_item = container(
                row![
                    text(&rec.license_id).font(FONT_MONO).size(11).width(Length::FillPortion(2)),
                    text(&rec.customer_name).font(FONT_REGULAR).size(12).width(Length::FillPortion(3)),
                    text(if is_perp { "دائمی" } else { "اشتراکی" })
                        .font(FONT_REGULAR)
                        .size(11)
                        .width(Length::FillPortion(1)),
                    text(utc_iso_to_jalali_display(&rec.issued_at))
                        .font(FONT_MONO)
                        .size(11)
                        .width(Length::FillPortion(2)),
                    text(exp_display).font(FONT_MONO).size(11).width(Length::FillPortion(2)),
                    text(format!("{} دستگاه", rec.max_devices))
                        .font(FONT_REGULAR)
                        .size(11)
                        .width(Length::FillPortion(1)),
                    button(text("تمدید و صدور جدید").font(FONT_REGULAR).size(11))
                        .on_press(Message::CloneAndRenew(lic_id))
                        .padding([4, 8])
                        .width(Length::FillPortion(2)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding([8, 16])
            .style(move |th| card_style(th, is_dark));

            table_rows = table_rows.push(row_item);
        }
    }

    let main_card = container(
        column![
            row![
                text("دفتر ثبت لایسنس‌ها و فروش‌های انجام‌شده").font(FONT_BOLD).size(15),
                Space::new().width(Length::Fill),
                text("پایگاه داده رمزنگاری‌شده محلی (SQLite Air-Gapped)").font(FONT_REGULAR).size(12),
            ]
            .align_y(Alignment::Center),
            search_bar,
            table_header,
            scrollable(table_rows).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill),
    )
    .padding(16)
    .height(Length::Fill)
    .style(move |th| card_style(th, is_dark));

    let layout = column![
        stats_row,
        main_card,
    ]
    .spacing(16)
    .height(Length::Fill)
    .padding(20);

    layout.into()
}
