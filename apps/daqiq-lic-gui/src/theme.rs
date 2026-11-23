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

// ================= MODERN BALANCED THEMES =================
// Dark: Comfortable Slate-Charcoal (#1E222B) — NOT Pitch-Black!
// Light: Crisp Daylight Minimalism (#F4F6FA)

pub fn dark_theme() -> Theme {
    Theme::custom(
        "DaqiqDarkModern".to_string(),
        Palette {
            background: Color::from_rgb8(30, 34, 43), // #1E222B Canvas
            text: Color::from_rgb8(238, 242, 246),      // #EEF2F6 Primary Text
            primary: Color::from_rgb8(79, 134, 247),    // #4F86F7 Accent
            success: Color::from_rgb8(74, 222, 128),    // #4ADE80
            warning: Color::from_rgb8(251, 191, 36),    // #FBBF24
            danger: Color::from_rgb8(248, 113, 113),    // #F87171
        },
    )
}

pub fn light_theme() -> Theme {
    Theme::custom(
        "DaqiqLightModern".to_string(),
        Palette {
            background: Color::from_rgb8(244, 246, 250), // #F4F6FA Canvas
            text: Color::from_rgb8(17, 24, 39),          // #111827 Primary Text
            primary: Color::from_rgb8(37, 99, 235),      // #2563EB Accent
            success: Color::from_rgb8(21, 128, 61),      // #15803D
            warning: Color::from_rgb8(180, 83, 9),       // #B45309
            danger: Color::from_rgb8(185, 28, 28),       // #B91C1C
        },
    )
}

// ================= COMPONENT STYLES =================

pub fn sidebar_style(_theme: &Theme, is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(25, 29, 37).into() // #191D25
        } else {
            Color::WHITE.into()
        }),
        border: border::rounded(0).width(1.0).color(if is_dark {
            Color::from_rgb8(46, 52, 66) // #2E3442
        } else {
            Color::from_rgb8(226, 232, 240) // #E2E8F0
        }),
        text_color: Some(if is_dark {
            Color::from_rgb8(238, 242, 246)
        } else {
            Color::from_rgb8(17, 24, 39)
        }),
        ..Default::default()
    }
}

pub fn card_style(theme: &Theme, is_dark: bool) -> container::Style {
    let p = theme.palette();
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(38, 43, 54).into() // #262B36
        } else {
            Color::WHITE.into()
        }),
        border: border::rounded(12).width(1.0).color(if is_dark {
            Color::from_rgb8(53, 61, 77) // #353D4D
        } else {
            Color::from_rgb8(220, 227, 238) // #DCE3EE
        }),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub fn subcard_style(_theme: &Theme, is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(27, 31, 39).into() // #1B1F27
        } else {
            Color::from_rgb8(249, 250, 252).into()
        }),
        border: border::rounded(8).width(1.0).color(if is_dark {
            Color::from_rgb8(46, 52, 66)
        } else {
            Color::from_rgb8(226, 232, 240)
        }),
        ..Default::default()
    }
}

pub fn top_header_style(theme: &Theme, is_dark: bool) -> container::Style {
    let p = theme.palette();
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(25, 29, 37).into()
        } else {
            Color::WHITE.into()
        }),
        border: border::rounded(0).width(1.0).color(if is_dark {
            Color::from_rgb8(46, 52, 66)
        } else {
            Color::from_rgb8(226, 232, 240)
        }),
        text_color: Some(p.text),
        ..Default::default()
    }
}


pub fn code_box_style(_theme: &Theme, is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(20, 24, 31).into()
        } else {
            Color::from_rgb8(243, 244, 246).into()
        }),
        border: border::rounded(8).width(1.0).color(if is_dark {
            Color::from_rgb8(46, 52, 66)
        } else {
            Color::from_rgb8(209, 213, 219)
        }),
        text_color: Some(if is_dark {
            Color::from_rgb8(226, 232, 240)
        } else {
            Color::from_rgb8(31, 41, 55)
        }),
        ..Default::default()
    }
}

pub fn status_pill_style(is_success: bool, is_dark: bool) -> container::Style {
    if is_success {
        container::Style {
            background: Some(if is_dark {
                Color::from_rgb8(31, 58, 46).into() // #1F3A2E
            } else {
                Color::from_rgb8(220, 252, 231).into() // #DCFCE7
            }),
            border: border::rounded(6).width(1.0).color(if is_dark {
                Color::from_rgb8(74, 222, 128)
            } else {
                Color::from_rgb8(21, 128, 61)
            }),
            text_color: Some(if is_dark {
                Color::from_rgb8(74, 222, 128)
            } else {
                Color::from_rgb8(21, 128, 61)
            }),
            ..Default::default()
        }
    } else {
        container::Style {
            background: Some(if is_dark {
                Color::from_rgb8(63, 35, 40).into() // #3F2328
            } else {
                Color::from_rgb8(254, 226, 226).into() // #FEE2E2
            }),
            border: border::rounded(6).width(1.0).color(if is_dark {
                Color::from_rgb8(248, 113, 113)
            } else {
                Color::from_rgb8(185, 28, 28)
            }),
            text_color: Some(if is_dark {
                Color::from_rgb8(248, 113, 113)
            } else {
                Color::from_rgb8(185, 28, 28)
            }),
            ..Default::default()
        }
    }
}

pub fn warning_pill_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark {
            Color::from_rgb8(61, 50, 29).into() // #3D321D
        } else {
            Color::from_rgb8(254, 243, 199).into() // #FEF3C7
        }),
        border: border::rounded(6).width(1.0).color(if is_dark {
            Color::from_rgb8(251, 191, 36)
        } else {
            Color::from_rgb8(180, 83, 9)
        }),
        text_color: Some(if is_dark {
            Color::from_rgb8(251, 191, 36)
        } else {
            Color::from_rgb8(180, 83, 9)
        }),
        ..Default::default()
    }
}
