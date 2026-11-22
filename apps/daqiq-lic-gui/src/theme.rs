use iced::theme::Palette;
use iced::widget::container;
use iced::{border, Color, Font, Theme};

pub const FONT_REGULAR: Font = Font {
    family: iced::font::Family::Name("Vazirmatn"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const FONT_BOLD: Font = Font {
    family: iced::font::Family::Name("Vazirmatn"),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const FONT_MONO: Font = Font {
    family: iced::font::Family::Name("JetBrains Mono"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub fn dark_theme() -> Theme {
    Theme::custom(
        "DaqiqDark".to_string(),
        Palette {
            background: Color::from_rgb8(11, 17, 32),
            text: Color::from_rgb8(241, 245, 249),
            primary: Color::from_rgb8(59, 130, 246),
            success: Color::from_rgb8(34, 197, 94),
            warning: Color::from_rgb8(245, 158, 11),
            danger: Color::from_rgb8(239, 68, 68),
        },
    )
}

pub fn light_theme() -> Theme {
    Theme::custom(
        "DaqiqLight".to_string(),
        Palette {
            background: Color::from_rgb8(244, 246, 249),
            text: Color::from_rgb8(15, 23, 42),
            primary: Color::from_rgb8(29, 78, 216),
            success: Color::from_rgb8(21, 128, 61),
            warning: Color::from_rgb8(180, 83, 9),
            danger: Color::from_rgb8(185, 28, 28),
        },
    )
}

pub fn card_style(theme: &Theme, is_dark: bool) -> container::Style {
    let p = theme.palette();
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(19, 28, 49).into()
        } else {
            Color::WHITE.into()
        }),
        border: border::rounded(8).width(1.0).color(if is_dark {
            Color::from_rgb8(41, 56, 85)
        } else {
            Color::from_rgb8(203, 213, 225)
        }),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub fn header_bar_style(theme: &Theme, is_dark: bool) -> container::Style {
    let p = theme.palette();
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(15, 23, 42).into()
        } else {
            Color::from_rgb8(255, 255, 255).into()
        }),
        border: border::rounded(0).width(1.0).color(if is_dark {
            Color::from_rgb8(30, 41, 59)
        } else {
            Color::from_rgb8(226, 232, 240)
        }),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub fn nav_bar_style(theme: &Theme, is_dark: bool) -> container::Style {
    let p = theme.palette();
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(15, 23, 42).into()
        } else {
            Color::from_rgb8(248, 250, 252).into()
        }),
        border: border::rounded(0).width(1.0).color(if is_dark {
            Color::from_rgb8(30, 41, 59)
        } else {
            Color::from_rgb8(226, 232, 240)
        }),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub fn code_box_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(10, 15, 26).into()
        } else {
            Color::from_rgb8(241, 245, 249).into()
        }),
        border: border::rounded(6).width(1.0).color(if is_dark {
            Color::from_rgb8(30, 41, 59)
        } else {
            Color::from_rgb8(203, 213, 225)
        }),
        text_color: Some(if is_dark {
            Color::from_rgb8(226, 232, 240)
        } else {
            Color::from_rgb8(15, 23, 42)
        }),
        ..Default::default()
    }
}

pub fn status_pill_style(is_success: bool, is_dark: bool) -> container::Style {
    if is_success {
        container::Style {
            background: Some(if is_dark {
                Color::from_rgb8(20, 83, 45).into()
            } else {
                Color::from_rgb8(220, 252, 231).into()
            }),
            border: border::rounded(12).width(1.0).color(if is_dark {
                Color::from_rgb8(34, 197, 94)
            } else {
                Color::from_rgb8(134, 239, 172)
            }),
            text_color: Some(if is_dark {
                Color::from_rgb8(240, 253, 244)
            } else {
                Color::from_rgb8(21, 128, 61)
            }),
            ..Default::default()
        }
    } else {
        container::Style {
            background: Some(if is_dark {
                Color::from_rgb8(127, 29, 29).into()
            } else {
                Color::from_rgb8(254, 226, 226).into()
            }),
            border: border::rounded(12).width(1.0).color(if is_dark {
                Color::from_rgb8(239, 68, 68)
            } else {
                Color::from_rgb8(252, 165, 165)
            }),
            text_color: Some(if is_dark {
                Color::from_rgb8(254, 242, 242)
            } else {
                Color::from_rgb8(185, 28, 28)
            }),
            ..Default::default()
        }
    }
}
