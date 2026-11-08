//! Windows appearance. Painting never calls this except through [`colors`].
//!
//! A Linux or macOS port replaces [`system_color_scheme`] and, if that toolkit
//! does not theme native menus and title bars itself, the two helpers under it.
//! Palettes stay in `eventide::theme`.

use std::cell::Cell;
use std::ffi::CString;

use eventide::{palette, ColorScheme, Palette};
use windows::core::{BOOL, PCSTR};
use windows::Win32::Foundation::{COLORREF, FALSE, HWND, TRUE};
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWINDOWATTRIBUTE,
};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, DeleteObject, HBRUSH};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExA, RegQueryValueExA, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_DWORD,
    REG_VALUE_TYPE,
};

const PERSONALIZE: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
/// `1` follows the system. `2` would force dark and ignore the setting.
const ALLOW_DARK: i32 = 1;
/// Immersive dark mode before `DWMWA_USE_IMMERSIVE_DARK_MODE` was numbered 20.
const DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(19);

thread_local! {
    static SCHEME: Cell<ColorScheme> = const { Cell::new(ColorScheme::Light) };
    static FIELD_BRUSH: Cell<HBRUSH> = const { Cell::new(HBRUSH(std::ptr::null_mut())) };
}

pub fn colors() -> Palette {
    palette(SCHEME.with(|scheme| scheme.get()))
}

pub fn is_dark() -> bool {
    SCHEME.with(|scheme| scheme.get()) == ColorScheme::Dark
}

/// Brush for text fields. Matches [`Palette::surface`].
pub fn field_brush() -> HBRUSH {
    let brush = FIELD_BRUSH.with(|cell| cell.get());
    if brush.is_invalid() {
        unsafe { rebuild_field_brush() }
    } else {
        brush
    }
}

/// Call once, before the first window is created, so menus can follow the system.
pub fn install() {
    allow_dark_menus();
    publish(system_color_scheme());
}

/// Re-read the system setting. Returns whether the palette changed.
pub fn follow_system() -> bool {
    let next = system_color_scheme();
    let changed = SCHEME.with(|scheme| scheme.get() != next);
    if changed {
        publish(next);
        flush_menu_themes();
    }
    changed
}

/// Title bar light or dark. Safe on a window that has no caption.
pub unsafe fn caption(hwnd: HWND) {
    if hwnd.is_invalid() {
        return;
    }
    let dark: BOOL = if is_dark() { TRUE } else { FALSE };
    let data = &dark as *const BOOL as *const core::ffi::c_void;
    let size = std::mem::size_of::<BOOL>() as u32;
    let _ = DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1, data, size);
    let _ = DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, data, size);
}

fn publish(scheme: ColorScheme) {
    SCHEME.with(|cell| cell.set(scheme));
    unsafe { rebuild_field_brush() };
}

unsafe fn rebuild_field_brush() -> HBRUSH {
    let brush = CreateSolidBrush(COLORREF(colors().surface));
    FIELD_BRUSH.with(|cell| {
        let previous = cell.replace(brush);
        if !previous.is_invalid() {
            let _ = DeleteObject(previous.into());
        }
    });
    brush
}

/// App color mode, not the taskbar color mode.
///
/// `AppsUseLightTheme` is `0` for dark and `1` for light. A missing value
/// means light, which is what Windows used before this setting existed.
fn system_color_scheme() -> ColorScheme {
    match unsafe { app_uses_light_theme() } {
        Some(false) => ColorScheme::Dark,
        _ => ColorScheme::Light,
    }
}

unsafe fn app_uses_light_theme() -> Option<bool> {
    let path = CString::new(PERSONALIZE).ok()?;
    let name = CString::new("AppsUseLightTheme").ok()?;
    let mut key = HKEY::default();
    if RegOpenKeyExA(
        HKEY_CURRENT_USER,
        PCSTR(path.as_ptr().cast()),
        Some(0),
        KEY_READ,
        &mut key,
    )
    .is_err()
    {
        return None;
    }
    let mut value = 0u32;
    let mut size = 4u32;
    let mut kind = REG_VALUE_TYPE::default();
    let status = RegQueryValueExA(
        key,
        PCSTR(name.as_ptr().cast()),
        None,
        Some(&mut kind),
        Some((&mut value as *mut u32).cast()),
        Some(&mut size),
    );
    let _ = RegCloseKey(key);
    if status.is_ok() && kind == REG_DWORD && size == 4 {
        Some(value != 0)
    } else {
        None
    }
}

/// `uxtheme` ordinal 135, `SetPreferredAppMode(AllowDark)`.
/// Standard menus then follow the same setting as [`system_color_scheme`].
fn allow_dark_menus() {
    unsafe {
        if let Some(set_mode) = uxtheme_ordinal(135) {
            let set_mode: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(set_mode);
            set_mode(ALLOW_DARK);
        }
    }
}

/// `uxtheme` ordinal 136, `FlushMenuThemes`.
fn flush_menu_themes() {
    unsafe {
        if let Some(flush) = uxtheme_ordinal(136) {
            let flush: unsafe extern "system" fn() = std::mem::transmute(flush);
            flush();
        }
    }
}

unsafe fn uxtheme_ordinal(ordinal: usize) -> Option<unsafe extern "system" fn() -> isize> {
    let module = LoadLibraryA(windows::core::s!("uxtheme.dll")).ok()?;
    GetProcAddress(module, PCSTR(ordinal as *const u8))
}
