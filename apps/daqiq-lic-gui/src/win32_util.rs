#![allow(dead_code)]

use std::ffi::c_void;
use windows_sys::Win32::Foundation::{HANDLE, HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, CreateFontW, DeleteObject, DrawTextW, RoundRect, SelectObject,
    SetBkMode, SetTextColor, DT_CENTER, DT_SINGLELINE, DT_VCENTER, FW_BOLD, FW_NORMAL, HBRUSH,
    HDC, HFONT, HPEN, TRANSPARENT,
};
use windows_sys::Win32::UI::Controls::{
    LVM_DELETEALLITEMS, LVM_INSERTCOLUMNW, LVM_INSERTITEMW, LVM_SETEXTENDEDLISTVIEWSTYLE,
    LVM_SETITEMTEXTW, LVS_EX_DOUBLEBUFFER, LVS_EX_FULLROWSELECT, LVS_REPORT, LVS_SHOWSELALWAYS,
    LVS_SINGLESEL, LVCF_TEXT, LVCF_WIDTH, LVCOLUMNW, LVIF_TEXT, LVITEMW, WC_LISTVIEWW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetWindowTextLengthW, GetWindowTextW, SendMessageW, SetWindowTextW,
    WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE, WS_EX_RIGHT, WS_EX_RTLREADING, WS_TABSTOP, WS_VISIBLE,
};

extern "system" {
    fn EnableWindow(hWnd: HWND, bEnable: i32) -> i32;
}

// Standard Win32 Button Styles & Messages
pub const BS_PUSHBUTTON: u32 = 0x00000000;
pub const BS_AUTOCHECKBOX: u32 = 0x00000003;
pub const BM_GETCHECK: u32 = 0x00F0;
pub const BM_SETCHECK: u32 = 0x00F1;
pub const BST_UNCHECKED: usize = 0x0000;
pub const BST_CHECKED: usize = 0x0001;

// Standard Win32 Edit Styles
pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_AUTOVSCROLL: u32 = 0x0040;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const ES_READONLY: u32 = 0x0800;
pub const ES_PASSWORD: u32 = 0x0020;

// Standard Win32 Combobox Styles & Messages
pub const CBS_DROPDOWNLIST: u32 = 0x0003;
pub const CB_ADDSTRING: u32 = 0x0143;
pub const CB_SETCURSEL: u32 = 0x014E;
pub const CB_GETCURSEL: u32 = 0x0147;
pub const CB_RESETCONTENT: u32 = 0x014B;

pub fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wstring(w: &[u16]) -> String {
    let len = w.iter().position(|&c| c == 0).unwrap_or(w.len());
    String::from_utf16_lossy(&w[..len])
}

pub fn get_window_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if read > 0 {
            from_wstring(&buf[..read as usize])
        } else {
            String::new()
        }
    }
}

pub fn set_window_text(hwnd: HWND, text: &str) {
    let w = to_wstring(text);
    unsafe {
        SetWindowTextW(hwnd, w.as_ptr());
    }
}

pub fn enable_window(hwnd: HWND, enable: bool) {
    unsafe {
        EnableWindow(hwnd, if enable { 1 } else { 0 });
    }
}

pub struct Fonts {
    pub regular: HFONT,
    pub bold: HFONT,
    pub heading: HFONT,
    pub subheading: HFONT,
    pub small: HFONT,
    pub mono: HFONT,
}

