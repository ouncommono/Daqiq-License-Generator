use windows_sys::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, DeleteObject, HBRUSH, HPEN, PS_SOLID,
};

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub mode: ThemeMode,
    pub canvas_bg: u32,
    pub sidebar_bg: u32,
    pub sidebar_divider: u32,
    pub card_bg: u32,
    pub card_border: u32,
    pub input_bg: u32,
    pub input_border: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub success_fg: u32,
    pub success_bg: u32,
    pub warning_fg: u32,
    pub warning_bg: u32,
    pub danger_fg: u32,
    pub danger_bg: u32,
}

pub fn dark_theme() -> ThemeColors {
    ThemeColors {
        mode: ThemeMode::Dark,
        canvas_bg: rgb(30, 34, 43),        // #1E222B Soft Charcoal
        sidebar_bg: rgb(24, 28, 36),       // #181C24
        sidebar_divider: rgb(46, 53, 68),  // #2E3544
        card_bg: rgb(38, 43, 54),          // #262B36
        card_border: rgb(54, 62, 79),      // #363E4F
        input_bg: rgb(27, 31, 40),         // #1B1F28
        input_border: rgb(64, 74, 94),     // #404A5E
        text_primary: rgb(238, 242, 246),  // #EEF2F6
        text_secondary: rgb(168, 179, 199),// #A8B3C7
        text_muted: rgb(126, 138, 159),    // #7E8A9F
        accent: rgb(79, 134, 247),         // #4F86F7
        accent_hover: rgb(101, 150, 250),  // #6596FA
        success_fg: rgb(74, 222, 128),     // #4ADE80
        success_bg: rgb(31, 58, 46),       // #1F3A2E
        warning_fg: rgb(251, 191, 36),     // #FBBF24
        warning_bg: rgb(61, 50, 29),       // #3D321D
        danger_fg: rgb(248, 113, 113),     // #F87171
        danger_bg: rgb(63, 35, 40),        // #3F2328
    }
}

pub fn light_theme() -> ThemeColors {
    ThemeColors {
        mode: ThemeMode::Light,
        canvas_bg: rgb(244, 246, 250),     // #F4F6FA Crisp Light
        sidebar_bg: rgb(255, 255, 255),    // #FFFFFF
        sidebar_divider: rgb(226, 232, 240), // #E2E8F0
        card_bg: rgb(255, 255, 255),       // #FFFFFF
        card_border: rgb(220, 227, 238),   // #DCE3EE
        input_bg: rgb(250, 251, 253),      // #FAFBFD
        input_border: rgb(203, 213, 225),  // #CBD5E1
        text_primary: rgb(17, 24, 39),     // #111827
        text_secondary: rgb(75, 85, 99),   // #4B5563
        text_muted: rgb(107, 114, 128),    // #6B7280
        accent: rgb(37, 99, 235),          // #2563EB
        accent_hover: rgb(29, 78, 216),    // #1D4ED8
        success_fg: rgb(21, 128, 61),      // #15803D
        success_bg: rgb(220, 252, 231),    // #DCFCE7
        warning_fg: rgb(180, 83, 9),       // #B45309
        warning_bg: rgb(254, 243, 199),    // #FEF3C7
        danger_fg: rgb(185, 28, 28),       // #B91C1C
        danger_bg: rgb(254, 226, 226),     // #FEE2E2
    }
}

pub struct ThemeGdiResources {
    pub colors: ThemeColors,
    pub brush_canvas: HBRUSH,
    pub brush_sidebar: HBRUSH,
    pub brush_card: HBRUSH,
    pub brush_input: HBRUSH,
    pub brush_accent: HBRUSH,
    pub brush_success_bg: HBRUSH,
    pub brush_warning_bg: HBRUSH,
    pub brush_danger_bg: HBRUSH,

    pub pen_card_border: HPEN,
    pub pen_input_border: HPEN,
    pub pen_sidebar_divider: HPEN,
    pub pen_accent: HPEN,
    pub pen_null: HPEN,
}

impl ThemeGdiResources {
    pub fn new(mode: ThemeMode) -> Self {
        let colors = match mode {
            ThemeMode::Dark => dark_theme(),
            ThemeMode::Light => light_theme(),
        };

        unsafe {
            let brush_canvas = CreateSolidBrush(colors.canvas_bg);
            let brush_sidebar = CreateSolidBrush(colors.sidebar_bg);
            let brush_card = CreateSolidBrush(colors.card_bg);
            let brush_input = CreateSolidBrush(colors.input_bg);
            let brush_accent = CreateSolidBrush(colors.accent);
            let brush_success_bg = CreateSolidBrush(colors.success_bg);
            let brush_warning_bg = CreateSolidBrush(colors.warning_bg);
            let brush_danger_bg = CreateSolidBrush(colors.danger_bg);

            let pen_card_border = CreatePen(PS_SOLID as i32, 1, colors.card_border);
            let pen_input_border = CreatePen(PS_SOLID as i32, 1, colors.input_border);
            let pen_sidebar_divider = CreatePen(PS_SOLID as i32, 1, colors.sidebar_divider);
            let pen_accent = CreatePen(PS_SOLID as i32, 1, colors.accent);
            let pen_null = CreatePen(windows_sys::Win32::Graphics::Gdi::PS_NULL as i32, 0, 0);

            Self {
                colors,
                brush_canvas,
                brush_sidebar,
                brush_card,
                brush_input,
                brush_accent,
                brush_success_bg,
                brush_warning_bg,
                brush_danger_bg,
                pen_card_border,
                pen_input_border,
                pen_sidebar_divider,
                pen_accent,
                pen_null,
            }
        }
    }
}

impl Drop for ThemeGdiResources {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.brush_canvas);
            DeleteObject(self.brush_sidebar);
            DeleteObject(self.brush_card);
            DeleteObject(self.brush_input);
            DeleteObject(self.brush_accent);
            DeleteObject(self.brush_success_bg);
            DeleteObject(self.brush_warning_bg);
            DeleteObject(self.brush_danger_bg);

            DeleteObject(self.pen_card_border);
            DeleteObject(self.pen_input_border);
            DeleteObject(self.pen_sidebar_divider);
            DeleteObject(self.pen_accent);
            DeleteObject(self.pen_null);
        }
    }
}