impl Fonts {
    pub fn new() -> Self {
        unsafe {
            // Load embedded TTF fonts into process memory
            let font_reg_bytes = include_bytes!("../assets/fonts/Vazirmatn-Regular.ttf");
            let mut num_fonts = 0u32;
            let _ = AddFontMemResourceEx(
                font_reg_bytes.as_ptr() as *mut c_void,
                font_reg_bytes.len() as u32,
                std::ptr::null_mut(),
                &mut num_fonts,
            );

            let font_bold_bytes = include_bytes!("../assets/fonts/Vazirmatn-Bold.ttf");
            let _ = AddFontMemResourceEx(
                font_bold_bytes.as_ptr() as *mut c_void,
                font_bold_bytes.len() as u32,
                std::ptr::null_mut(),
                &mut num_fonts,
            );

            let font_mono_bytes = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
            let _ = AddFontMemResourceEx(
                font_mono_bytes.as_ptr() as *mut c_void,
                font_mono_bytes.len() as u32,
                std::ptr::null_mut(),
                &mut num_fonts,
            );

            let vazir = to_wstring("Vazirmatn");
            let mono_name = to_wstring("JetBrains Mono");

            let regular = CreateFontW(
                -14, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, 0, 0, 0, 5, 0, vazir.as_ptr(),
            );
            let bold = CreateFontW(
                -14, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 0, 0, 0, 5, 0, vazir.as_ptr(),
            );
            let heading = CreateFontW(
                -18, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 0, 0, 0, 5, 0, vazir.as_ptr(),
            );
            let subheading = CreateFontW(
                -15, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 0, 0, 0, 5, 0, vazir.as_ptr(),
            );
            let small = CreateFontW(
                -11, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, 0, 0, 0, 5, 0, vazir.as_ptr(),
            );
            let mono = CreateFontW(
                -13, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, 0, 0, 0, 5, 0, mono_name.as_ptr(),
            );

            Self {
                regular,
                bold,
                heading,
                subheading,
                small,
                mono,
            }
        }
    }

    pub fn apply(&self, hwnd: HWND, font: HFONT) {
        unsafe {
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT,
                font as usize,
                1,
            );
        }
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.regular);
            DeleteObject(self.bold);
            DeleteObject(self.heading);
            DeleteObject(self.subheading);
            DeleteObject(self.small);
            DeleteObject(self.mono);
        }
    }
}

pub fn create_label(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
) -> HWND {
    let class_name = to_wstring("STATIC");
    let text_w = to_wstring(text);
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_RTLREADING | WS_EX_RIGHT,
            class_name.as_ptr(),
            text_w.as_ptr(),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        hwnd
    }
}

pub fn create_label_left(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
) -> HWND {
    let class_name = to_wstring("STATIC");
    let text_w = to_wstring(text);
    unsafe {
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            text_w.as_ptr(),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        hwnd
    }
}

pub fn create_edit(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
    rtl: bool,
    multiline: bool,
    readonly: bool,
    password: bool,
) -> HWND {
    let class_name = to_wstring("EDIT");
    let text_w = to_wstring(text);
    let mut style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER;
    let mut ex_style = WS_EX_CLIENTEDGE;

    if rtl {
        ex_style |= WS_EX_RTLREADING | WS_EX_RIGHT;
    }
    if multiline {
        style |= ES_MULTILINE | ES_AUTOVSCROLL;
    } else {
        style |= ES_AUTOHSCROLL;
    }
    if readonly {
        style |= ES_READONLY;
    }
    if password {
        style |= ES_PASSWORD;
    }

    unsafe {
        let hwnd = CreateWindowExW(
            ex_style,
            class_name.as_ptr(),
            text_w.as_ptr(),
            style,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        hwnd
    }
}

pub fn create_button(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
) -> HWND {
    let class_name = to_wstring("BUTTON");
    let text_w = to_wstring(text);
    unsafe {
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            text_w.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        hwnd
    }
}

pub fn create_checkbox(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
    checked: bool,
) -> HWND {
    let class_name = to_wstring("BUTTON");
    let text_w = to_wstring(text);
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_RTLREADING | WS_EX_RIGHT,
            class_name.as_ptr(),
            text_w.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        if checked {
            SendMessageW(hwnd, BM_SETCHECK, BST_CHECKED, 0);
        }
        hwnd
    }
}

pub fn get_checkbox_checked(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, 0, 0) == BST_CHECKED as isize }
}

pub fn set_checkbox_checked(hwnd: HWND, checked: bool) {
    unsafe {
        SendMessageW(
            hwnd,
            BM_SETCHECK,
            if checked { BST_CHECKED } else { BST_UNCHECKED },
            0,
        );
    }
}

pub fn create_combobox(
    parent: HWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
) -> HWND {
    let class_name = to_wstring("COMBOBOX");
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_RTLREADING | WS_EX_RIGHT,
            class_name.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | CBS_DROPDOWNLIST | WS_BORDER,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        hwnd
    }
}

pub fn add_combobox_item(hwnd: HWND, text: &str) {
    let text_w = to_wstring(text);
    unsafe {
        SendMessageW(hwnd, CB_ADDSTRING, 0, text_w.as_ptr() as isize);
    }
}

pub fn get_combobox_selected_index(hwnd: HWND) -> i32 {
    unsafe { SendMessageW(hwnd, CB_GETCURSEL, 0, 0) as i32 }
}

pub fn set_combobox_selected_index(hwnd: HWND, idx: i32) {
    unsafe {
        SendMessageW(hwnd, CB_SETCURSEL, idx as usize, 0);
    }
}

pub fn clear_combobox(hwnd: HWND) {
    unsafe {
        SendMessageW(hwnd, CB_RESETCONTENT, 0, 0);
    }
}

pub fn create_listview(
    parent: HWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    font: HFONT,
) -> HWND {
    unsafe {
        let class_name = WC_LISTVIEWW;
        let hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            class_name,
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | LVS_REPORT as u32 | LVS_SINGLESEL as u32 | LVS_SHOWSELALWAYS as u32,
            x,
            y,
            w,
            h,
            parent,
            id as HANDLE,
            0 as HANDLE,
            std::ptr::null_mut(),
        );
        SendMessageW(
            hwnd,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            (LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER) as usize,
            (LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER) as isize,
        );
        SendMessageW(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::WM_SETFONT, font as usize, 1);
        hwnd
    }
}

pub fn add_listview_column(hwnd: HWND, col_idx: i32, width: i32, text: &str) {
    let text_w = to_wstring(text);
    let mut col: LVCOLUMNW = unsafe { std::mem::zeroed() };
    col.mask = LVCF_TEXT | LVCF_WIDTH;
    col.cx = width;
    col.pszText = text_w.as_ptr() as *mut u16;
    col.cchTextMax = text_w.len() as i32;
    col.iSubItem = col_idx;
    col.iOrder = col_idx;

    unsafe {
        SendMessageW(
            hwnd,
            LVM_INSERTCOLUMNW,
            col_idx as usize,
            &mut col as *mut _ as isize,
        );
    }
}

pub fn add_listview_row(hwnd: HWND, row_idx: i32, cols: &[&str]) {
    if cols.is_empty() {
        return;
    }
    let first_w = to_wstring(cols[0]);
    let mut item: LVITEMW = unsafe { std::mem::zeroed() };
    item.mask = LVIF_TEXT;
    item.iItem = row_idx;
    item.iSubItem = 0;
    item.pszText = first_w.as_ptr() as *mut u16;
    item.cchTextMax = first_w.len() as i32;

    unsafe {
        SendMessageW(hwnd, LVM_INSERTITEMW, 0, &mut item as *mut _ as isize);
        for (col_idx, &val) in cols.iter().enumerate().skip(1) {
            let val_w = to_wstring(val);
            let mut sub: LVITEMW = std::mem::zeroed();
            sub.mask = LVIF_TEXT;
            sub.iItem = row_idx;
            sub.iSubItem = col_idx as i32;
            sub.pszText = val_w.as_ptr() as *mut u16;
            sub.cchTextMax = val_w.len() as i32;

            SendMessageW(hwnd, LVM_SETITEMTEXTW, row_idx as usize, &mut sub as *mut _ as isize);
        }
    }
}

pub fn clear_listview(hwnd: HWND) {
    unsafe {
        SendMessageW(hwnd, LVM_DELETEALLITEMS, 0, 0);
    }
}

pub fn draw_rounded_card(
    hdc: HDC,
    rect: &RECT,
    bg_brush: HBRUSH,
    border_pen: HPEN,
    radius: i32,
) {
    unsafe {
        let old_brush = SelectObject(hdc, bg_brush);
        let old_pen = SelectObject(hdc, border_pen);
        RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, radius, radius);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
    }
}

pub fn draw_pill(
    hdc: HDC,
    rect: &RECT,
    bg_brush: HBRUSH,
    border_pen: HPEN,
    text_color: u32,
    text: &str,
    font: HFONT,
) {
    unsafe {
        let old_brush = SelectObject(hdc, bg_brush);
        let old_pen = SelectObject(hdc, border_pen);
        RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);

        let old_font = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);
        let mut text_rect = *rect;
        let text_w = to_wstring(text);
        DrawTextW(
            hdc,
            text_w.as_ptr(),
            text_w.len() as i32 - 1,
            &mut text_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(hdc, old_font);
    }
}
