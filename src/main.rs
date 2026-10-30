//! EvenTide window. Dim lenses and the scheduled color ramp are separate controls.

#![windows_subsystem = "windows"]

use std::cell::{Cell, RefCell};
use std::ffi::CString;

use eventide::{
    canonical_city, city_escape, city_menu_origin, city_opening_release, city_press, city_release,
    city_row_index, city_wheel, config_dir, day_kelvin_from_drag, dialog_origin, dim_control_pair,
    dim_override_from_saved, dim_override_to_saved, dim_schedule_from_legacy, dimming_enabled,
    ensure_config_files, format_clock, gdi_index_from_device_name, identity_ramp, lens_alpha,
    lens_enabled, level_from_saved_index, location_text, monitor_band, monitor_kelvin,
    monitor_path, monitors_to_json, nearest_slider_thumb, night_kelvin_from_drag, parse_clock,
    parse_monitor_file, parse_preset_file, place_tiles, preset_label, preset_menu_names,
    preset_path, presets_to_json, schedule, scheduled_ramp, scroll_city_wheel, select_city,
    settings_number, sky_label, slider_track_edges, status_line, step_wake_minute, thumb_x,
    tile_caption, tile_controls_are_labeled, tile_kelvin, tile_label, tile_zones,
    toggled_dim_override, zone_at, CityEffect, ColorControlSize, CustomPreset, DimMoment,
    DimOverride, DimSchedule, LensSettings, MonitorInput, MonitorSet, PresetChoice, Schedule,
    ScreenRect, SliderThumb, SunStatus, TilePart, UtcTime, APP_NAME, BRIGHTNESS_LABELS,
    BUILTIN_PRESETS, CITIES, CITY_MENU_ROWS, CLIENT_HEIGHT, CLIENT_WIDTH, DEFAULT_REVERT_MINUTE,
    DEFAULT_TRANSITION_MINUTE, EARLIER_REG_KEY, LEGACY_REG_KEY, LENS_EXTENDED_STYLE, LENS_FILL,
    LENS_RAISE_FLAGS, PRIOR_REG_KEY, REG_KEY, SLIDER_MAX_KELVIN,
};
use windows::core::{s, PCSTR, PCWSTR};
use windows::Win32::Devices::Display::GAMMARAMP;
use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_MODE_INFO,
    DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::Foundation::{
    COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WIN32_ERROR, WPARAM,
};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED};
use windows::Win32::Graphics::Gdi::{
    AlphaBlend, BeginPaint, BitBlt, ClientToScreen, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateDCW, CreateDIBSection, CreateFontIndirectA, CreateFontIndirectW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, Ellipse, EndPaint, EnumDisplayDevicesW,
    EnumDisplayMonitors, FillRect, GetDC, GetMonitorInfoW, GetStockObject, GetSysColor,
    GetTextExtentPoint32W, IntersectClipRect, InvalidateRect, LineTo, MoveToEx, Polygon, Rectangle,
    ReleaseDC, RestoreDC, RoundRect, SaveDC, ScreenToClient, SelectObject, SetBkColor, SetBkMode,
    SetTextColor, TextOutW, UpdateWindow, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, BLENDFUNCTION, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_MENU, COLOR_MENUTEXT,
    DIB_RGB_COLORS, DISPLAY_DEVICEW, DRAW_TEXT_FORMAT, DT_CALCRECT, DT_NOPREFIX, DT_WORDBREAK,
    HBRUSH, HDC, HFONT, HMONITOR, HOLLOW_BRUSH, LOGFONTA, MONITORINFO, MONITORINFOEXW, PAINTSTRUCT,
    PS_SOLID, SRCCOPY, TRANSPARENT, WHITE_BRUSH,
};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExA, RegOpenKeyExA, RegQueryValueExA, RegSetValueExA, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE, REG_SZ,
    REG_VALUE_TYPE,
};
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::System::Time::GetTimeZoneInformation;
use windows::Win32::UI::ColorSystem::SetDeviceGammaRamp;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, IsWindowEnabled, ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_LEAVE,
    TRACKMOUSEEVENT, VK_DOWN, VK_ESCAPE, VK_RETURN, VK_UP,
};
use windows::Win32::UI::Shell::{
    ShellExecuteW, Shell_NotifyIconA, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
    NOTIFYICONDATAA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuA, CallWindowProcW, CreateIconFromResourceEx, CreatePopupMenu, CreateWindowExA,
    DefWindowProcA, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageA, FindWindowA,
    DrawIconEx, GetClientRect, GetCursorPos, GetMessageA, GetSystemMetrics, GetWindowRect,
    GetWindowTextW, IsIconic, IsWindow, IsWindowVisible, LoadCursorW, LoadIconW,
    PostMessageA, PostQuitMessage, RegisterClassExA, SendMessageW, SetCursor, SetForegroundWindow,
    SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW, SetWindowPos, SetWindowTextW,
    ShowWindow, SystemParametersInfoW, TrackPopupMenu, TranslateMessage, CS_DBLCLKS,
    CW_USEDEFAULT, DI_NORMAL, GWLP_WNDPROC, HICON, HMENU, HTCLIENT, HWND_TOPMOST, ICON_BIG,
    ICON_SMALL, IDC_ARROW, IDC_HAND, IDI_APPLICATION, IMAGE_FLAGS, LWA_ALPHA, MENU_ITEM_FLAGS,
    MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, NONCLIENTMETRICSW, SET_WINDOW_POS_FLAGS,
    SM_CXICON, SM_CXSMICON, SM_CXVIRTUALSCREEN, SM_CYICON, SM_CYMENU, SM_CYSMICON,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SPI_GETNONCLIENTMETRICS,
    SPI_GETWHEELSCROLLLINES, SWP_NOACTIVATE, SWP_NOREDRAW, SW_HIDE, SW_RESTORE, SW_SHOW,
    SW_SHOWNORMAL, TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATE,
    WM_CAPTURECHANGED, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLOREDIT, WM_DESTROY, WM_ERASEBKGND,
    WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_NCLBUTTONDOWN, WM_NULL, WM_PAINT, WM_SETCURSOR, WM_SETFONT, WM_SETICON, WM_TIMER,
    WNDCLASSEXA, WNDPROC, WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_VISIBLE,
};

const CLASS_MAIN: &str = "EvenTide";
const CLASS_OWNER: &str = "EvenTideOwner";
const CLASS_LENS: &str = "EvenTideOverlay";
const CLASS_SAVE: &str = "EvenTideSave";
const CLASS_TIME: &str = "EvenTideTime";
const CLASS_CITY: &str = "EvenTideCity";
const CLASS_ABOUT: &str = "EvenTideAbout";
const TRAY_MSG: u32 = 0x405;
const TIMER_PAINT: usize = 1;
const TIMER_RAISE: usize = 2;
const CMD_EXIT: u32 = 3;
const CMD_DIM: u32 = 4;
const CMD_DEFAULT: u32 = 5;
const CMD_ABOUT: u32 = 6;
const CMD_BRIGHTNESS: u32 = 100;
const CMD_PRESET: u32 = 200;
const CMD_CITY: u32 = 400;
const SAVE_NAME_NEW: u32 = 1110;
const SAVE_NAME_PRESET: u32 = 1111;
const TIME_TRANSITION: u32 = 1120;
const TIME_REVERT: u32 = 1121;
const TIME_TRANSITION_EARLIER: u32 = 1122;
const TIME_TRANSITION_LATER: u32 = 1123;
const TIME_REVERT_EARLIER: u32 = 1124;
const TIME_REVERT_LATER: u32 = 1125;
const TIME_OK: u32 = 1126;
const TIME_CANCEL: u32 = 1127;
const TIME_SCHEDULE: u32 = 1128;

const BG: u32 = 0x00F3F1EF;
const INK: u32 = 0x00383332;
const BLUE: u32 = 0x00C49172;
const BLUE_DEEP: u32 = 0x0094624D;
const MUTED: u32 = 0x00988880;
const TILE: u32 = 0x00FFFFFF;
const TILE_DIM: u32 = 0x00E4DDD6;
const LINE: u32 = 0x00D5D0CB;
const TRACK: u32 = 0x00800000;
const NAVY: u32 = 0x00800000;
const NIGHT_HANDLE: u32 = 0x00288CF2;
const DAY_HANDLE: u32 = 0x00F0C67E;

const ICON_16: &[u8] = include_bytes!("../assets/icon-16.bgra");
const ICON_24: &[u8] = include_bytes!("../assets/icon-24.bgra");
const ICON_48: &[u8] = include_bytes!("../assets/icon-48.bgra");
const ICON_72: &[u8] = include_bytes!("../assets/icon-72.bgra");
const ICON_256: &[u8] = include_bytes!("../assets/icon-256.bgra");
const ABOUT_WIDTH: i32 = 440;
const ABOUT_ICON: i32 = 48;
const ABOUT_MARGIN: i32 = 24;
const ABOUT_DESCRIPTION: &str =
    "EvenTide dims the monitors you choose and shifts their color from daylight to nighttime.";
const ABOUT_COPYRIGHT: &str = "\u{00A9} 2026 clintljohnson";
const ABOUT_VERSION: &str = concat!("Version ", env!("CARGO_PKG_VERSION"));
const ABOUT_LINKS: &[(&str, &str)] = &[
    ("MIT License", "https://opensource.org/license/mit"),
    (
        "github.com/clintljohnson/EvenTide",
        "https://github.com/clintljohnson/EvenTide",
    ),
];
/// Corner ellipse for the temperature bubble. Small enough that the button stays mostly rectangular.
const BUBBLE_CORNER: i32 = 8;

struct LiveMonitor {
    number: Option<u32>,
    device_key: String,
    rect: RECT,
    lens: HWND,
}

struct SaveDialog {
    hwnd: HWND,
    edit: HWND,
    /// `None` is the New row. `Some` is an index into the saved presets.
    selected: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TimeField {
    Transition,
    Revert,
}

struct TimeDialog {
    hwnd: HWND,
    draft: DimSchedule,
}

struct App {
    hwnd: HWND,
    owner: HWND,
    monitors: Vec<LiveMonitor>,
    settings: LensSettings,
    dim: MonitorSet,
    color: MonitorSet,
    dim_schedule: DimSchedule,
    dim_override: DimOverride,
    last_dim_gate: Option<bool>,
    last_lens_alpha: Option<u8>,
    customs: Vec<CustomPreset>,
    /// Menu choice. Kept while its three kelvin values still match the live sliders.
    preset_choice: Option<PresetChoice>,
    presets_writable: bool,
    monitors_writable: bool,
    preset_warning: bool,
    monitor_warning: bool,
    dragging: bool,
    slider_thumb: SliderThumb,
    last_ramps: Vec<(String, [[u16; 256]; 3])>,
    painted_minute: u32,
    painted_label: String,
    painted_kelvin: i32,
    last_sun_up: Option<bool>,
    started: bool,
    font_header: HFONT,
    font_body: HFONT,
    font_wake: HFONT,
    font_wake_edit: HFONT,
    font_kelvin: HFONT,
    font_small: HFONT,
    icon_big: HICON,
    icon_small: HICON,
    /// Temperature button under the pointer. Empty until the pointer is over one.
    color_hover: Option<ScreenRect>,
    tracking_mouse: bool,
    save: Option<SaveDialog>,
    time: Option<TimeDialog>,
    /// Clock editor inside the time dialog. Empty until a time is clicked.
    time_edit: HWND,
    time_field: Option<TimeField>,
    location_hit: RECT,
    about_hwnd: HWND,
    about_icon: HICON,
    about_modal: bool,
}

thread_local! {
    static APP: RefCell<App> = RefCell::new(App {
        hwnd: HWND::default(),
        owner: HWND::default(),
        monitors: Vec::new(),
        settings: LensSettings::seed(),
        dim: MonitorSet::All,
        color: MonitorSet::All,
        dim_schedule: DimSchedule::solar(),
        dim_override: DimOverride::Schedule,
        last_dim_gate: None,
        last_lens_alpha: None,
        customs: Vec::new(),
        preset_choice: None,
        presets_writable: true,
        monitors_writable: true,
        preset_warning: false,
        monitor_warning: false,
        dragging: false,
        slider_thumb: SliderThumb::Day,
        last_ramps: Vec::new(),
        painted_minute: u32::MAX,
        painted_label: String::new(),
        painted_kelvin: -1,
        last_sun_up: None,
        started: false,
        font_header: HFONT::default(),
        font_body: HFONT::default(),
        font_wake: HFONT::default(),
        font_wake_edit: HFONT::default(),
        font_kelvin: HFONT::default(),
        font_small: HFONT::default(),
        icon_big: HICON::default(),
        icon_small: HICON::default(),
        color_hover: None,
        tracking_mouse: false,
        save: None,
        time: None,
        time_edit: HWND::default(),
        time_field: None,
        location_hit: RECT::default(),
        about_hwnd: HWND::default(),
        about_icon: HICON::default(),
        about_modal: false,
    });
}

thread_local! {
    static WAKE_OLD_PROC: Cell<isize> = const { Cell::new(0) };
}

thread_local! {
    static CITY_HWND: Cell<isize> = const { Cell::new(0) };
    static CITY_FONT: Cell<isize> = const { Cell::new(0) };
    static CITY_SCROLL: Cell<i32> = const { Cell::new(0) };
    static CITY_HOT: Cell<i32> = const { Cell::new(-1) };
    static CITY_OPEN: Cell<bool> = const { Cell::new(false) };
    static CITY_CHOICE: Cell<i32> = const { Cell::new(-1) };
    static CITY_WHEEL: Cell<i32> = const { Cell::new(0) };
    static CITY_ARMED: Cell<bool> = const { Cell::new(false) };
}

fn main() {
    let layout = std::env::args().any(|arg| arg == "--layout");
    unsafe {
        if layout {
            let _ = AllocConsole();
            let text = layout_report();
            println!("{text}");
            if let Ok(()) = ensure_config_files() {
                if let Some(dir) = config_dir() {
                    let _ = std::fs::write(dir.join("layout-check.txt"), &text);
                }
            }
            return;
        }
        run();
    }
}

unsafe fn run() {
    let class = leak_cstr(CLASS_MAIN);
    if let Ok(existing) = FindWindowA(class, None) {
        if !existing.0.is_null() {
            let _ = ShowWindow(existing, SW_RESTORE);
            let _ = SetForegroundWindow(existing);
            return;
        }
    }
    load_state();
    let instance = GetModuleHandleA(None).unwrap_or_default();
    let icon_big = shell_icon(GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON));
    let icon_small = shell_icon(GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON));
    APP.with(|app| {
        let mut app = app.borrow_mut();
        if let Some(icon) = icon_big {
            app.icon_big = icon;
        }
        if let Some(icon) = icon_small {
            app.icon_small = icon;
        }
    });
    if register_class(
        class,
        wnd_proc,
        instance,
        icon_big.unwrap_or_default(),
        icon_small.unwrap_or_default(),
    )
    .is_err()
    {
        return;
    }
    let _ = register_class(
        leak_cstr(CLASS_OWNER),
        owner_proc,
        instance,
        HICON::default(),
        HICON::default(),
    );
    let _ = register_class(
        leak_cstr(CLASS_LENS),
        lens_proc,
        instance,
        HICON::default(),
        HICON::default(),
    );
    let _ = register_class(
        leak_cstr(CLASS_SAVE),
        save_proc,
        instance,
        icon_big.unwrap_or_default(),
        icon_small.unwrap_or_default(),
    );
    let _ = register_class(
        leak_cstr(CLASS_TIME),
        time_proc,
        instance,
        icon_big.unwrap_or_default(),
        icon_small.unwrap_or_default(),
    );
    let _ = register_class(
        leak_cstr(CLASS_CITY),
        city_proc,
        instance,
        HICON::default(),
        HICON::default(),
    );
    let _ = register_class(
        leak_cstr(CLASS_ABOUT),
        about_proc,
        instance,
        icon_big.unwrap_or_default(),
        icon_small.unwrap_or_default(),
    );

    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: CLIENT_WIDTH,
        bottom: CLIENT_HEIGHT,
    };
    let _ = windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
    let hwnd = CreateWindowExA(
        WINDOW_EX_STYLE::default(),
        class,
        leak_cstr(APP_NAME),
        style | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
        None,
        None,
        Some(instance.into()),
        None,
    );
    let Ok(hwnd) = hwnd else { return };
    if let Some(icon) = icon_big {
        let _ = SendMessageW(
            hwnd,
            WM_SETICON,
            Some(WPARAM(ICON_BIG as usize)),
            Some(LPARAM(icon.0 as isize)),
        );
    }
    if let Some(icon) = icon_small {
        let _ = SendMessageW(
            hwnd,
            WM_SETICON,
            Some(WPARAM(ICON_SMALL as usize)),
            Some(LPARAM(icon.0 as isize)),
        );
    }
    let owner = CreateWindowExA(
        WINDOW_EX_STYLE(0x80),
        leak_cstr(CLASS_OWNER),
        leak_cstr(APP_NAME),
        WINDOW_STYLE(0),
        0,
        0,
        0,
        0,
        Some(hwnd),
        None,
        Some(instance.into()),
        None,
    )
    .unwrap_or_default();
    APP.with(|app| {
        let mut app = app.borrow_mut();
        app.hwnd = hwnd;
        app.owner = owner;
        app.started = true;
    });
    refresh_monitors();
    sync_lenses();
    apply_gamma();
    let _ = ShowWindow(hwnd, SW_SHOW);
    let mut msg = MSG::default();
    while GetMessageA(&mut msg, None, 0, 0).into() {
        let _ = TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
    save_state();
}

unsafe fn register_class(
    name: PCSTR,
    proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    instance: windows::Win32::Foundation::HMODULE,
    icon: HICON,
    icon_small: HICON,
) -> windows::core::Result<u16> {
    let cursor = LoadCursorW(None, IDC_ARROW)?;
    let fallback = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
    let class = WNDCLASSEXA {
        cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
        style: CS_DBLCLKS,
        lpfnWndProc: Some(proc),
        hInstance: instance.into(),
        hIcon: if icon.0.is_null() { fallback } else { icon },
        hCursor: cursor,
        hbrBackground: HBRUSH::default(),
        lpszClassName: name,
        hIconSm: if icon_small.0.is_null() {
            fallback
        } else {
            icon_small
        },
        ..Default::default()
    };
    let atom = RegisterClassExA(&class);
    if atom == 0 {
        Err(windows::core::Error::from_win32())
    } else {
        Ok(atom)
    }
}

fn leak_cstr(text: &str) -> PCSTR {
    let cstring = CString::new(text).unwrap_or_else(|_| CString::new(APP_NAME).unwrap());
    PCSTR(Box::leak(Box::new(cstring)).as_ptr().cast())
}

unsafe extern "system" fn owner_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcA(hwnd, msg, wparam, lparam)
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            add_tray(hwnd);
            let _ = SetTimer(Some(hwnd), TIMER_PAINT, 1000, None);
            let _ = SetTimer(Some(hwnd), TIMER_RAISE, 100, None);
            APP.with(|app| {
                let mut app = app.borrow_mut();
                app.font_header = make_font(-20, 400);
                app.font_body = make_font(-16, 400);
                app.font_wake = make_font(-28, 400);
                app.font_wake_edit = make_font(-18, 400);
                app.font_kelvin = make_font(-26, 400);
                app.font_small = make_font(-14, 400);
            });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_CTLCOLOREDIT => {
            let dc = HDC(wparam.0 as *mut core::ffi::c_void);
            let _ = SetTextColor(dc, COLORREF(BLUE_DEEP));
            let _ = SetBkColor(dc, COLORREF(0x00FFFFFF));
            LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_RAISE {
                raise_lenses();
            } else if wparam.0 == TIMER_PAINT {
                let was_sun = APP.with(|app| app.borrow().last_sun_up);
                poll_dim_clock();
                let planned = current_schedule();
                let sun_up = planned.status == SunStatus::Up;
                let moment = dim_moment(&planned, &local_time());
                let (gate, alpha) = APP.with(|app| {
                    let app = app.borrow();
                    let level = level_from_saved_index(app.settings.brightness_index as i32);
                    (
                        dimming_enabled(app.dim_schedule, moment, app.dim_override),
                        lens_alpha(app.dim_schedule, moment, app.dim_override, level),
                    )
                });
                let gate_changed = APP.with(|app| {
                    let mut app = app.borrow_mut();
                    let changed = app.last_dim_gate != Some(gate);
                    app.last_dim_gate = Some(gate);
                    changed
                });
                let alpha_changed = APP.with(|app| app.borrow().last_lens_alpha != Some(alpha));
                if gate_changed || alpha_changed {
                    sync_lenses();
                }
                let gamma_changed = apply_gamma();
                let (minute, label, kelvin) = visible_key();
                let changed = APP.with(|app| {
                    let mut app = app.borrow_mut();
                    let changed = app.painted_minute != minute
                        || app.painted_label != label
                        || app.painted_kelvin != kelvin;
                    app.painted_minute = minute;
                    app.painted_label = label;
                    app.painted_kelvin = kelvin;
                    changed
                });
                if changed || gamma_changed || gate_changed || was_sun != Some(sun_up) {
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                refresh_monitors_if_changed();
            }
            LRESULT(0)
        }
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            on_click(hwnd, lparam_point(lparam));
            LRESULT(0)
        }
        WM_NCLBUTTONDOWN => {
            finish_time_edit(true, true);
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
        WM_ACTIVATE => {
            if (wparam.0 & 0xFFFF) == 0 {
                finish_time_edit(true, true);
            }
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
        WM_MOUSEMOVE => {
            update_color_hover(hwnd, lparam_point(lparam));
            if drag_slider(lparam_point(lparam)) {
                apply_gamma();
                sync_lenses();
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            let hovered = APP.with(|app| {
                let mut app = app.borrow_mut();
                app.tracking_mouse = false;
                app.color_hover.take().is_some()
            });
            if hovered {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_SETCURSOR => {
            if color_button_cursor() {
                LRESULT(1)
            } else {
                DefWindowProcA(hwnd, msg, wparam, lparam)
            }
        }
        WM_LBUTTONUP => {
            let was_dragging = APP.with(|app| app.borrow().dragging);
            end_slider_drag(hwnd);
            if was_dragging {
                let _ = ReleaseCapture();
            }
            LRESULT(0)
        }
        WM_CAPTURECHANGED => {
            end_slider_drag(hwnd);
            LRESULT(0)
        }
        WM_COMMAND => {
            on_command(hwnd, (wparam.0 & 0xFFFF) as u32);
            LRESULT(0)
        }
        msg if msg == TRAY_MSG => {
            let event = (lparam.0 & 0xFFFF) as u32;
            if event == 0x0201 {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
            } else if event == 0x0205 {
                show_tray_menu(hwnd);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            finish_time_edit(true, true);
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        WM_DESTROY => {
            let about = APP.with(|app| app.borrow().about_hwnd);
            if !about.0.is_null() && IsWindow(Some(about)).as_bool() {
                let _ = DestroyWindow(about);
            }
            restore_identity_gamma();
            destroy_lenses();
            remove_tray(hwnd);
            APP.with(|app| {
                let app = app.borrow();
                for font in [
                    app.font_header,
                    app.font_body,
                    app.font_wake,
                    app.font_wake_edit,
                    app.font_kelvin,
                    app.font_small,
                ] {
                    if !font.is_invalid() {
                        let _ = DeleteObject(font.into());
                    }
                }
                if !app.icon_big.0.is_null() {
                    let _ = DestroyIcon(app.icon_big);
                }
                if !app.icon_small.0.is_null() && app.icon_small != app.icon_big {
                    let _ = DestroyIcon(app.icon_small);
                }
            });
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

fn visible_key() -> (u32, String, i32) {
    let planned = current_schedule();
    let (label, minute) = APP.with(|app| {
        let app = app.borrow();
        (
            current_preset_label(&app),
            local_time().minute,
        )
    });
    (minute, label, planned.kelvin)
}

fn on_click(hwnd: HWND, point: POINT) {
    if point_in(point, gear_rect()) {
        open_time(hwnd);
        return;
    }
    if point_in(point, brightness_rect()) {
        unsafe {
            show_brightness_menu(hwnd);
        }
        return;
    }
    if point_in(point, save_rect()) {
        open_save(hwnd);
        return;
    }
    if point_in(point, dimming_rect()) {
        toggle_dim_button(hwnd);
        return;
    }
    if point_in(point, preset_rect()) {
        unsafe {
            show_preset_menu(hwnd);
        }
        return;
    }
    let location = APP.with(|app| app.borrow().location_hit);
    if point_in(point, location) {
        unsafe {
            show_city_menu(hwnd);
        }
        return;
    }
    if point_in(point, slider_hit_rect()) {
        let (day, night) = APP.with(|app| {
            let app = app.borrow();
            (app.settings.day_kelvin, app.settings.night_kelvin)
        });
        let track = slider_track();
        let thumb = nearest_slider_thumb(
            point.x,
            thumb_x(track.left, track.right, day),
            thumb_x(track.left, track.right, night),
        );
        APP.with(|app| {
            let mut app = app.borrow_mut();
            app.dragging = true;
            app.slider_thumb = thumb;
        });
        unsafe {
            let _ = SetCapture(hwnd);
            let _ = drag_slider(point);
            apply_gamma();
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        return;
    }
    toggle_tile(hwnd, point);
}

fn toggle_tile(hwnd: HWND, point: POINT) {
    let tiles = current_tiles();
    let labeled = tile_controls_are_labeled(tiles.len());
    let planned = current_schedule();
    let active = planned.kelvin;
    let (day, night, color) = APP.with(|app| {
        let app = app.borrow();
        (
            app.settings.day_kelvin,
            app.settings.night_kelvin,
            app.color.clone(),
        )
    });
    let sun_up = planned.status == SunStatus::Up;
    let Some(hit) = (unsafe {
        let dc = GetDC(Some(hwnd));
        let hit = tiles.iter().find_map(|tile| {
            let on = color_enabled(tile.number, &color);
            let kelvin = tile_kelvin(on, sun_up, active, day, night);
            let zones = tile_layout(dc, tile, labeled, tiles.len(), kelvin);
            zone_at(&zones, point.x, point.y).map(|part| (tile.number, part))
        });
        let _ = ReleaseDC(Some(hwnd), dc);
        hit
    }) else {
        return;
    };
    let Some(number) = hit.0 else { return };
    let invoke_dim = hit.1 == TilePart::Dim && {
        let moment = dim_moment(&planned, &local_time());
        APP.with(|app| {
            let app = app.borrow();
            !app.dim_schedule.dim_with_schedule
                && !dimming_enabled(app.dim_schedule, moment, app.dim_override)
        })
    };
    APP.with(|app| {
        let attached = attached_numbers(&app.borrow().monitors);
        let mut app = app.borrow_mut();
        match hit.1 {
            TilePart::Dim => {
                app.dim = app.dim.toggle(number, &attached);
                if invoke_dim {
                    if !app.dim.contains(number) {
                        app.dim = app.dim.toggle(number, &attached);
                    }
                    app.dim_override = DimOverride::On;
                }
            }
            TilePart::Color => app.color = app.color.toggle(number, &attached),
        }
    });
    unsafe {
        sync_lenses();
        apply_gamma();
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

fn end_slider_drag(hwnd: HWND) {
    let was_dragging = APP.with(|app| {
        let mut app = app.borrow_mut();
        let was_dragging = app.dragging;
        app.dragging = false;
        was_dragging
    });
    if !was_dragging {
        return;
    }
    unsafe {
        apply_gamma();
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

fn drag_slider(point: POINT) -> bool {
    let (dragging, thumb) = APP.with(|app| {
        let app = app.borrow();
        (app.dragging, app.slider_thumb)
    });
    if !dragging {
        return false;
    }
    let track = slider_track();
    APP.with(|app| {
        let mut app = app.borrow_mut();
        match thumb {
            SliderThumb::Day => {
                let before = app.settings.day_kelvin;
                app.settings.day_kelvin = day_kelvin_from_drag(
                    point.x,
                    track.left,
                    track.right,
                    app.settings.night_kelvin,
                );
                before != app.settings.day_kelvin
            }
            SliderThumb::Night => {
                let before = app.settings.night_kelvin;
                app.settings.night_kelvin = night_kelvin_from_drag(
                    point.x,
                    track.left,
                    track.right,
                    app.settings.day_kelvin,
                );
                before != app.settings.night_kelvin
            }
        }
    })
}

fn on_command(hwnd: HWND, id: u32) {
    if id == CMD_EXIT {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
        return;
    }
    if id == CMD_ABOUT {
        unsafe {
            show_about(hwnd);
        }
        return;
    }
    if id == CMD_DIM {
        toggle_dim_button(hwnd);
        return;
    }
    if id == CMD_DEFAULT {
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let mut settings = app.settings.clone();
            let mut dim = app.dim.clone();
            let mut color = app.color.clone();
            eventide::restore_first_run(&mut settings, &mut dim, &mut color);
            app.settings = settings;
            app.dim = dim;
            app.color = color;
            app.dim_override = DimOverride::Schedule;
            app.preset_choice = None;
        });
        unsafe {
            sync_lenses();
            apply_gamma();
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        return;
    }
    if (CMD_BRIGHTNESS..CMD_BRIGHTNESS + 9).contains(&id) {
        APP.with(|app| app.borrow_mut().settings.brightness_index = id - CMD_BRIGHTNESS);
        unsafe {
            sync_lenses();
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        return;
    }
    if (CMD_PRESET..CMD_CITY).contains(&id) {
        let index = (id - CMD_PRESET) as usize;
        if index < BUILTIN_PRESETS.len() {
            let preset = BUILTIN_PRESETS[index];
            APP.with(|app| {
                let mut app = app.borrow_mut();
                app.settings.day_kelvin = preset.day;
                app.settings.night_kelvin = preset.night;
                app.settings.late_kelvin = preset.late;
                app.preset_choice = Some(PresetChoice::Builtin(preset.name));
            });
        } else {
            let custom_index = index - BUILTIN_PRESETS.len();
            let preset = APP.with(|app| app.borrow().customs.get(custom_index).cloned());
            if let Some(preset) = preset {
                APP.with(|app| apply_preset(&mut app.borrow_mut(), &preset));
                unsafe {
                    sync_lenses();
                }
            }
        }
        unsafe {
            apply_gamma();
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        return;
    }
    if (CMD_CITY..CMD_CITY + CITIES.len() as u32).contains(&id) {
        let city = CITIES[(id - CMD_CITY) as usize];
        APP.with(|app| select_city(&mut app.borrow_mut().settings, &city));
        unsafe {
            apply_gamma();
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
}

fn current_tiles() -> Vec<eventide::Tile> {
    let inputs = APP.with(|app| {
        app.borrow()
            .monitors
            .iter()
            .map(|monitor| MonitorInput {
                number: monitor.number,
                bounds: ScreenRect {
                    left: monitor.rect.left,
                    top: monitor.rect.top,
                    right: monitor.rect.right,
                    bottom: monitor.rect.bottom,
                },
            })
            .collect::<Vec<_>>()
    });
    place_tiles(&inputs, monitor_band())
}

fn attached_numbers(monitors: &[LiveMonitor]) -> Vec<u32> {
    let mut numbers: Vec<u32> = monitors
        .iter()
        .filter_map(|monitor| monitor.number)
        .collect();
    numbers.sort_unstable();
    numbers.dedup();
    numbers
}

unsafe fn paint(hwnd: HWND) {
    let mut paint = PAINTSTRUCT::default();
    let window_dc = BeginPaint(hwnd, &mut paint);
    if window_dc.is_invalid() {
        return;
    }
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);
    let buffer = CreateCompatibleDC(Some(window_dc));
    let bitmap = CreateCompatibleBitmap(window_dc, client.right.max(1), client.bottom.max(1));
    let previous = SelectObject(buffer, bitmap.into());
    fill(buffer, client, BG);
    let _ = SetBkMode(buffer, TRANSPARENT);
    let snapshot = APP.with(|app| {
        let app = app.borrow();
        (
            app.settings.clone(),
            app.customs.clone(),
            app.dim.clone(),
            app.color.clone(),
            app.dim_schedule,
            app.dim_override,
            app.preset_warning,
            app.monitor_warning,
            [
                app.font_header,
                app.font_body,
                app.font_wake,
                app.font_small,
            ],
            app.preset_choice.clone(),
        )
    });
    let (
        settings,
        customs,
        dim,
        color,
        dim_schedule,
        mode,
        preset_warning,
        monitor_warning,
        fonts,
        preset_choice,
    ) = snapshot;
    let planned = schedule(
        local_time(),
        settings.color_settings(),
        utc_offset_minutes(),
    );
    let tiles = current_tiles();
    let labeled = tile_controls_are_labeled(tiles.len());
    draw_command_button(buffer, fonts[1], brightness_rect(), "Brightness");
    draw_gear_button(buffer, gear_rect());
    draw_command_button(buffer, fonts[1], save_rect(), "Save preset");
    let dim_on = dimming_enabled(dim_schedule, dim_moment(&planned, &local_time()), mode);
    let dim_bounds = live_dim_rect();
    draw_pill(buffer, fonts[1], dim_bounds, dimming_label(dim_on), dim_on);
    let header = sky_label(planned.altitude_deg, planned.solar_hours);
    let header_size = text_size(buffer, fonts[0], header);
    let header_row = header_row();
    text(
        buffer,
        fonts[0],
        INK,
        right_edge() - header_size.cx,
        header_row.top + (header_row.bottom - header_row.top - header_size.cy) / 2,
        header,
    );
    let active = planned.kelvin;
    let hover = APP.with(|app| app.borrow().color_hover);
    for tile in &tiles {
        draw_tile(
            buffer,
            tile,
            labeled,
            tiles.len(),
            &dim,
            &color,
            fonts[1],
            fonts[0],
            fonts[3],
            tile_kelvin(
                color_enabled(tile.number, &color),
                planned.status == SunStatus::Up,
                active,
                settings.day_kelvin,
                settings.night_kelvin,
            ),
            hover,
        );
    }
    let name = preset_label(
        settings.day_kelvin,
        settings.night_kelvin,
        settings.late_kelvin,
        &customs,
        preset_choice.as_ref(),
    );
    draw_preset(buffer, fonts[1], &name);
    draw_curve(buffer, planned.solar_hours, planned.status == SunStatus::Up);
    draw_slider(buffer, settings.day_kelvin, settings.night_kelvin);
    let kelvin_font = APP.with(|app| app.borrow().font_kelvin);
    let day_label = format!("{}K", settings.day_kelvin.min(SLIDER_MAX_KELVIN).max(0));
    let night_label = format!("{}K", settings.night_kelvin.min(SLIDER_MAX_KELVIN).max(0));
    let night_size = text_size(buffer, kelvin_font, &night_label);
    let (_, day_width) = kelvin_label_widths();
    let track = slider_track();
    let track_mid = (track.top + track.bottom) / 2;
    text(
        buffer,
        kelvin_font,
        INK,
        24,
        track_mid - night_size.cy / 2,
        &night_label,
    );
    text(
        buffer,
        kelvin_font,
        INK,
        right_edge() - day_width,
        track_mid - night_size.cy / 2,
        &day_label,
    );
    let bubble = APP.with(|app| {
        let app = app.borrow();
        if !app.dragging {
            return None;
        }
        let kelvin = match app.slider_thumb {
            SliderThumb::Day => app.settings.day_kelvin,
            SliderThumb::Night => app.settings.night_kelvin,
        };
        Some((app.slider_thumb, kelvin))
    });
    if let Some((thumb, kelvin)) = bubble {
        draw_temperature_bubble(buffer, fonts[1], thumb, kelvin);
    }
    let sentence = status_line(&planned);
    let sentence_w = text_width(buffer, fonts[1], &sentence);
    let sentence_x = track.left + (track.right - track.left - sentence_w) / 2;
    text(
        buffer,
        fonts[1],
        NAVY,
        sentence_x,
        track.bottom + 18,
        &sentence,
    );
    let degrees = degrees_text(settings.latitude_hundredths, settings.longitude_hundredths);
    let degrees_size = text_size(buffer, fonts[3], &degrees);
    let city_size = text_size(buffer, fonts[1], &settings.city);
    let right = CLIENT_WIDTH - 24;
    let degrees_x = right - degrees_size.cx;
    let city_x = right - city_size.cx;
    text(buffer, fonts[3], MUTED, degrees_x, 300, &degrees);
    text(buffer, fonts[1], INK, city_x, 320, &settings.city);
    let text_bottom = 320 + city_size.cy.max(16);
    APP.with(|app| {
        app.borrow_mut().location_hit = RECT {
            left: degrees_x.min(city_x) - 16,
            top: 300 - 16,
            right: right + 16,
            bottom: text_bottom + 16,
        };
    });
    if preset_warning {
        text(
            buffer,
            fonts[3],
            BLUE_DEEP,
            24,
            dim_bounds.top - 20,
            "Preset file was not read.",
        );
    } else if monitor_warning {
        text(
            buffer,
            fonts[3],
            BLUE_DEEP,
            24,
            dim_bounds.top - 20,
            "Monitor list was not read.",
        );
    }
    let _ = BitBlt(
        window_dc,
        0,
        0,
        client.right,
        client.bottom,
        Some(buffer),
        0,
        0,
        SRCCOPY,
    );
    let _ = SelectObject(buffer, previous);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(buffer);
    let _ = EndPaint(hwnd, &paint);
}

unsafe fn draw_tile(
    dc: HDC,
    tile: &eventide::Tile,
    labeled: bool,
    count: usize,
    dim: &MonitorSet,
    color: &MonitorSet,
    temp_font: HFONT,
    number_font: HFONT,
    small_font: HFONT,
    kelvin: i32,
    hover: Option<ScreenRect>,
) {
    let on_dim = match tile.number {
        Some(number) => dim.contains(number),
        None => matches!(dim, MonitorSet::All),
    };
    let on_color = color_enabled(tile.number, color);
    let bounds = rect_from(tile.bounds);
    fill(dc, bounds, if on_dim { TILE_DIM } else { TILE });
    stroke_rect(dc, bounds, if on_dim { BLUE } else { LINE });
    let zones = tile_layout(dc, tile, labeled, count, kelvin);
    let label = tile_caption(tile.number, count);
    let used_font = if tile.bounds.height() >= 90 {
        number_font
    } else {
        small_font
    };
    let width = text_width(dc, used_font, &label);
    let number_x = tile.bounds.left + (tile.bounds.width() - width) / 2;
    text(dc, used_font, INK, number_x, zones.number_y, &label);
    let hot = hover.is_some_and(|hit| hit == zones.color);
    if labeled {
        draw_temperature_button(
            dc,
            temp_font,
            zones.color,
            &format!("{kelvin}K"),
            on_color,
            hot,
        );
    } else {
        let mark = rect_from(zones.color);
        if on_color {
            fill_round(dc, mark, INK);
        }
        if hot {
            stroke_round(dc, mark, if on_color { 0x00FFFFFF } else { BLUE_DEEP });
        }
    }
}

unsafe fn tile_layout(
    dc: HDC,
    tile: &eventide::Tile,
    labeled: bool,
    count: usize,
    kelvin: i32,
) -> eventide::TileZones {
    let number_font = if tile.bounds.height() >= 90 {
        APP.with(|app| app.borrow().font_header)
    } else {
        APP.with(|app| app.borrow().font_small)
    };
    let number_h = text_size(dc, number_font, &tile_caption(tile.number, count))
        .cy
        .max(1);
    let control = if labeled {
        let temp_font = APP.with(|app| app.borrow().font_body);
        let size = text_size(dc, temp_font, &format!("{kelvin}K"));
        ColorControlSize {
            text_width: size.cx,
            text_height: size.cy,
            icon_width: 0,
            icon_height: 0,
            number_height: number_h,
        }
    } else {
        ColorControlSize {
            text_width: 0,
            text_height: 0,
            icon_width: 0,
            icon_height: 0,
            number_height: number_h,
        }
    };
    tile_zones(tile.bounds, labeled, control)
}

fn color_enabled(number: Option<u32>, color: &MonitorSet) -> bool {
    match number {
        Some(number) => color.contains(number),
        None => matches!(color, MonitorSet::All),
    }
}

fn update_color_hover(hwnd: HWND, point: POINT) {
    let hover = unsafe { color_button_at(hwnd, point) };
    let changed = APP.with(|app| {
        let mut app = app.borrow_mut();
        let changed = app.color_hover != hover;
        app.color_hover = hover;
        changed
    });
    let track = APP.with(|app| {
        let mut app = app.borrow_mut();
        if app.tracking_mouse {
            false
        } else {
            app.tracking_mouse = true;
            true
        }
    });
    if track {
        let mut event = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        };
        if unsafe { TrackMouseEvent(&mut event).is_err() } {
            APP.with(|app| app.borrow_mut().tracking_mouse = false);
        }
    }
    if changed {
        let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
    }
}

unsafe fn color_button_at(hwnd: HWND, point: POINT) -> Option<ScreenRect> {
    let tiles = current_tiles();
    let labeled = tile_controls_are_labeled(tiles.len());
    let planned = current_schedule();
    let active = planned.kelvin;
    let (day, night, color) = APP.with(|app| {
        let app = app.borrow();
        (
            app.settings.day_kelvin,
            app.settings.night_kelvin,
            app.color.clone(),
        )
    });
    let sun_up = planned.status == SunStatus::Up;
    let dc = GetDC(Some(hwnd));
    let found = tiles.iter().find_map(|tile| {
        let kelvin = tile_kelvin(
            color_enabled(tile.number, &color),
            sun_up,
            active,
            day,
            night,
        );
        let zones = tile_layout(dc, tile, labeled, tiles.len(), kelvin);
        if zones.color.contains(point.x, point.y) {
            Some(zones.color)
        } else {
            None
        }
    });
    let _ = ReleaseDC(Some(hwnd), dc);
    found
}

fn color_button_cursor() -> bool {
    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point).is_err() } {
        return false;
    }
    let hwnd = APP.with(|app| app.borrow().hwnd);
    if hwnd.0.is_null() || !unsafe { ScreenToClient(hwnd, &mut point).as_bool() } {
        return false;
    }
    let over = unsafe { color_button_at(hwnd, point) }.is_some();
    if over {
        unsafe {
            SetCursor(Some(LoadCursorW(None, IDC_HAND).unwrap_or_default()));
        }
    }
    over
}

unsafe fn draw_temperature_button(
    dc: HDC,
    font: HFONT,
    zone: ScreenRect,
    label: &str,
    on: bool,
    hovered: bool,
) {
    let rect = rect_from(zone);
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    if on {
        fill_round(dc, rect, INK);
    }
    if hovered {
        stroke_round(dc, rect, if on { 0x00FFFFFF } else { BLUE_DEEP });
    }
    let size = text_size(dc, font, label);
    text(
        dc,
        font,
        if on { 0x00FFFFFF } else { INK },
        rect.left + (rect.right - rect.left - size.cx) / 2,
        rect.top + (rect.bottom - rect.top - size.cy) / 2,
        label,
    );
}

unsafe fn fill_round(dc: HDC, rect: RECT, color: u32) {
    let brush = CreateSolidBrush(COLORREF(color));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(color));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(
        dc,
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
        BUBBLE_CORNER,
        BUBBLE_CORNER,
    );
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}

unsafe fn stroke_round(dc: HDC, rect: RECT, color: u32) {
    let brush = GetStockObject(HOLLOW_BRUSH);
    let pen = CreatePen(PS_SOLID, 1, COLORREF(color));
    let old_brush = SelectObject(dc, brush);
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(
        dc,
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
        BUBBLE_CORNER,
        BUBBLE_CORNER,
    );
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(pen.into());
}

unsafe fn draw_pill(dc: HDC, font: HFONT, zone: ScreenRect, label: &str, on: bool) {
    let rect = rect_from(zone);
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    let brush = CreateSolidBrush(COLORREF(if on { BLUE } else { 0x00FFFFFF }));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(if on { BLUE_DEEP } else { LINE }));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let width = text_width(dc, font, label);
    text(
        dc,
        font,
        if on { 0x00FFFFFF } else { INK },
        rect.left + ((rect.right - rect.left) - width) / 2,
        rect.top + 6,
        label,
    );
}

unsafe fn draw_gear_button(dc: HDC, rect: RECT) {
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(LINE));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 8, 8);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let teeth = 8i32;
    let outer = 10.0f64;
    let root = 7.0f64;
    let step = std::f64::consts::TAU / f64::from(teeth);
    let mut points = Vec::with_capacity((teeth * 4) as usize);
    for index in 0..teeth {
        let base = f64::from(index) * step - std::f64::consts::FRAC_PI_2;
        let inset = step * 0.14;
        let tooth = step * 0.36;
        points.push(gear_point(cx, cy, outer, base + inset));
        points.push(gear_point(cx, cy, outer, base + inset + tooth));
        points.push(gear_point(cx, cy, root, base + inset + tooth + inset));
        points.push(gear_point(cx, cy, root, base + step - inset));
    }
    let brush = CreateSolidBrush(COLORREF(INK));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(INK));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = Polygon(dc, &points);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let hole = 3;
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(0x00FFFFFF));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = Ellipse(dc, cx - hole, cy - hole, cx + hole + 1, cy + hole + 1);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}

fn gear_point(cx: i32, cy: i32, radius: f64, angle: f64) -> POINT {
    POINT {
        x: cx + (radius * angle.cos()).round() as i32,
        y: cy + (radius * angle.sin()).round() as i32,
    }
}

unsafe fn draw_command_button(dc: HDC, font: HFONT, rect: RECT, label: &str) {
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(LINE));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let width = text_width(dc, font, label);
    text(
        dc,
        font,
        INK,
        rect.left + (rect.right - rect.left - width) / 2,
        rect.top + 7,
        label,
    );
}

unsafe fn draw_preset(dc: HDC, font: HFONT, label: &str) {
    let rect = preset_rect();
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(LINE));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let saved = SaveDC(dc);
    let _ = IntersectClipRect(dc, rect.left + 8, rect.top, rect.right - 26, rect.bottom);
    let size = text_size(dc, font, label);
    text(
        dc,
        font,
        INK,
        rect.left + 14,
        rect.top + (rect.bottom - rect.top - size.cy) / 2,
        label,
    );
    if saved != 0 {
        let _ = RestoreDC(dc, saved);
    }
    let pen = CreatePen(PS_SOLID, 1, COLORREF(MUTED));
    let old = SelectObject(dc, pen.into());
    let x = rect.right - 22;
    let y = rect.top + (rect.bottom - rect.top - 4) / 2;
    let _ = MoveToEx(dc, x, y, None);
    let _ = LineTo(dc, x + 4, y + 4);
    let _ = LineTo(dc, x + 8, y);
    let _ = SelectObject(dc, old);
    let _ = DeleteObject(pen.into());
}

unsafe fn draw_slider(dc: HDC, day_kelvin: i32, night_kelvin: i32) {
    let track = slider_track();
    fill(dc, track, TRACK);
    draw_slider_thumb(
        dc,
        thumb_x(track.left, track.right, night_kelvin),
        track,
        NIGHT_HANDLE,
    );
    draw_slider_thumb(
        dc,
        thumb_x(track.left, track.right, day_kelvin),
        track,
        DAY_HANDLE,
    );
}

unsafe fn draw_slider_thumb(dc: HDC, x: i32, track: RECT, color: u32) {
    let brush = CreateSolidBrush(COLORREF(color));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(color));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = Rectangle(dc, x - 3, track.top - 8, x + 3, track.bottom + 9);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}

unsafe fn draw_temperature_bubble(dc: HDC, font: HFONT, thumb: SliderThumb, kelvin: i32) {
    let label = match thumb {
        SliderThumb::Day => "Day Temperature",
        SliderThumb::Night => "Night Temperature",
    };
    let size = text_size(dc, font, label);
    let track = slider_track();
    let anchor = thumb_x(track.left, track.right, kelvin);
    let body_w = size.cx + 28;
    let body_h = size.cy + 16;
    let tip_y = track.top - 10;
    let body_bottom = tip_y - 7;
    let body_top = body_bottom - body_h;
    let mut left = anchor - body_w / 2;
    let mut right = left + body_w;
    if left < 8 {
        right += 8 - left;
        left = 8;
    }
    if right > CLIENT_WIDTH - 8 {
        let shift = right - (CLIENT_WIDTH - 8);
        left -= shift;
        right -= shift;
    }
    let tip_x = anchor.clamp(left + 16, right - 16);
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(NAVY));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, left, body_top, right, body_bottom, body_h, body_h);
    let points = [
        POINT {
            x: tip_x - 8,
            y: body_bottom - 1,
        },
        POINT { x: tip_x, y: tip_y },
        POINT {
            x: tip_x + 8,
            y: body_bottom - 1,
        },
    ];
    let _ = Polygon(dc, &points);
    let _ = MoveToEx(dc, tip_x - 8, body_bottom - 1, None);
    let _ = LineTo(dc, tip_x, tip_y);
    let _ = LineTo(dc, tip_x + 8, body_bottom - 1);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let cover = CreatePen(PS_SOLID, 2, COLORREF(0x00FFFFFF));
    let old = SelectObject(dc, cover.into());
    let _ = MoveToEx(dc, tip_x - 7, body_bottom - 1, None);
    let _ = LineTo(dc, tip_x + 7, body_bottom - 1);
    let _ = SelectObject(dc, old);
    let _ = DeleteObject(cover.into());
    text(
        dc,
        font,
        NAVY,
        left + (body_w - size.cx) / 2,
        body_top + (body_h - size.cy) / 2,
        label,
    );
}

unsafe fn draw_curve(dc: HDC, solar_hours: f64, sun_up: bool) {
    let left = 24;
    let right = CLIENT_WIDTH - 24;
    let (top, bottom) = curve_box();
    let width = right - left;
    let radius = 1.55;
    let pad = 14;
    let origin_y = top - pad;
    let height = (bottom - top) + pad * 2;
    let mut pixels = vec![0u8; (width.max(0) * height.max(0) * 4) as usize];
    for x in left..right {
        for (y, cover, color) in stroke_column(x, left, width, top, bottom, radius) {
            blend_pixel(
                &mut pixels,
                width,
                height,
                x - left,
                y - origin_y,
                color,
                (cover * 255.0).round() as u8,
            );
        }
    }
    let marker_t = (solar_hours / 24.0).clamp(0.0, 0.999);
    let marker_x = f64::from(left) + marker_t * f64::from(width.max(1));
    let marker_y = curve_y(marker_t, top, bottom);
    let fill = if sun_up { 0x0064B4F0 } else { BLUE };
    paint_marker(
        &mut pixels,
        width,
        height,
        left,
        origin_y,
        marker_x,
        marker_y,
        fill,
    );
    blit_alpha(dc, left, origin_y, width, height, &pixels);
    let rule_y = bottom + 9;
    for x in (left + 1)..right {
        let t = f64::from(x - left) / f64::from(width.max(1));
        let pen = CreatePen(PS_SOLID, 2, COLORREF(curve_color(t)));
        let old = SelectObject(dc, pen.into());
        let _ = MoveToEx(dc, x - 1, rule_y, None);
        let _ = LineTo(dc, x, rule_y);
        let _ = SelectObject(dc, old);
        let _ = DeleteObject(pen.into());
    }
}

fn stroke_column(
    x: i32,
    left: i32,
    width: i32,
    top: i32,
    bottom: i32,
    radius: f64,
) -> Vec<(i32, f64, u32)> {
    let n = 4;
    let span = f64::from(width.max(1));
    let t = ((f64::from(x) + 0.5 - f64::from(left)) / span).clamp(0.0, 1.0);
    let cy = curve_y(t, top, bottom);
    let color = curve_color(t);
    let y0 = (cy - radius - 1.0).floor() as i32;
    let y1 = (cy + radius + 1.0).ceil() as i32;
    let mut rows = Vec::new();
    let total = f64::from(n * n);
    for y in y0..=y1 {
        let mut hits = 0.0;
        for sy in 0..n {
            for sx in 0..n {
                let px = f64::from(x) + (f64::from(sx) + 0.5) / f64::from(n);
                let py = f64::from(y) + (f64::from(sy) + 0.5) / f64::from(n);
                let sample_t = ((px - f64::from(left)) / span).clamp(0.0, 1.0);
                if (py - curve_y(sample_t, top, bottom)).abs() <= radius {
                    hits += 1.0;
                }
            }
        }
        let cover = hits / total;
        if cover > 0.0 {
            rows.push((y, cover, color));
        }
    }
    rows
}

fn paint_marker(
    pixels: &mut [u8],
    width: i32,
    height: i32,
    origin_x: i32,
    origin_y: i32,
    cx: f64,
    cy: f64,
    fill: u32,
) {
    let outer = 11.0;
    let inner = 9.0;
    let n = 4;
    let x0 = (cx - outer - 1.0).floor() as i32;
    let x1 = (cx + outer + 1.0).ceil() as i32;
    let y0 = (cy - outer - 1.0).floor() as i32;
    let y1 = (cy + outer + 1.0).ceil() as i32;
    let total = f64::from(n * n);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let mut fill_hits = 0.0;
            let mut ring_hits = 0.0;
            for sy in 0..n {
                for sx in 0..n {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) / f64::from(n);
                    let py = f64::from(y) + (f64::from(sy) + 0.5) / f64::from(n);
                    let distance = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                    if distance <= inner {
                        fill_hits += 1.0;
                    } else if distance <= outer {
                        ring_hits += 1.0;
                    }
                }
            }
            if ring_hits > 0.0 {
                blend_pixel(
                    pixels,
                    width,
                    height,
                    x - origin_x,
                    y - origin_y,
                    0x00FFFFFF,
                    (ring_hits / total * 255.0).round() as u8,
                );
            }
            if fill_hits > 0.0 {
                blend_pixel(
                    pixels,
                    width,
                    height,
                    x - origin_x,
                    y - origin_y,
                    fill,
                    (fill_hits / total * 255.0).round() as u8,
                );
            }
        }
    }
}

fn blend_pixel(pixels: &mut [u8], width: i32, height: i32, x: i32, y: i32, color: u32, alpha: u8) {
    if alpha == 0 || x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    let index = ((y * width + x) * 4) as usize;
    if index + 3 >= pixels.len() {
        return;
    }
    let src_a = u32::from(alpha);
    let inv = 255 - src_a;
    let src_r = (color & 0xFF) * src_a / 255;
    let src_g = ((color >> 8) & 0xFF) * src_a / 255;
    let src_b = ((color >> 16) & 0xFF) * src_a / 255;
    pixels[index] = (src_b + u32::from(pixels[index]) * inv / 255) as u8;
    pixels[index + 1] = (src_g + u32::from(pixels[index + 1]) * inv / 255) as u8;
    pixels[index + 2] = (src_r + u32::from(pixels[index + 2]) * inv / 255) as u8;
    pixels[index + 3] = (src_a + u32::from(pixels[index + 3]) * inv / 255) as u8;
}

unsafe fn blit_alpha(dc: HDC, x: i32, y: i32, width: i32, height: i32, pixels: &[u8]) {
    if width <= 0 || height <= 0 || pixels.len() < (width * height * 4) as usize {
        return;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = std::ptr::null_mut();
    let Ok(bitmap) = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) else {
        return;
    };
    if bits.is_null() {
        let _ = DeleteObject(bitmap.into());
        return;
    }
    std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits.cast(), (width * height * 4) as usize);
    let mem = CreateCompatibleDC(Some(dc));
    if mem.is_invalid() {
        let _ = DeleteObject(bitmap.into());
        return;
    }
    let previous = SelectObject(mem, bitmap.into());
    let _ = AlphaBlend(
        dc,
        x,
        y,
        width,
        height,
        mem,
        0,
        0,
        width,
        height,
        BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        },
    );
    let _ = SelectObject(mem, previous);
    let _ = DeleteDC(mem);
    let _ = DeleteObject(bitmap.into());
}

fn curve_altitude(t: f64) -> f64 {
    ((t * 24.0 - 6.0) / 24.0 * std::f64::consts::TAU).sin()
}

fn curve_y(t: f64, top: i32, bottom: i32) -> f64 {
    let mid = f64::from(top + bottom) / 2.0;
    let amp = f64::from(bottom - top) / 3.0;
    mid - curve_altitude(t) * amp
}

fn curve_color(t: f64) -> u32 {
    let day = ((curve_altitude(t) + 1.0) * 0.5).clamp(0.0, 1.0);
    mix(0x00402828, 0x0064B4F0, day)
}

fn mix(from: u32, to: u32, t: f64) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |shift: u32| {
        let a = f64::from((from >> shift) & 0xFF);
        let b = f64::from((to >> shift) & 0xFF);
        (a + (b - a) * t).round() as u32
    };
    channel(0) | (channel(8) << 8) | (channel(16) << 16)
}

unsafe fn draw_step_button(dc: HDC, font: HFONT, rect: RECT, label: &str) {
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(BLUE));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    let size = text_size(dc, font, label);
    text(
        dc,
        font,
        BLUE_DEEP,
        rect.left + (rect.right - rect.left - size.cx) / 2,
        rect.top + (rect.bottom - rect.top - size.cy) / 2,
        label,
    );
}

unsafe fn fill(dc: HDC, rect: RECT, color: u32) {
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    let brush = CreateSolidBrush(COLORREF(color));
    let _ = FillRect(dc, &rect, brush);
    let _ = DeleteObject(brush.into());
}

unsafe fn stroke_rect(dc: HDC, rect: RECT, color: u32) {
    let pen = CreatePen(PS_SOLID, 1, COLORREF(color));
    let old = SelectObject(dc, pen.into());
    let _ = MoveToEx(dc, rect.left, rect.top, None);
    let _ = LineTo(dc, rect.right - 1, rect.top);
    let _ = LineTo(dc, rect.right - 1, rect.bottom - 1);
    let _ = LineTo(dc, rect.left, rect.bottom - 1);
    let _ = LineTo(dc, rect.left, rect.top);
    let _ = SelectObject(dc, old);
    let _ = DeleteObject(pen.into());
}

unsafe fn text(dc: HDC, font: HFONT, color: u32, x: i32, y: i32, value: &str) {
    if font.is_invalid() {
        return;
    }
    let old = SelectObject(dc, font.into());
    let _ = SetTextColor(dc, COLORREF(color));
    let chars: Vec<u16> = value.encode_utf16().collect();
    let _ = TextOutW(dc, x, y, &chars);
    let _ = SelectObject(dc, old);
}

unsafe fn text_width(dc: HDC, font: HFONT, value: &str) -> i32 {
    text_size(dc, font, value).cx
}

unsafe fn text_size(dc: HDC, font: HFONT, value: &str) -> SIZE {
    if font.is_invalid() {
        return SIZE::default();
    }
    let old = SelectObject(dc, font.into());
    let mut size = SIZE::default();
    let chars: Vec<u16> = value.encode_utf16().collect();
    let _ = GetTextExtentPoint32W(dc, &chars, &mut size);
    let _ = SelectObject(dc, old);
    size
}

unsafe fn make_font(height: i32, weight: i32) -> HFONT {
    let mut face = [0i8; 32];
    for (slot, byte) in face.iter_mut().zip(b"Calibri\0") {
        *slot = *byte as i8;
    }
    let mut log = LOGFONTA::default();
    log.lfHeight = height;
    log.lfWeight = weight;
    log.lfQuality = windows::Win32::Graphics::Gdi::FONT_QUALITY(5);
    log.lfFaceName = face;
    CreateFontIndirectA(&log)
}

fn right_edge() -> i32 {
    CLIENT_WIDTH - 24
}

fn content_top() -> i32 {
    current_tiles()
        .iter()
        .map(|tile| tile.bounds.top)
        .min()
        .unwrap_or(monitor_band().top)
}

fn live_dim_rect() -> ScreenRect {
    dim_control_pair(content_top()).1
}

fn brightness_rect() -> RECT {
    rect_from(dim_control_pair(content_top()).0)
}

fn dimming_rect() -> RECT {
    rect_from(live_dim_rect())
}

fn header_row() -> RECT {
    RECT {
        left: 16,
        top: 12,
        right: 90,
        bottom: 44,
    }
}

fn preset_pulldown_width() -> i32 {
    let (font, names) = APP.with(|app| {
        let app = app.borrow();
        let mut names: Vec<String> = BUILTIN_PRESETS
            .iter()
            .map(|preset| preset.name.to_string())
            .collect();
        names.extend(app.customs.iter().map(|preset| preset.name.clone()));
        names.push("Custom".to_string());
        (app.font_body, names)
    });
    unsafe {
        let dc = GetDC(None);
        if dc.is_invalid() || font.is_invalid() {
            return 200;
        }
        let text = names
            .iter()
            .map(|name| text_width(dc, font, name))
            .max()
            .unwrap_or(120);
        let _ = ReleaseDC(None, dc);
        (text + 44).clamp(168, 236)
    }
}

fn gear_rect() -> RECT {
    RECT {
        left: 16,
        top: 12,
        right: 48,
        bottom: 44,
    }
}

fn preset_rect() -> RECT {
    let left = gear_rect().right + 8;
    RECT {
        left,
        top: 12,
        right: left + preset_pulldown_width(),
        bottom: 44,
    }
}

fn save_rect() -> RECT {
    let preset = preset_rect();
    RECT {
        left: preset.right + 8,
        top: 12,
        right: preset.right + 146,
        bottom: 44,
    }
}
fn kelvin_label_widths() -> (i32, i32) {
    let (font, day, night) = APP.with(|app| {
        let app = app.borrow();
        (
            app.font_kelvin,
            format!("{}K", app.settings.day_kelvin),
            format!("{}K", app.settings.night_kelvin),
        )
    });
    unsafe {
        let dc = GetDC(None);
        if dc.is_invalid() {
            return (96, 96);
        }
        let day_width = text_width(dc, font, &day).max(1);
        let night_width = text_width(dc, font, &night).max(1);
        let _ = ReleaseDC(None, dc);
        (night_width, day_width)
    }
}

/// Day curve under the header. The slider and sentence sit below its rule.
fn curve_box() -> (i32, i32) {
    (66, 152)
}

fn slider_track() -> RECT {
    let (night_width, day_width) = kelvin_label_widths();
    let (left, right) = slider_track_edges(24, right_edge(), night_width, day_width, 16);
    RECT {
        left,
        top: 220,
        right,
        bottom: 226,
    }
}

fn slider_hit_rect() -> RECT {
    let track = slider_track();
    RECT {
        left: track.left - 14,
        top: track.top - 18,
        right: track.right + 14,
        bottom: track.bottom + 18,
    }
}
fn time_clock_width() -> i32 {
    let font = APP.with(|app| app.borrow().font_wake);
    unsafe {
        let dc = GetDC(None);
        if dc.is_invalid() {
            return 108;
        }
        let width = text_width(dc, font, "12:00 PM");
        let _ = ReleaseDC(None, dc);
        width.max(1)
    }
}

fn point_in(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.y >= rect.top && point.x < rect.right && point.y < rect.bottom
}

fn lparam_point(lparam: LPARAM) -> POINT {
    POINT {
        x: (lparam.0 & 0xFFFF) as i16 as i32,
        y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
    }
}

fn rect_from(rect: ScreenRect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

fn degrees_text(latitude_hundredths: i32, longitude_hundredths: i32) -> String {
    let full = location_text(latitude_hundredths, longitude_hundredths);
    full.trim_start_matches("Location: ").to_string()
}

fn time_edit_bounds(outer: RECT) -> RECT {
    let font = APP.with(|app| app.borrow().font_wake_edit);
    let text_h = unsafe {
        let dc = GetDC(None);
        let height = if dc.is_invalid() {
            18
        } else {
            text_size(dc, font, "12:30 AM").cy.max(1)
        };
        if !dc.is_invalid() {
            let _ = ReleaseDC(None, dc);
        }
        height
    };
    let outer_h = (outer.bottom - outer.top).max(text_h);
    let top = outer.top + (outer_h - text_h) / 2;
    RECT {
        left: outer.left + 4,
        top,
        right: outer.right - 4,
        bottom: top + text_h,
    }
}

fn open_time_edit(parent: HWND, field: TimeField) {
    let existing = APP.with(|app| app.borrow().time_edit);
    if !existing.0.is_null() {
        let _ = unsafe { SetFocus(Some(existing)) };
        return;
    }
    let text = APP.with(|app| {
        let app = app.borrow();
        let minute = app
            .time
            .as_ref()
            .map(|time| match field {
                TimeField::Transition => time.draft.transition_minute,
                TimeField::Revert => time.draft.revert_minute,
            })
            .unwrap_or(if field == TimeField::Transition {
                DEFAULT_TRANSITION_MINUTE
            } else {
                DEFAULT_REVERT_MINUTE
            });
        format_clock(minute)
    });
    let outer = match field {
        TimeField::Transition => time_transition_clock_rect(),
        TimeField::Revert => time_revert_clock_rect(),
    };
    let rect = time_edit_bounds(outer);
    unsafe {
        let instance = GetModuleHandleA(None).unwrap_or_default();
        let edit = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            s!("EDIT"),
            PCSTR(title_ansi(&text).as_ptr().cast()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x81),
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            Some(parent),
            Some(HMENU(1200isize as _)),
            Some(instance.into()),
            None,
        )
        .unwrap_or_default();
        if edit.0.is_null() {
            return;
        }
        let font = APP.with(|app| app.borrow().font_wake_edit);
        if !font.is_invalid() {
            let _ = SendMessageW(
                edit,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
        }
        let _ = SetWindowTextW(edit, PCWSTR(wide_z(&text).as_ptr()));
        let previous = SetWindowLongPtrW(edit, GWLP_WNDPROC, time_edit_proc as *const () as isize);
        WAKE_OLD_PROC.with(|cell| cell.set(previous));
        APP.with(|app| {
            let mut app = app.borrow_mut();
            app.time_edit = edit;
            app.time_field = Some(field);
        });
        let _ = InvalidateRect(Some(parent), None, false);
        let _ = SetFocus(Some(edit));
        let _ = SendMessageW(edit, 0x00B1, Some(WPARAM(0)), Some(LPARAM(-1)));
    }
}

fn finish_time_edit(commit: bool, close_if_invalid: bool) {
    let edit = APP.with(|app| app.borrow().time_edit);
    if edit.0.is_null() {
        return;
    }
    let text = unsafe { window_text(edit) };
    if commit {
        if let Some(minute) = parse_clock(&text) {
            let field = APP.with(|app| app.borrow().time_field);
            APP.with(|app| {
                let mut app = app.borrow_mut();
                app.time_edit = HWND::default();
                app.time_field = None;
                if let Some(time) = app.time.as_mut() {
                    match field {
                        Some(TimeField::Transition) => {
                            time.draft.transition_minute = minute.rem_euclid(24 * 60)
                        }
                        Some(TimeField::Revert) => {
                            time.draft.revert_minute = minute.rem_euclid(24 * 60)
                        }
                        None => {}
                    }
                }
            });
            unsafe {
                let _ = DestroyWindow(edit);
            }
            invalidate_time_dialog();
            return;
        }
        if !close_if_invalid {
            return;
        }
    }
    APP.with(|app| {
        let mut app = app.borrow_mut();
        app.time_edit = HWND::default();
        app.time_field = None;
    });
    unsafe {
        let _ = DestroyWindow(edit);
        invalidate_time_dialog();
    }
}

fn invalidate_time_dialog() {
    let hwnd = APP.with(|app| {
        app.borrow()
            .time
            .as_ref()
            .map(|time| time.hwnd)
            .unwrap_or_default()
    });
    if !hwnd.0.is_null() {
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
}

unsafe fn window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let length = if hwnd.0.is_null() {
        0
    } else {
        GetWindowTextW(hwnd, &mut buf)
    };
    String::from_utf16_lossy(&buf[..length.max(0) as usize])
}

unsafe extern "system" fn time_edit_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_KEYDOWN {
        let key = wparam.0 as u16;
        if key == VK_RETURN.0 {
            finish_time_edit(true, false);
            return LRESULT(0);
        }
        if key == VK_ESCAPE.0 {
            finish_time_edit(false, true);
            return LRESULT(0);
        }
    }
    if msg == WM_KILLFOCUS {
        finish_time_edit(true, true);
        return LRESULT(0);
    }
    let previous = WAKE_OLD_PROC.with(|cell| cell.get());
    if previous == 0 {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }
    let proc: WNDPROC = std::mem::transmute(previous);
    CallWindowProcW(proc, hwnd, msg, wparam, lparam)
}

unsafe fn fill_brightness_menu(menu: HMENU, kept: &mut Vec<CString>) {
    let index = APP.with(|app| app.borrow().settings.brightness_index);
    for (offset, label) in BRIGHTNESS_LABELS.iter().enumerate() {
        let flags = if offset as u32 == index {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING
        };
        append(menu, flags, CMD_BRIGHTNESS + offset as u32, label, kept);
    }
}

unsafe fn show_brightness_menu(hwnd: HWND) {
    let mut kept = Vec::new();
    let menu = CreatePopupMenu().unwrap_or_default();
    fill_brightness_menu(menu, &mut kept);
    show_popup_at(hwnd, menu, brightness_rect());
    let _ = DestroyMenu(menu);
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let mut kept = Vec::new();
    let menu = CreatePopupMenu().unwrap_or_default();
    let dim_on = {
        let moment = dim_moment(&current_schedule(), &local_time());
        APP.with(|app| {
            let app = app.borrow();
            dimming_enabled(app.dim_schedule, moment, app.dim_override)
        })
    };
    append(menu, MF_STRING, CMD_DIM, dimming_label(dim_on), &mut kept);
    let brightness = CreatePopupMenu().unwrap_or_default();
    fill_brightness_menu(brightness, &mut kept);
    append_popup(menu, brightness, "Brightness", &mut kept);
    append(menu, MF_SEPARATOR, 0, "", &mut kept);
    let presets = CreatePopupMenu().unwrap_or_default();
    append(presets, MF_STRING, CMD_DEFAULT, "Default", &mut kept);
    let customs = APP.with(|app| app.borrow().customs.clone());
    for (index, preset) in customs.iter().enumerate() {
        append(
            presets,
            MF_STRING,
            CMD_PRESET + (BUILTIN_PRESETS.len() + index) as u32,
            &preset.name,
            &mut kept,
        );
    }
    append_popup(menu, presets, "Presets", &mut kept);
    append(menu, MF_STRING, CMD_ABOUT, "About", &mut kept);
    append(menu, MF_STRING, CMD_EXIT, "Exit", &mut kept);
    let _ = SetForegroundWindow(hwnd);
    let mut cursor = POINT::default();
    let _ = GetCursorPos(&mut cursor);
    let _ = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON,
        cursor.x,
        cursor.y,
        Some(0),
        hwnd,
        None,
    );
    let _ = PostMessageA(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);
}

struct AboutLink {
    label: &'static str,
    url: &'static str,
    rect: RECT,
}

struct AboutLayout {
    icon: RECT,
    title: POINT,
    version: POINT,
    description: RECT,
    copyright: POINT,
    links: Vec<AboutLink>,
    close: RECT,
    height: i32,
}

unsafe fn show_about(owner: HWND) {
    let existing = APP.with(|app| app.borrow().about_hwnd);
    if !existing.0.is_null() && IsWindow(Some(existing)).as_bool() {
        let _ = ShowWindow(existing, SW_SHOW);
        let _ = SetForegroundWindow(existing);
        return;
    }
    let icon = shell_icon(ABOUT_ICON, ABOUT_ICON).unwrap_or_default();
    let shown = !owner.0.is_null()
        && IsWindowVisible(owner).as_bool()
        && !IsIconic(owner).as_bool();
    let modal = shown && IsWindowEnabled(owner).as_bool();
    APP.with(|app| {
        let mut app = app.borrow_mut();
        app.about_icon = icon;
        app.about_modal = modal;
    });
    let screen = GetDC(None);
    let layout = about_layout(screen);
    if !screen.is_invalid() {
        let _ = ReleaseDC(None, screen);
    }
    let instance = GetModuleHandleA(None).unwrap_or_default();
    let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE;
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: ABOUT_WIDTH,
        bottom: layout.height,
    };
    let _ = windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
    let width = bounds.right - bounds.left;
    let height = bounds.bottom - bounds.top;
    let (x, y) = if owner.0.is_null() {
        let screen = virtual_screen();
        dialog_origin(screen, width, height, screen)
    } else {
        popup_origin(owner, width, height)
    };
    let parent = if modal { Some(owner) } else { None };
    let hwnd = CreateWindowExA(
        WINDOW_EX_STYLE(0x101),
        leak_cstr(CLASS_ABOUT),
        leak_cstr("About EvenTide"),
        style,
        x,
        y,
        width,
        height,
        parent,
        None,
        Some(instance.into()),
        None,
    );
    let Ok(hwnd) = hwnd else {
        close_about_icon();
        return;
    };
    if modal {
        let _ = EnableWindow(owner, false);
    }
    APP.with(|app| app.borrow_mut().about_hwnd = hwnd);
    let _ = SetForegroundWindow(hwnd);
    let _ = SetFocus(Some(hwnd));
}

fn close_about_icon() {
    let icon = APP.with(|app| {
        let mut app = app.borrow_mut();
        let icon = app.about_icon;
        app.about_icon = HICON::default();
        app.about_modal = false;
        icon
    });
    if !icon.0.is_null() {
        unsafe {
            let _ = DestroyIcon(icon);
        }
    }
}

unsafe fn about_layout(dc: HDC) -> AboutLayout {
    let (header, body, small) = APP.with(|app| {
        let app = app.borrow();
        (app.font_header, app.font_body, app.font_small)
    });
    let title_size = text_size(dc, header, APP_NAME);
    let version_size = text_size(dc, small, ABOUT_VERSION);
    let icon = RECT {
        left: ABOUT_MARGIN,
        top: ABOUT_MARGIN,
        right: ABOUT_MARGIN + ABOUT_ICON,
        bottom: ABOUT_MARGIN + ABOUT_ICON,
    };
    let block = title_size.cy.max(18) + 4 + version_size.cy.max(14);
    let block_top = icon.top + (ABOUT_ICON - block).max(0) / 2;
    let text_x = icon.right + 16;
    let title = POINT {
        x: text_x,
        y: block_top,
    };
    let version = POINT {
        x: text_x,
        y: block_top + title_size.cy.max(18) + 4,
    };
    let mut y = icon.bottom.max(version.y + version_size.cy.max(14)) + 20;
    let mut description = RECT {
        left: ABOUT_MARGIN,
        top: y,
        right: ABOUT_WIDTH - ABOUT_MARGIN,
        bottom: y + 8,
    };
    if !dc.is_invalid() {
        let _ = draw_wrapped(
            dc,
            body,
            INK,
            &mut description,
            ABOUT_DESCRIPTION,
            DT_WORDBREAK | DT_NOPREFIX | DT_CALCRECT,
        );
    }
    if description.bottom < description.top + 18 {
        description.bottom = description.top + 36;
    }
    y = description.bottom + 16;
    let copyright = POINT {
        x: ABOUT_MARGIN,
        y,
    };
    y += text_size(dc, small, ABOUT_COPYRIGHT).cy.max(16) + 12;
    let mut links = Vec::new();
    for (label, url) in ABOUT_LINKS {
        let size = text_size(dc, body, label);
        let rect = RECT {
            left: ABOUT_MARGIN,
            top: y,
            right: ABOUT_MARGIN + size.cx.max(1),
            bottom: y + size.cy.max(18),
        };
        y = rect.bottom + 8;
        links.push(AboutLink {
            label,
            url,
            rect,
        });
    }
    y += 12;
    let close = RECT {
        left: ABOUT_WIDTH - ABOUT_MARGIN - 96,
        top: y,
        right: ABOUT_WIDTH - ABOUT_MARGIN,
        bottom: y + 36,
    };
    AboutLayout {
        icon,
        title,
        version,
        description,
        copyright,
        links,
        close,
        height: close.bottom + 20,
    }
}

unsafe fn draw_wrapped(
    dc: HDC,
    font: HFONT,
    color: u32,
    rect: &mut RECT,
    value: &str,
    format: DRAW_TEXT_FORMAT,
) -> i32 {
    if font.is_invalid() || dc.is_invalid() {
        return 0;
    }
    let old = SelectObject(dc, font.into());
    let _ = SetTextColor(dc, COLORREF(color));
    let mut chars: Vec<u16> = value.encode_utf16().collect();
    let height = DrawTextW(dc, &mut chars, rect, format);
    let _ = SelectObject(dc, old);
    height
}

unsafe fn link_font() -> HFONT {
    let mut face = [0i8; 32];
    for (slot, byte) in face.iter_mut().zip(b"Calibri\0") {
        *slot = *byte as i8;
    }
    let mut log = LOGFONTA::default();
    log.lfHeight = -16;
    log.lfWeight = 400;
    log.lfUnderline = 1;
    log.lfQuality = windows::Win32::Graphics::Gdi::FONT_QUALITY(5);
    log.lfFaceName = face;
    CreateFontIndirectA(&log)
}

unsafe fn open_about_link(hwnd: HWND, url: &str) {
    let mut file: Vec<u16> = url.encode_utf16().collect();
    file.push(0);
    let _ = ShellExecuteW(
        Some(hwnd),
        windows::core::w!("open"),
        windows::core::PCWSTR(file.as_ptr()),
        windows::core::PCWSTR::null(),
        windows::core::PCWSTR::null(),
        SW_SHOWNORMAL,
    );
}

unsafe fn paint_about(hwnd: HWND) {
    let mut paint = PAINTSTRUCT::default();
    let window_dc = BeginPaint(hwnd, &mut paint);
    if window_dc.is_invalid() {
        return;
    }
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);
    let buffer = CreateCompatibleDC(Some(window_dc));
    let bitmap = CreateCompatibleBitmap(window_dc, client.right.max(1), client.bottom.max(1));
    let previous = SelectObject(buffer, bitmap.into());
    fill(buffer, client, BG);
    let _ = SetBkMode(buffer, TRANSPARENT);
    let layout = about_layout(buffer);
    let icon = APP.with(|app| app.borrow().about_icon);
    if !icon.0.is_null() {
        let _ = DrawIconEx(
            buffer,
            layout.icon.left,
            layout.icon.top,
            icon,
            ABOUT_ICON,
            ABOUT_ICON,
            0,
            None,
            DI_NORMAL,
        );
    }
    let (header, body, small) = APP.with(|app| {
        let app = app.borrow();
        (app.font_header, app.font_body, app.font_small)
    });
    text(buffer, header, INK, layout.title.x, layout.title.y, APP_NAME);
    text(
        buffer,
        small,
        MUTED,
        layout.version.x,
        layout.version.y,
        ABOUT_VERSION,
    );
    let mut description = layout.description;
    let _ = draw_wrapped(
        buffer,
        body,
        INK,
        &mut description,
        ABOUT_DESCRIPTION,
        DT_WORDBREAK | DT_NOPREFIX,
    );
    text(
        buffer,
        small,
        MUTED,
        layout.copyright.x,
        layout.copyright.y,
        ABOUT_COPYRIGHT,
    );
    let underline = link_font();
    for link in &layout.links {
        text(
            buffer,
            underline,
            BLUE_DEEP,
            link.rect.left,
            link.rect.top,
            link.label,
        );
    }
    if !underline.is_invalid() {
        let _ = DeleteObject(underline.into());
    }
    draw_pill(
        buffer,
        body,
        screen_from(layout.close),
        "Close",
        true,
    );
    let _ = BitBlt(
        window_dc,
        0,
        0,
        client.right,
        client.bottom,
        Some(buffer),
        0,
        0,
        SRCCOPY,
    );
    let _ = SelectObject(buffer, previous);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(buffer);
    let _ = EndPaint(hwnd, &paint);
}

fn about_link_at(point: POINT, layout: &AboutLayout) -> Option<&'static str> {
    layout
        .links
        .iter()
        .find(|link| point_in(point, link.rect))
        .map(|link| link.url)
}

unsafe extern "system" fn about_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            paint_about(hwnd);
            LRESULT(0)
        }
        WM_SETCURSOR => {
            let hit = (lparam.0 as u32) & 0xFFFF;
            if hit == HTCLIENT {
                let mut point = POINT::default();
                if GetCursorPos(&mut point).is_ok() && ScreenToClient(hwnd, &mut point).as_bool() {
                    let dc = GetDC(Some(hwnd));
                    let layout = about_layout(dc);
                    if !dc.is_invalid() {
                        let _ = ReleaseDC(Some(hwnd), dc);
                    }
                    if about_link_at(point, &layout).is_some() {
                        SetCursor(Some(LoadCursorW(None, IDC_HAND).unwrap_or_default()));
                        return LRESULT(1);
                    }
                }
            }
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
        WM_LBUTTONDOWN => {
            let point = lparam_point(lparam);
            let dc = GetDC(Some(hwnd));
            let layout = about_layout(dc);
            if !dc.is_invalid() {
                let _ = ReleaseDC(Some(hwnd), dc);
            }
            if point_in(point, layout.close) {
                let _ = DestroyWindow(hwnd);
            } else if let Some(url) = about_link_at(point, &layout) {
                open_about_link(hwnd, url);
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            if key == VK_ESCAPE.0 || key == VK_RETURN.0 {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            let (owner, modal) = APP.with(|app| {
                let mut app = app.borrow_mut();
                app.about_hwnd = HWND::default();
                let modal = app.about_modal;
                app.about_modal = false;
                (app.hwnd, modal)
            });
            close_about_icon();
            if modal && !owner.0.is_null() && IsWindow(Some(owner)).as_bool() {
                let _ = EnableWindow(owner, true);
                if IsWindowVisible(owner).as_bool() {
                    let _ = SetForegroundWindow(owner);
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_preset_menu(hwnd: HWND) {
    let mut kept = Vec::new();
    let menu = CreatePopupMenu().unwrap_or_default();
    let customs = APP.with(|app| app.borrow().customs.clone());
    let names = preset_menu_names(&customs);
    let custom_count = customs.len();
    for (index, name) in names.iter().enumerate() {
        if index == custom_count && custom_count > 0 {
            append(menu, MF_SEPARATOR, 0, "", &mut kept);
        }
        let id = if index < custom_count {
            CMD_PRESET + (BUILTIN_PRESETS.len() + index) as u32
        } else {
            CMD_PRESET + (index - custom_count) as u32
        };
        append(menu, MF_STRING, id, name, &mut kept);
    }
    show_popup_at(hwnd, menu, preset_rect());
    let _ = DestroyMenu(menu);
}

fn screen_rect(window: RECT) -> ScreenRect {
    ScreenRect {
        left: window.left,
        top: window.top,
        right: window.right,
        bottom: window.bottom,
    }
}

unsafe fn virtual_screen() -> ScreenRect {
    let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
    ScreenRect {
        left,
        top,
        right: left + GetSystemMetrics(SM_CXVIRTUALSCREEN),
        bottom: top + GetSystemMetrics(SM_CYVIRTUALSCREEN),
    }
}

unsafe fn popup_origin(parent: HWND, width: i32, height: i32) -> (i32, i32) {
    let mut window = RECT::default();
    let _ = GetWindowRect(parent, &mut window);
    dialog_origin(screen_rect(window), width, height, virtual_screen())
}

unsafe fn disable_window_transitions(hwnd: HWND) {
    let disabled: i32 = 1;
    let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_TRANSITIONS_FORCEDISABLED,
        &disabled as *const i32 as *const core::ffi::c_void,
        std::mem::size_of::<i32>() as u32,
    );
}

unsafe fn show_city_menu(owner: HWND) {
    let row = city_row_px();
    let font = menu_font();
    let width = city_menu_width(font).max(160);
    let height = row * (CITY_MENU_ROWS + 1);
    let anchor = APP.with(|app| app.borrow().location_hit);
    let mut hit = POINT {
        x: anchor.right,
        y: anchor.top,
    };
    let _ = ClientToScreen(owner, &mut hit);
    let mut window = RECT::default();
    let _ = GetWindowRect(owner, &mut window);
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };
    let style = WS_POPUP | WS_BORDER;
    let _ = windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
    let window_w = bounds.right - bounds.left;
    let window_h = bounds.bottom - bounds.top;
    let (left, top) = city_menu_origin(
        screen_rect(window),
        hit.x,
        window_w,
        window_h,
        virtual_screen(),
    );
    CITY_SCROLL.with(|cell| cell.set(0));
    CITY_HOT.with(|cell| cell.set(-1));
    CITY_CHOICE.with(|cell| cell.set(-1));
    CITY_WHEEL.with(|cell| cell.set(0));
    CITY_ARMED.with(|cell| cell.set(false));
    CITY_OPEN.with(|cell| cell.set(true));
    CITY_FONT.with(|cell| cell.set(font.0 as isize));
    let instance = GetModuleHandleA(None).unwrap_or_default();
    let popup = CreateWindowExA(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        leak_cstr(CLASS_CITY),
        leak_cstr("United States"),
        style,
        left,
        top,
        window_w,
        window_h,
        Some(owner),
        None,
        Some(instance.into()),
        None,
    )
    .unwrap_or_default();
    if popup.0.is_null() {
        CITY_OPEN.with(|cell| cell.set(false));
        if !font.is_invalid() {
            let _ = DeleteObject(font.into());
        }
        return;
    }
    CITY_HWND.with(|cell| cell.set(popup.0 as isize));
    disable_window_transitions(popup);
    let _ = SetWindowPos(
        popup,
        Some(HWND_TOPMOST),
        0,
        0,
        0,
        0,
        SET_WINDOW_POS_FLAGS(LENS_RAISE_FLAGS) | SWP_NOREDRAW,
    );
    let _ = ShowWindow(popup, SW_SHOW);
    let _ = InvalidateRect(Some(popup), None, false);
    let _ = UpdateWindow(popup);
    let _ = SetForegroundWindow(popup);
    let _ = SetFocus(Some(popup));
    let _ = SetCapture(popup);
    let mut msg = MSG::default();
    let mut running = true;
    while CITY_OPEN.with(|cell| cell.get()) {
        running = GetMessageA(&mut msg, None, 0, 0).into();
        if !running {
            break;
        }
        if msg.message == WM_MOUSEWHEEL {
            let over = cursor_over(popup);
            let _ = city_wheel(over);
            if over {
                scroll_city_list(wheel_delta(msg.wParam));
                set_city_hot(city_index_at_cursor(popup));
            }
            continue;
        }
        if msg.message == WM_LBUTTONUP && !CITY_ARMED.with(|cell| cell.get()) {
            let _ = city_opening_release(city_index_at_cursor(popup));
            CITY_ARMED.with(|cell| cell.set(true));
            APP.with(|app| app.borrow_mut().dragging = false);
            set_city_hot(city_index_at_cursor(popup));
            continue;
        }
        if msg.message == WM_LBUTTONDOWN && !cursor_over(popup) {
            if city_press(false) == CityEffect::Close {
                CITY_OPEN.with(|cell| cell.set(false));
            }
            continue;
        }
        if msg.message == WM_KEYDOWN && msg.wParam.0 as u16 == VK_ESCAPE.0 {
            if city_escape() == CityEffect::Close {
                CITY_OPEN.with(|cell| cell.set(false));
            }
            continue;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
    let choice = CITY_CHOICE.with(|cell| cell.get());
    let _ = ReleaseCapture();
    CITY_HWND.with(|cell| cell.set(0));
    CITY_OPEN.with(|cell| cell.set(false));
    let _ = DestroyWindow(popup);
    if !font.is_invalid() {
        let _ = DeleteObject(font.into());
    }
    CITY_FONT.with(|cell| cell.set(0));
    let _ = SetForegroundWindow(owner);
    if choice >= 0 {
        let _ = PostMessageA(Some(owner), WM_COMMAND, WPARAM(choice as usize), LPARAM(0));
    }
    if !running {
        PostQuitMessage(msg.wParam.0 as i32);
    }
}

unsafe extern "system" fn city_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            paint_city_menu(hwnd);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            set_city_hot(city_index_at(lparam_point(lparam).y));
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let armed = CITY_ARMED.with(|cell| cell.get());
            let index = city_index_at(lparam_point(lparam).y);
            match city_release(armed, index) {
                CityEffect::Choose(index) => {
                    CITY_CHOICE.with(|cell| cell.set(CMD_CITY as i32 + index));
                    CITY_OPEN.with(|cell| cell.set(false));
                }
                CityEffect::Close => CITY_OPEN.with(|cell| cell.set(false)),
                CityEffect::Stay => {
                    if !armed {
                        CITY_ARMED.with(|cell| cell.set(true));
                    }
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let mut client = RECT::default();
            let _ = GetClientRect(hwnd, &mut client);
            if city_press(point_in(lparam_point(lparam), client)) == CityEffect::Close {
                CITY_OPEN.with(|cell| cell.set(false));
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let over = cursor_over(hwnd);
            let _ = city_wheel(over);
            if over {
                scroll_city_list(wheel_delta(wparam));
                set_city_hot(city_index_at_cursor(hwnd));
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            if key == VK_ESCAPE.0 && city_escape() == CityEffect::Close {
                CITY_OPEN.with(|cell| cell.set(false));
            } else if key == VK_RETURN.0 {
                let hot = CITY_HOT.with(|cell| cell.get());
                if hot >= 0 {
                    CITY_CHOICE.with(|cell| cell.set(CMD_CITY as i32 + hot));
                    CITY_OPEN.with(|cell| cell.set(false));
                }
            } else if key == VK_DOWN.0 || key == VK_UP.0 {
                move_city_hot(if key == VK_DOWN.0 { 1 } else { -1 });
            }
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

fn city_row_px() -> i32 {
    unsafe { menu_row_height().max(22) as i32 }
}

fn city_index_at(y: i32) -> i32 {
    city_row_index(
        y,
        city_row_px(),
        CITY_SCROLL.with(|cell| cell.get()),
        CITIES.len() as i32,
    )
}

fn set_city_hot(index: i32) {
    let changed = CITY_HOT.with(|cell| {
        let changed = cell.get() != index;
        cell.set(index);
        changed
    });
    if changed {
        let hwnd = city_hwnd();
        if !hwnd.0.is_null() {
            let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
        }
    }
}

fn scroll_city_list(raw_delta: i32) {
    let mut pending = CITY_WHEEL.with(|cell| cell.get());
    let scroll = CITY_SCROLL.with(|cell| cell.get());
    let next = scroll_city_wheel(
        scroll,
        &mut pending,
        raw_delta,
        wheel_scroll_lines(),
        CITIES.len() as i32,
        CITY_MENU_ROWS,
    );
    CITY_WHEEL.with(|cell| cell.set(pending));
    let changed = CITY_SCROLL.with(|cell| {
        let changed = cell.get() != next;
        cell.set(next);
        changed
    });
    if changed {
        let hwnd = city_hwnd();
        if !hwnd.0.is_null() {
            let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
        }
    }
}

fn move_city_hot(step: i32) {
    let max_index = CITIES.len() as i32 - 1;
    let current = CITY_HOT.with(|cell| cell.get());
    let next = if current < 0 {
        CITY_SCROLL.with(|cell| cell.get())
    } else {
        (current + step).clamp(0, max_index)
    };
    let max_scroll = (CITIES.len() as i32 - CITY_MENU_ROWS).max(0);
    let mut scroll = CITY_SCROLL.with(|cell| cell.get());
    if next < scroll {
        scroll = next;
    } else if next >= scroll + CITY_MENU_ROWS {
        scroll = next - CITY_MENU_ROWS + 1;
    }
    scroll = scroll.clamp(0, max_scroll);
    CITY_SCROLL.with(|cell| cell.set(scroll));
    CITY_HOT.with(|cell| cell.set(next));
    let hwnd = city_hwnd();
    if !hwnd.0.is_null() {
        let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
    }
}

fn city_hwnd() -> HWND {
    HWND(CITY_HWND.with(|cell| cell.get()) as *mut core::ffi::c_void)
}

fn city_index_at_cursor(hwnd: HWND) -> i32 {
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        return -1;
    }
    let _ = unsafe { windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut cursor) };
    city_index_at(cursor.y)
}

fn cursor_over(hwnd: HWND) -> bool {
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        return false;
    }
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok() && point_in(cursor, rect)
}

fn wheel_delta(wparam: WPARAM) -> i32 {
    ((wparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32
}

fn wheel_scroll_lines() -> i32 {
    let mut lines = 3u32;
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWHEELSCROLLLINES,
            0,
            Some((&mut lines as *mut u32).cast()),
            Default::default(),
        )
    };
    if ok.is_err() || lines == 0 {
        3
    } else if lines == u32::MAX {
        CITY_MENU_ROWS
    } else {
        lines.min(CITY_MENU_ROWS as u32) as i32
    }
}

unsafe fn menu_font() -> HFONT {
    let mut metrics = NONCLIENTMETRICSW {
        cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
        ..Default::default()
    };
    let ok = SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS,
        metrics.cbSize,
        Some((&mut metrics as *mut NONCLIENTMETRICSW).cast()),
        Default::default(),
    );
    let font = if ok.is_ok() {
        CreateFontIndirectW(&metrics.lfMenuFont)
    } else {
        HFONT::default()
    };
    if font.is_invalid() {
        make_font(-14, 400)
    } else {
        font
    }
}

unsafe fn city_menu_width(font: HFONT) -> i32 {
    let dc = CreateCompatibleDC(None);
    let mut width = text_width(dc, font, "United States");
    for city in CITIES {
        width = width.max(text_width(dc, font, &city.label()));
    }
    let _ = DeleteDC(dc);
    width + 36
}

unsafe fn paint_city_menu(hwnd: HWND) {
    let mut paint = PAINTSTRUCT::default();
    let window_dc = BeginPaint(hwnd, &mut paint);
    if window_dc.is_invalid() {
        return;
    }
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);
    let buffer = CreateCompatibleDC(Some(window_dc));
    let bitmap = CreateCompatibleBitmap(window_dc, client.right.max(1), client.bottom.max(1));
    let previous = SelectObject(buffer, bitmap.into());
    let row = city_row_px();
    let scroll = CITY_SCROLL.with(|cell| cell.get());
    let hot = CITY_HOT.with(|cell| cell.get());
    let font = HFONT(CITY_FONT.with(|cell| cell.get()) as *mut core::ffi::c_void);
    let menu_color = GetSysColor(COLOR_MENU);
    let text_color = GetSysColor(COLOR_MENUTEXT);
    let hot_color = GetSysColor(COLOR_HIGHLIGHT);
    let hot_text = GetSysColor(COLOR_HIGHLIGHTTEXT);
    fill(buffer, client, menu_color);
    let _ = SetBkMode(buffer, TRANSPARENT);
    text(
        buffer,
        font,
        text_color,
        12,
        (row - text_size(buffer, font, "United States").cy) / 2,
        "United States",
    );
    let line = RECT {
        left: 0,
        top: row - 1,
        right: client.right,
        bottom: row,
    };
    fill(buffer, line, text_color);
    let max_scroll = (CITIES.len() as i32 - CITY_MENU_ROWS).max(0);
    for slot in 0..CITY_MENU_ROWS {
        let index = scroll + slot;
        if index < 0 || index >= CITIES.len() as i32 {
            continue;
        }
        let top = row * (slot + 1);
        let item = RECT {
            left: 0,
            top,
            right: client.right,
            bottom: top + row,
        };
        let selected = index == hot;
        if selected {
            fill(buffer, item, hot_color);
        }
        let label = CITIES[index as usize].label();
        let color = if selected { hot_text } else { text_color };
        let text_y = top + (row - text_size(buffer, font, &label).cy) / 2;
        text(buffer, font, color, 12, text_y, &label);
        if slot == 0 && scroll > 0 {
            let mark = "▲";
            let mark_w = text_width(buffer, font, mark);
            text(buffer, font, color, client.right - mark_w - 8, text_y, mark);
        }
        if slot == CITY_MENU_ROWS - 1 && scroll < max_scroll {
            let mark = "▼";
            let mark_w = text_width(buffer, font, mark);
            text(buffer, font, color, client.right - mark_w - 8, text_y, mark);
        }
    }
    let _ = BitBlt(
        window_dc,
        0,
        0,
        client.right,
        client.bottom,
        Some(buffer),
        0,
        0,
        SRCCOPY,
    );
    let _ = SelectObject(buffer, previous);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(buffer);
    let _ = EndPaint(hwnd, &paint);
}

unsafe fn menu_row_height() -> u32 {
    let mut metrics = NONCLIENTMETRICSW {
        cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
        ..Default::default()
    };
    let ok = SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS,
        metrics.cbSize,
        Some((&mut metrics as *mut NONCLIENTMETRICSW).cast()),
        Default::default(),
    );
    if ok.is_ok() && metrics.iMenuHeight > 0 {
        metrics.iMenuHeight as u32
    } else {
        GetSystemMetrics(SM_CYMENU).max(18) as u32
    }
}

unsafe fn show_popup_at(hwnd: HWND, menu: HMENU, anchor: RECT) {
    let mut origin = POINT {
        x: anchor.left,
        y: anchor.bottom,
    };
    let _ = ClientToScreen(hwnd, &mut origin);
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON,
        origin.x,
        origin.y,
        Some(0),
        hwnd,
        None,
    );
}

unsafe fn append(
    menu: HMENU,
    flags: MENU_ITEM_FLAGS,
    id: u32,
    text: &str,
    kept: &mut Vec<CString>,
) {
    let cstring = CString::new(text).unwrap_or_else(|_| CString::new("").unwrap());
    let _ = AppendMenuA(menu, flags, id as usize, PCSTR(cstring.as_ptr().cast()));
    kept.push(cstring);
}

unsafe fn append_popup(menu: HMENU, child: HMENU, text: &str, kept: &mut Vec<CString>) {
    let cstring = CString::new(text).unwrap_or_else(|_| CString::new("").unwrap());
    let _ = AppendMenuA(
        menu,
        MF_POPUP,
        child.0 as usize,
        PCSTR(cstring.as_ptr().cast()),
    );
    kept.push(cstring);
}

unsafe fn add_tray(hwnd: HWND) {
    let icon = APP.with(|app| {
        if app.borrow().icon_small.0.is_null() {
            LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
        } else {
            app.borrow().icon_small
        }
    });
    let mut data = NOTIFYICONDATAA {
        cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: TRAY_MSG,
        hIcon: icon,
        ..Default::default()
    };
    let tip = b"EvenTide\0";
    for (slot, byte) in data.szTip.iter_mut().zip(tip) {
        *slot = *byte as i8;
    }
    let _ = Shell_NotifyIconA(NIM_ADD, &data);
}

unsafe fn remove_tray(hwnd: HWND) {
    let data = NOTIFYICONDATAA {
        cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    };
    let _ = Shell_NotifyIconA(NIM_DELETE, &data);
}

fn poll_dim_clock() {
    let sun_up = current_schedule().status == SunStatus::Up;
    APP.with(|app| app.borrow_mut().last_sun_up = Some(sun_up));
}

fn dim_moment(planned: &Schedule, local: &UtcTime) -> DimMoment {
    DimMoment {
        minute_of_day: (local.hour * 60 + local.minute) as i32,
        altitude_deg: planned.altitude_deg,
        morning: planned.solar_hours < 12.0,
        sunset_minute: planned.sunset_minute,
        sunrise_minute: planned.sunrise_minute,
    }
}

fn apply_preset(app: &mut App, preset: &CustomPreset) {
    let mut settings = app.settings.clone();
    let mut dim = app.dim.clone();
    let mut color = app.color.clone();
    eventide::apply_custom(&mut settings, &mut dim, &mut color, preset);
    app.settings = settings;
    app.dim = dim;
    app.color = color;
    app.dim_override = DimOverride::Schedule;
    app.preset_choice = Some(PresetChoice::Custom(preset.name.clone()));
}

fn current_preset_label(app: &App) -> String {
    preset_label(
        app.settings.day_kelvin,
        app.settings.night_kelvin,
        app.settings.late_kelvin,
        &app.customs,
        app.preset_choice.as_ref(),
    )
}

fn dimming_label(enabled: bool) -> &'static str {
    if enabled {
        "Dimming On"
    } else {
        "Dimming Off"
    }
}

fn toggle_dim_button(hwnd: HWND) {
    let moment = dim_moment(&current_schedule(), &local_time());
    APP.with(|app| {
        let mut app = app.borrow_mut();
        let enabled = dimming_enabled(app.dim_schedule, moment, app.dim_override);
        app.dim_override = toggled_dim_override(enabled);
    });
    unsafe {
        sync_lenses();
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

fn current_schedule() -> Schedule {
    let settings = APP.with(|app| app.borrow().settings.color_settings());
    schedule(local_time(), settings, utc_offset_minutes())
}

fn current_lens_alpha() -> u8 {
    let moment = dim_moment(&current_schedule(), &local_time());
    APP.with(|app| {
        let app = app.borrow();
        let level = level_from_saved_index(app.settings.brightness_index as i32);
        lens_alpha(app.dim_schedule, moment, app.dim_override, level)
    })
}

unsafe fn sync_lenses() {
    let alpha = current_lens_alpha();
    APP.with(|app| app.borrow_mut().last_lens_alpha = Some(alpha));
    if alpha == 0 {
        let hwnds = APP.with(|app| {
            app.borrow()
                .monitors
                .iter()
                .map(|monitor| monitor.lens)
                .collect::<Vec<_>>()
        });
        for hwnd in hwnds {
            if !hwnd.0.is_null() {
                let _ = DestroyWindow(hwnd);
            }
        }
        APP.with(|app| {
            for monitor in &mut app.borrow_mut().monitors {
                monitor.lens = HWND::default();
            }
        });
        return;
    }
    let (level, owner, count) = APP.with(|app| {
        let app = app.borrow();
        (
            level_from_saved_index(app.settings.brightness_index as i32),
            app.owner,
            app.monitors.len(),
        )
    });
    let dim = APP.with(|app| app.borrow().dim.clone());
    let instance = GetModuleHandleA(None).unwrap_or_default();
    let plans: Vec<(usize, bool)> = APP.with(|app| {
        app.borrow()
            .monitors
            .iter()
            .enumerate()
            .map(|(index, monitor)| (index, lens_enabled(monitor.number, &dim, level, count)))
            .collect()
    });
    for (index, want) in plans {
        if !want {
            let hwnd = APP.with(|app| {
                app.borrow()
                    .monitors
                    .get(index)
                    .map(|monitor| monitor.lens)
                    .unwrap_or_default()
            });
            if !hwnd.0.is_null() {
                let _ = DestroyWindow(hwnd);
                APP.with(|app| {
                    if let Some(monitor) = app.borrow_mut().monitors.get_mut(index) {
                        monitor.lens = HWND::default();
                    }
                });
            }
            continue;
        }
        let (mut hwnd, rect) = APP.with(|app| {
            let monitor = &app.borrow().monitors[index];
            (monitor.lens, monitor.rect)
        });
        if hwnd.0.is_null() {
            hwnd = CreateWindowExA(
                WINDOW_EX_STYLE(LENS_EXTENDED_STYLE),
                leak_cstr(CLASS_LENS),
                leak_cstr(APP_NAME),
                WS_POPUP | WS_VISIBLE,
                rect.left,
                rect.top,
                (rect.right - rect.left).max(1),
                (rect.bottom - rect.top).max(1),
                Some(owner),
                None,
                Some(instance.into()),
                None,
            )
            .unwrap_or_default();
            APP.with(|app| {
                if let Some(monitor) = app.borrow_mut().monitors.get_mut(index) {
                    monitor.lens = hwnd;
                }
            });
        }
        if hwnd.0.is_null() {
            continue;
        }
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            rect.left,
            rect.top,
            (rect.right - rect.left).max(1),
            (rect.bottom - rect.top).max(1),
            SWP_NOACTIVATE,
        );
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

unsafe fn raise_lenses() {
    let hwnds = APP.with(|app| {
        app.borrow()
            .monitors
            .iter()
            .map(|monitor| monitor.lens)
            .filter(|hwnd| !hwnd.0.is_null())
            .collect::<Vec<_>>()
    });
    let city = city_hwnd();
    if !city.0.is_null() {
        let flags = SET_WINDOW_POS_FLAGS(LENS_RAISE_FLAGS) | SWP_NOREDRAW;
        for hwnd in hwnds {
            let _ = SetWindowPos(hwnd, Some(city), 0, 0, 0, 0, flags);
        }
        return;
    }
    for hwnd in hwnds {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SET_WINDOW_POS_FLAGS(LENS_RAISE_FLAGS),
        );
    }
}

unsafe fn destroy_lenses() {
    let hwnds = APP.with(|app| {
        app.borrow()
            .monitors
            .iter()
            .map(|monitor| monitor.lens)
            .collect::<Vec<_>>()
    });
    for hwnd in hwnds {
        if !hwnd.0.is_null() {
            let _ = DestroyWindow(hwnd);
        }
    }
}

unsafe extern "system" fn lens_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_PAINT {
        let mut paint = PAINTSTRUCT::default();
        let dc = BeginPaint(hwnd, &mut paint);
        if !dc.is_invalid() {
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            fill(dc, rect, LENS_FILL);
        }
        let _ = EndPaint(hwnd, &paint);
        return LRESULT(1);
    }
    DefWindowProcA(hwnd, msg, wparam, lparam)
}

fn apply_gamma() -> bool {
    let planned = current_schedule();
    let (held, day, night, color) = APP.with(|app| {
        let app = app.borrow();
        let held = if app.dragging {
            Some(app.slider_thumb)
        } else {
            None
        };
        (
            held,
            app.settings.day_kelvin,
            app.settings.night_kelvin,
            app.color.clone(),
        )
    });
    let sun_up = planned.status == SunStatus::Up;
    let monitors = APP.with(|app| {
        app.borrow()
            .monitors
            .iter()
            .map(|monitor| (monitor.device_key.clone(), monitor.number))
            .collect::<Vec<_>>()
    });
    let mut changed = false;
    for device in unsafe { enum_attached_devices() } {
        let number = monitors
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(&device))
            .and_then(|(_, number)| *number);
        let on = match number {
            Some(number) => color.contains(number),
            None => matches!(color, MonitorSet::All),
        };
        let ramp = scheduled_ramp(monitor_kelvin(on, sun_up, day, night, held));
        let same = APP.with(|app| {
            app.borrow()
                .last_ramps
                .iter()
                .any(|(key, previous)| key.eq_ignore_ascii_case(&device) && *previous == ramp)
        });
        if same {
            continue;
        }
        if unsafe { set_device_ramp(&device, &ramp) } {
            changed = true;
            APP.with(|app| {
                let mut app = app.borrow_mut();
                if let Some(slot) = app
                    .last_ramps
                    .iter_mut()
                    .find(|(key, _)| key.eq_ignore_ascii_case(&device))
                {
                    slot.1 = ramp;
                } else {
                    app.last_ramps.push((device, ramp));
                }
            });
        }
    }
    changed
}

fn restore_identity_gamma() {
    let identity = identity_ramp();
    for device in unsafe { enum_attached_devices() } {
        unsafe {
            let _ = set_device_ramp(&device, &identity);
        }
    }
    APP.with(|app| app.borrow_mut().last_ramps.clear());
}

unsafe fn set_device_ramp(device: &str, ramp: &[[u16; 256]; 3]) -> bool {
    let wide = wide_z(device);
    let dc = CreateDCW(
        windows::core::w!("DISPLAY"),
        PCWSTR(wide.as_ptr()),
        None,
        None,
    );
    if dc.is_invalid() {
        return false;
    }
    let gamma = GAMMARAMP {
        Red: ramp[0],
        Green: ramp[1],
        Blue: ramp[2],
    };
    let ok = SetDeviceGammaRamp(dc, &gamma as *const GAMMARAMP as *const _).as_bool();
    let _ = DeleteDC(dc);
    ok
}

unsafe fn enum_attached_devices() -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0u32;
    loop {
        let mut device = DISPLAY_DEVICEW::default();
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        if !EnumDisplayDevicesW(None, index, &mut device, 0).as_bool() {
            break;
        }
        index += 1;
        if device.StateFlags.0 & 0x1 == 0 || device.StateFlags.0 & 0x8 != 0 {
            continue;
        }
        names.push(wide_string(&device.DeviceName));
    }
    names
}

fn refresh_monitors_if_changed() {
    let fresh = unsafe { collect_monitors() };
    let changed = APP.with(|app| {
        let app = app.borrow();
        app.monitors.len() != fresh.len()
            || app.monitors.iter().zip(fresh.iter()).any(|(old, new)| {
                old.device_key != new.device_key
                    || old.rect.left != new.rect.left
                    || old.rect.top != new.rect.top
                    || old.rect.right != new.rect.right
                    || old.rect.bottom != new.rect.bottom
                    || old.number != new.number
            })
    });
    if !changed {
        return;
    }
    unsafe {
        destroy_lenses();
    }
    APP.with(|app| app.borrow_mut().monitors = fresh);
    unsafe {
        sync_lenses();
        let hwnd = APP.with(|app| app.borrow().hwnd);
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

fn refresh_monitors() {
    let monitors = unsafe { collect_monitors() };
    APP.with(|app| app.borrow_mut().monitors = monitors);
}

unsafe fn collect_monitors() -> Vec<LiveMonitor> {
    let mut raw: Vec<RawMonitor> = Vec::new();
    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(monitor_enum),
        LPARAM(&mut raw as *mut Vec<RawMonitor> as isize),
    );
    let sources = query_sources();
    let primary = raw
        .iter()
        .find(|monitor| monitor.primary)
        .and_then(|monitor| gdi_index_from_device_name(&monitor.device));
    raw.into_iter()
        .map(|monitor| {
            let matched = sources
                .iter()
                .find(|(name, _, _)| name.eq_ignore_ascii_case(&monitor.device))
                .map(|(name, _, _)| name.clone())
                .or_else(|| {
                    sources
                        .iter()
                        .min_by_key(|(_, x, y)| {
                            let dx = i64::from(*x - monitor.rect.left);
                            let dy = i64::from(*y - monitor.rect.top);
                            dx * dx + dy * dy
                        })
                        .map(|(name, _, _)| name.clone())
                });
            let device_key = matched.unwrap_or(monitor.device);
            let number = settings_number(gdi_index_from_device_name(&device_key), primary);
            LiveMonitor {
                number,
                device_key,
                rect: monitor.rect,
                lens: HWND::default(),
            }
        })
        .collect()
}

struct RawMonitor {
    device: String,
    rect: RECT,
    primary: bool,
}

unsafe extern "system" fn monitor_enum(
    monitor: HMONITOR,
    _dc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> windows::core::BOOL {
    let list = &mut *(data.0 as *mut Vec<RawMonitor>);
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if GetMonitorInfoW(
        monitor,
        &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
    )
    .as_bool()
    {
        list.push(RawMonitor {
            device: wide_string(&info.szDevice),
            rect: info.monitorInfo.rcMonitor,
            primary: info.monitorInfo.dwFlags & 1 != 0,
        });
    }
    windows::core::BOOL(1)
}

unsafe fn query_sources() -> Vec<(String, i32, i32)> {
    let mut path_count = 0u32;
    let mut mode_count = 0u32;
    if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
        != WIN32_ERROR(0)
    {
        return Vec::new();
    }
    let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
    let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
    let status = QueryDisplayConfig(
        QDC_ONLY_ACTIVE_PATHS,
        &mut path_count,
        paths.as_mut_ptr(),
        &mut mode_count,
        modes.as_mut_ptr(),
        None,
    );
    if status != WIN32_ERROR(0) {
        return Vec::new();
    }
    paths.truncate(path_count as usize);
    modes.truncate(mode_count as usize);
    let mut sources = Vec::new();
    for path in &paths {
        let mut request = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
        request.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        request.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
        request.header.adapterId = path.sourceInfo.adapterId;
        request.header.id = path.sourceInfo.id;
        if DisplayConfigGetDeviceInfo(&mut request.header) != 0 {
            continue;
        }
        let name = wide_string(&request.viewGdiDeviceName);
        let mode_index = path.sourceInfo.Anonymous.modeInfoIdx as usize;
        let index = if mode_index < modes.len() {
            mode_index
        } else {
            mode_index & 0xFFFF
        };
        let (x, y) = if index < modes.len()
            && modes[index].infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE
        {
            let source = modes[index].Anonymous.sourceMode;
            (source.position.x, source.position.y)
        } else {
            (0, 0)
        };
        sources.push((name, x, y));
    }
    sources
}

fn wide_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|unit| *unit == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

fn wide_z(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn local_time() -> UtcTime {
    unsafe {
        let now = GetLocalTime();
        UtcTime {
            year: i32::from(now.wYear),
            month: u32::from(now.wMonth),
            day: u32::from(now.wDay),
            hour: u32::from(now.wHour),
            minute: u32::from(now.wMinute),
            second: u32::from(now.wSecond),
        }
    }
}

fn utc_offset_minutes() -> i32 {
    unsafe {
        let mut info = windows::Win32::System::Time::TIME_ZONE_INFORMATION::default();
        let kind = GetTimeZoneInformation(&mut info);
        let mut bias = info.Bias;
        if kind == 2 {
            bias += info.DaylightBias;
        } else if kind == 1 {
            bias += info.StandardBias;
        }
        -bias
    }
}

fn layout_report() -> String {
    let monitors = unsafe { collect_monitors() };
    let mut lines = Vec::new();
    for monitor in monitors {
        lines.push(format!(
            "{} {} {},{} {},{}",
            tile_label(monitor.number),
            monitor.device_key,
            monitor.rect.left,
            monitor.rect.top,
            monitor.rect.right,
            monitor.rect.bottom
        ));
    }
    lines.join("\n")
}

fn load_state() {
    let (settings, auto, mode) = unsafe { load_settings() };
    let _ = ensure_config_files();
    let (customs, presets_writable, preset_warning) = match preset_path() {
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(text) => match parse_preset_file(&text) {
                Ok(presets) => (presets, true, false),
                Err(_) => (Vec::new(), false, true),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Vec::new(), true, false),
            Err(_) => (Vec::new(), false, true),
        },
        None => (Vec::new(), false, true),
    };
    let (dim, color, monitors_writable, monitor_warning) = match monitor_path() {
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(text) => match parse_monitor_file(&text) {
                Ok((dim, color)) => (dim, color, true, false),
                Err(_) => (MonitorSet::All, MonitorSet::All, false, true),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (MonitorSet::All, MonitorSet::All, true, false)
            }
            Err(_) => (MonitorSet::All, MonitorSet::All, false, true),
        },
        None => (MonitorSet::All, MonitorSet::All, false, true),
    };
    APP.with(|app| {
        let mut app = app.borrow_mut();
        app.settings = settings;
        app.customs = customs;
        app.presets_writable = presets_writable;
        app.preset_warning = preset_warning;
        app.dim = dim;
        app.color = color;
        app.dim_schedule = auto;
        app.dim_override = mode;
        app.monitors_writable = monitors_writable;
        app.monitor_warning = monitor_warning;
    });
}

unsafe fn load_settings() -> (LensSettings, DimSchedule, DimOverride) {
    let mut settings = LensSettings::seed();
    let mut follows = false;
    let mut saw_follows = false;
    let mut dim_minute: Option<u32> = None;
    let mut transition_at_sunset: Option<u32> = None;
    let mut transition_minute: Option<u32> = None;
    let mut revert_at_sunrise: Option<u32> = None;
    let mut revert_minute: Option<u32> = None;
    let mut mode = DimOverride::Schedule;
    let Some(key) = open_settings_key() else {
        return (settings, DimSchedule::solar(), mode);
    };
    if let Some(value) = read_dword(key, "BrightnessIndex") {
        let index = value as i32;
        settings.brightness_index = if (0..9).contains(&index) {
            index as u32
        } else {
            3
        };
    }
    if let Some(value) = read_dword(key, "DayKelvin") {
        settings.day_kelvin = value as i32;
    }
    if let Some(value) = read_dword(key, "NightKelvin") {
        settings.night_kelvin = value as i32;
    }
    if let Some(value) = read_dword(key, "LateKelvin") {
        settings.late_kelvin = value as i32;
    }
    if let Some(value) = read_dword(key, "WakeMinute") {
        settings.wake_minute = (value as i32).rem_euclid(24 * 60);
    }
    if let Some(value) = read_dword(key, "Latitude") {
        settings.latitude_hundredths = value as i32;
    }
    if let Some(value) = read_dword(key, "Longitude") {
        settings.longitude_hundredths = value as i32;
    }
    if let Some(city) = read_sz(key, "City") {
        settings.city = canonical_city(&city);
    }
    if let Some(value) = read_dword(key, "DimFollowsSchedule") {
        follows = value != 0;
        saw_follows = true;
    }
    if let Some(value) = read_dword(key, "DimMinute") {
        dim_minute = Some(value);
    }
    if let Some(value) = read_dword(key, "TransitionAtSunset") {
        transition_at_sunset = Some(value);
    }
    if let Some(value) = read_dword(key, "TransitionMinute") {
        transition_minute = Some(value);
    }
    if let Some(value) = read_dword(key, "RevertAtSunrise") {
        revert_at_sunrise = Some(value);
    }
    if let Some(value) = read_dword(key, "RevertMinute") {
        revert_minute = Some(value);
    }
    let dim_with_schedule = read_dword(key, "DimWithSchedule");
    if let Some(value) = read_dword(key, "DimOverride") {
        mode = dim_override_from_saved(value);
    }
    unsafe {
        let _ = RegCloseKey(key);
    }
    let mut schedule = if transition_at_sunset.is_some() || revert_at_sunrise.is_some() {
        let mut schedule = DimSchedule::solar();
        if let Some(value) = transition_at_sunset {
            schedule.transition_at_sunset = value != 0;
        }
        if let Some(value) = transition_minute {
            if value < 24 * 60 {
                schedule.transition_minute = value as i32;
            }
        }
        if let Some(value) = revert_at_sunrise {
            schedule.revert_at_sunrise = value != 0;
        }
        if let Some(value) = revert_minute {
            if value < 24 * 60 {
                schedule.revert_minute = value as i32;
            }
        }
        schedule
    } else if saw_follows || dim_minute.is_some() {
        dim_schedule_from_legacy(follows, dim_minute)
    } else {
        DimSchedule::solar()
    };
    if let Some(value) = dim_with_schedule {
        schedule.dim_with_schedule = value != 0;
    }
    (settings, schedule, mode)
}

fn save_state() {
    let started = APP.with(|app| app.borrow().started);
    if !started {
        return;
    }
    let snapshot = APP.with(|app| {
        let app = app.borrow();
        (
            app.settings.clone(),
            app.customs.clone(),
            app.dim.clone(),
            app.color.clone(),
            app.dim_schedule,
            app.dim_override,
            app.presets_writable,
            app.monitors_writable,
        )
    });
    let (settings, customs, dim, color, auto, mode, presets_writable, monitors_writable) = snapshot;
    let _ = write_settings(&settings, auto, mode);
    if presets_writable || monitors_writable {
        if let Some(dir) = config_dir() {
            let _ = std::fs::create_dir_all(&dir);
        }
    }
    if presets_writable {
        if let Some(path) = preset_path() {
            let _ = std::fs::write(path, presets_to_json(&customs));
        }
    }
    if monitors_writable {
        if let Some(path) = monitor_path() {
            let _ = std::fs::write(path, monitors_to_json(&dim, &color));
        }
    }
}

fn write_settings(
    settings: &LensSettings,
    schedule: DimSchedule,
    mode: DimOverride,
) -> windows::core::Result<()> {
    unsafe {
        let key = create_key()?;
        write_dword(key, "BrightnessIndex", settings.brightness_index)?;
        write_dword(key, "DayKelvin", settings.day_kelvin as u32)?;
        write_dword(key, "NightKelvin", settings.night_kelvin as u32)?;
        write_dword(key, "LateKelvin", settings.late_kelvin as u32)?;
        write_dword(key, "WakeMinute", settings.wake_minute as u32)?;
        write_dword(key, "Latitude", settings.latitude_hundredths as u32)?;
        write_dword(key, "Longitude", settings.longitude_hundredths as u32)?;
        write_sz(key, "City", &settings.city)?;
        write_dword(
            key,
            "TransitionAtSunset",
            u32::from(schedule.transition_at_sunset),
        )?;
        write_dword(
            key,
            "TransitionMinute",
            schedule.transition_minute.rem_euclid(24 * 60) as u32,
        )?;
        write_dword(
            key,
            "RevertAtSunrise",
            u32::from(schedule.revert_at_sunrise),
        )?;
        write_dword(
            key,
            "RevertMinute",
            schedule.revert_minute.rem_euclid(24 * 60) as u32,
        )?;
        write_dword(
            key,
            "DimWithSchedule",
            u32::from(schedule.dim_with_schedule),
        )?;
        write_dword(
            key,
            "DimFollowsSchedule",
            u32::from(schedule.transition_at_sunset && schedule.revert_at_sunrise),
        )?;
        let minute = if schedule.transition_at_sunset {
            u32::MAX
        } else {
            schedule.transition_minute.rem_euclid(24 * 60) as u32
        };
        write_dword(key, "DimMinute", minute)?;
        write_dword(key, "DimOverride", dim_override_to_saved(mode))?;
        let _ = RegCloseKey(key);
        Ok(())
    }
}

unsafe fn open_settings_key() -> Option<HKEY> {
    for path in [REG_KEY, PRIOR_REG_KEY, EARLIER_REG_KEY, LEGACY_REG_KEY] {
        if let Some(key) = open_key(path, KEY_READ.0) {
            if read_dword(key, "BrightnessIndex").is_some() || read_sz(key, "City").is_some() {
                return Some(key);
            }
            let _ = RegCloseKey(key);
        }
    }
    None
}

unsafe fn open_key(path: &str, access: u32) -> Option<HKEY> {
    let mut key = HKEY::default();
    let path = CString::new(path).ok()?;
    let status = RegOpenKeyExA(
        HKEY_CURRENT_USER,
        PCSTR(path.as_ptr().cast()),
        Some(0),
        windows::Win32::System::Registry::REG_SAM_FLAGS(access),
        &mut key,
    );
    if status.is_ok() {
        Some(key)
    } else {
        None
    }
}

unsafe fn create_key() -> windows::core::Result<HKEY> {
    let mut key = HKEY::default();
    let mut disposition = windows::Win32::System::Registry::REG_CREATE_KEY_DISPOSITION::default();
    let path = CString::new(REG_KEY).map_err(|_| windows::core::Error::from_win32())?;
    RegCreateKeyExA(
        HKEY_CURRENT_USER,
        PCSTR(path.as_ptr().cast()),
        Some(0),
        None,
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        None,
        &mut key,
        Some(&mut disposition),
    )
    .ok()?;
    Ok(key)
}

unsafe fn read_dword(key: HKEY, name: &str) -> Option<u32> {
    let mut value = 0u32;
    let mut size = 4u32;
    let mut kind = REG_VALUE_TYPE::default();
    let name = CString::new(name).ok()?;
    let status = RegQueryValueExA(
        key,
        PCSTR(name.as_ptr().cast()),
        None,
        Some(&mut kind),
        Some((&mut value as *mut u32).cast()),
        Some(&mut size),
    );
    if status.is_ok() && kind == REG_DWORD && size == 4 {
        Some(value)
    } else {
        None
    }
}

unsafe fn read_sz(key: HKEY, name: &str) -> Option<String> {
    let mut buf = [0u8; 512];
    let mut size = buf.len() as u32;
    let mut kind = REG_VALUE_TYPE::default();
    let name = CString::new(name).ok()?;
    let status = RegQueryValueExA(
        key,
        PCSTR(name.as_ptr().cast()),
        None,
        Some(&mut kind),
        Some(buf.as_mut_ptr()),
        Some(&mut size),
    );
    if status.is_err() || kind != REG_SZ {
        return None;
    }
    let n = (size as usize).min(buf.len());
    let bytes = &buf[..n];
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    Some(String::from_utf8_lossy(&bytes[..end]).to_string())
}

unsafe fn write_dword(key: HKEY, name: &str, value: u32) -> windows::core::Result<()> {
    let name = CString::new(name).map_err(|_| windows::core::Error::from_win32())?;
    let bytes = value.to_le_bytes();
    RegSetValueExA(
        key,
        PCSTR(name.as_ptr().cast()),
        Some(0),
        REG_DWORD,
        Some(&bytes),
    )
    .ok()?;
    Ok(())
}

unsafe fn write_sz(key: HKEY, name: &str, value: &str) -> windows::core::Result<()> {
    let name = CString::new(name).map_err(|_| windows::core::Error::from_win32())?;
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    RegSetValueExA(
        key,
        PCSTR(name.as_ptr().cast()),
        Some(0),
        REG_SZ,
        Some(&bytes),
    )
    .ok()?;
    Ok(())
}

fn open_save(parent: HWND) {
    let open = APP.with(|app| {
        app.borrow()
            .save
            .as_ref()
            .is_some_and(|save| !save.hwnd.0.is_null())
    });
    if open {
        return;
    }
    let selected = APP.with(|app| {
        let app = app.borrow();
        let label = current_preset_label(&app);
        app.customs.iter().position(|preset| preset.name == label)
    });
    unsafe {
        let instance = GetModuleHandleA(None).unwrap_or_default();
        APP.with(|app| {
            app.borrow_mut().save = Some(SaveDialog {
                hwnd: HWND::default(),
                edit: HWND::default(),
                selected,
            });
        });
        let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_VISIBLE;
        let mut bounds = RECT {
            left: 0,
            top: 0,
            right: 468,
            bottom: save_client_height(),
        };
        let _ =
            windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        let (x, y) = popup_origin(parent, width, height);
        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE(0x101),
            leak_cstr(CLASS_SAVE),
            leak_cstr("Save preset"),
            style,
            x,
            y,
            width,
            height,
            Some(parent),
            None,
            Some(instance.into()),
            None,
        );
        let Ok(hwnd) = hwnd else {
            APP.with(|app| app.borrow_mut().save = None);
            return;
        };
        let _ = EnableWindow(parent, false);
        APP.with(|app| {
            if let Some(save) = app.borrow_mut().save.as_mut() {
                save.hwnd = hwnd;
            }
        });
    }
}

fn save_name_is_new() -> bool {
    APP.with(|app| {
        app.borrow()
            .save
            .as_ref()
            .is_some_and(|save| save.selected.is_none())
    })
}

fn save_name_rect() -> RECT {
    RECT {
        left: 24,
        top: 40,
        right: 444,
        bottom: 76,
    }
}
fn save_edit_rect() -> RECT {
    RECT {
        left: 24,
        top: 84,
        right: 444,
        bottom: 120,
    }
}

fn save_buttons_top() -> i32 {
    if save_name_is_new() {
        136
    } else {
        92
    }
}

fn save_delete_rect() -> RECT {
    let top = save_buttons_top();
    RECT {
        left: 24,
        top,
        right: 116,
        bottom: top + 36,
    }
}

fn save_cancel_rect() -> RECT {
    let top = save_buttons_top();
    RECT {
        left: 248,
        top,
        right: 340,
        bottom: top + 36,
    }
}

fn save_ok_rect() -> RECT {
    let top = save_buttons_top();
    RECT {
        left: 352,
        top,
        right: 444,
        bottom: top + 36,
    }
}

fn save_client_height() -> i32 {
    save_buttons_top() + 52
}

unsafe extern "system" fn save_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let instance = GetModuleHandleA(None).unwrap_or_default();
            let editing = save_name_is_new();
            let field = save_edit_rect();
            let (edit_x, edit_y, edit_w, edit_h) = if editing {
                (
                    field.left + 12,
                    field.top + 8,
                    field.right - field.left - 24,
                    field.bottom - field.top - 16,
                )
            } else {
                (0, 0, 0, 0)
            };
            let mut style = WS_CHILD | WINDOW_STYLE(0x80);
            if editing {
                style |= WS_VISIBLE;
            }
            let edit = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                s!("EDIT"),
                s!(""),
                style,
                edit_x,
                edit_y,
                edit_w,
                edit_h,
                Some(hwnd),
                Some(HMENU(1001isize as _)),
                Some(instance.into()),
                None,
            )
            .unwrap_or_default();
            let font = APP.with(|app| app.borrow().font_body);
            if !font.is_invalid() {
                let _ = SendMessageW(
                    edit,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
            APP.with(|app| {
                if let Some(save) = app.borrow_mut().save.as_mut() {
                    save.hwnd = hwnd;
                    save.edit = edit;
                }
            });
            if editing && !edit.0.is_null() {
                let _ = SetFocus(Some(edit));
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u32;
            if id != 1001 {
                save_menu_command(hwnd, id);
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            paint_save(hwnd);
            LRESULT(0)
        }
        WM_CTLCOLOREDIT => {
            let dc = HDC(wparam.0 as *mut core::ffi::c_void);
            let _ = SetTextColor(dc, COLORREF(INK));
            let _ = SetBkColor(dc, COLORREF(0x00FFFFFF));
            LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
        }
        WM_LBUTTONDOWN => {
            save_click(hwnd, lparam_point(lparam));
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            let parent = APP.with(|app| app.borrow().hwnd);
            APP.with(|app| app.borrow_mut().save = None);
            if !parent.0.is_null() {
                let _ = EnableWindow(parent, true);
                let _ = SetForegroundWindow(parent);
                let _ = InvalidateRect(Some(parent), None, false);
            }
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

fn save_click(hwnd: HWND, point: POINT) {
    if point_in(point, save_name_rect()) {
        unsafe {
            show_save_names(hwnd);
        }
        return;
    }
    if !save_name_is_new() && point_in(point, save_delete_rect()) {
        delete_selected_preset(hwnd);
        return;
    }
    if point_in(point, save_ok_rect()) {
        if commit_save() {
            unsafe {
                sync_lenses();
                apply_gamma();
                let _ = DestroyWindow(hwnd);
            }
        }
        return;
    }
    if point_in(point, save_cancel_rect()) {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
    }
}

fn save_menu_command(hwnd: HWND, id: u32) {
    if id == SAVE_NAME_NEW {
        choose_save_name(None);
    } else if id >= SAVE_NAME_PRESET {
        let index = (id - SAVE_NAME_PRESET) as usize;
        let count = APP.with(|app| app.borrow().customs.len());
        if index >= count {
            return;
        }
        choose_save_name(Some(index));
    } else {
        return;
    }
    unsafe {
        refresh_save_chrome(hwnd);
    }
}

fn choose_save_name(index: Option<usize>) {
    APP.with(|app| {
        if let Some(save) = app.borrow_mut().save.as_mut() {
            save.selected = index;
        }
    });
}

fn delete_selected_preset(hwnd: HWND) {
    let removed = APP.with(|app| {
        let mut app = app.borrow_mut();
        let index = app.save.as_ref().and_then(|save| save.selected);
        let Some(index) = index else {
            return false;
        };
        if index >= app.customs.len() {
            return false;
        }
        let removed = app.customs.remove(index);
        if app.preset_choice.as_ref() == Some(&PresetChoice::Custom(removed.name)) {
            app.preset_choice = None;
        }
        if let Some(save) = app.save.as_mut() {
            save.selected = None;
        }
        true
    });
    if !removed {
        return;
    }
    unsafe {
        refresh_save_chrome(hwnd);
        let parent = APP.with(|app| app.borrow().hwnd);
        if !parent.0.is_null() {
            let _ = InvalidateRect(Some(parent), None, false);
        }
    }
}

unsafe fn show_save_names(hwnd: HWND) {
    let mut kept = Vec::new();
    let menu = CreatePopupMenu().unwrap_or_default();
    let (selected, names) = APP.with(|app| {
        let app = app.borrow();
        let selected = app.save.as_ref().and_then(|save| save.selected);
        let names = app
            .customs
            .iter()
            .map(|preset| preset.name.clone())
            .collect::<Vec<_>>();
        (selected, names)
    });
    let new_flags = if selected.is_none() {
        MF_STRING | MF_CHECKED
    } else {
        MF_STRING
    };
    append(menu, new_flags, SAVE_NAME_NEW, "New", &mut kept);
    for (index, name) in names.iter().enumerate() {
        let flags = if selected == Some(index) {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING
        };
        append(
            menu,
            flags,
            SAVE_NAME_PRESET + index as u32,
            name,
            &mut kept,
        );
    }
    show_popup_at(hwnd, menu, save_name_rect());
    let _ = DestroyMenu(menu);
}

unsafe fn refresh_save_chrome(hwnd: HWND) {
    let edit = APP.with(|app| {
        app.borrow()
            .save
            .as_ref()
            .map(|save| save.edit)
            .unwrap_or_default()
    });
    if !edit.0.is_null() {
        let editing = save_name_is_new();
        let field = save_edit_rect();
        let (x, y, w, h) = if editing {
            (
                field.left + 12,
                field.top + 8,
                (field.right - field.left - 24).max(1),
                (field.bottom - field.top - 16).max(1),
            )
        } else {
            (0, 0, 0, 0)
        };
        let _ = SetWindowPos(edit, None, x, y, w, h, SET_WINDOW_POS_FLAGS(0x14));
        let _ = ShowWindow(edit, if editing { SW_SHOW } else { SW_HIDE });
        if editing {
            let _ = SetFocus(Some(edit));
        }
    }
    let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_VISIBLE;
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: 468,
        bottom: save_client_height(),
    };
    let _ = windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
    let mut window = RECT::default();
    let _ = GetWindowRect(hwnd, &mut window);
    let _ = SetWindowPos(
        hwnd,
        None,
        window.left,
        window.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
        SET_WINDOW_POS_FLAGS(0x4),
    );
    let _ = InvalidateRect(Some(hwnd), None, false);
}

unsafe fn paint_save(hwnd: HWND) {
    let mut paint = PAINTSTRUCT::default();
    let window_dc = BeginPaint(hwnd, &mut paint);
    if window_dc.is_invalid() {
        return;
    }
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);
    let buffer = CreateCompatibleDC(Some(window_dc));
    let bitmap = CreateCompatibleBitmap(window_dc, client.right.max(1), client.bottom.max(1));
    let previous = SelectObject(buffer, bitmap.into());
    fill(buffer, client, BG);
    let _ = SetBkMode(buffer, TRANSPARENT);
    let (fonts, name, editing) = APP.with(|app| {
        let app = app.borrow();
        let name = match app.save.as_ref().and_then(|save| save.selected) {
            Some(index) => app
                .customs
                .get(index)
                .map(|preset| preset.name.clone())
                .unwrap_or_else(|| "New".to_string()),
            None => "New".to_string(),
        };
        let editing = app
            .save
            .as_ref()
            .is_some_and(|save| save.selected.is_none());
        ([app.font_body, app.font_small], name, editing)
    });
    text(buffer, fonts[1], MUTED, 24, 16, "Name");
    draw_save_name(buffer, fonts[0], save_name_rect(), &name);
    if editing {
        let field = save_edit_rect();
        let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
        let pen = CreatePen(PS_SOLID, 1, COLORREF(LINE));
        let old_brush = SelectObject(buffer, brush.into());
        let old_pen = SelectObject(buffer, pen.into());
        let _ = RoundRect(
            buffer,
            field.left,
            field.top,
            field.right,
            field.bottom,
            12,
            12,
        );
        let _ = SelectObject(buffer, old_brush);
        let _ = SelectObject(buffer, old_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
    if !editing {
        draw_command_button(buffer, fonts[0], save_delete_rect(), "Delete");
    }
    draw_command_button(buffer, fonts[0], save_cancel_rect(), "Cancel");
    draw_pill(buffer, fonts[0], screen_from(save_ok_rect()), "Save", true);
    let _ = BitBlt(
        window_dc,
        0,
        0,
        client.right,
        client.bottom,
        Some(buffer),
        0,
        0,
        SRCCOPY,
    );
    let _ = SelectObject(buffer, previous);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(buffer);
    let _ = EndPaint(hwnd, &paint);
}

unsafe fn draw_checkbox(dc: HDC, font: HFONT, mark: RECT, label: &str, on: bool) {
    let brush = CreateSolidBrush(COLORREF(if on { BLUE } else { 0x00FFFFFF }));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(if on { BLUE_DEEP } else { LINE }));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, mark.left, mark.top, mark.right, mark.bottom, 6, 6);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    if on {
        let pen = CreatePen(PS_SOLID, 2, COLORREF(0x00FFFFFF));
        let old = SelectObject(dc, pen.into());
        let _ = MoveToEx(dc, mark.left + 5, mark.top + 12, None);
        let _ = LineTo(dc, mark.left + 10, mark.bottom - 6);
        let _ = LineTo(dc, mark.right - 5, mark.top + 6);
        let _ = SelectObject(dc, old);
        let _ = DeleteObject(pen.into());
    }
    text(dc, font, INK, mark.right + 10, mark.top + 2, label);
}

fn screen_from(rect: RECT) -> ScreenRect {
    ScreenRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

fn title_ansi(text: &str) -> CString {
    CString::new(text).unwrap_or_else(|_| CString::new("").unwrap())
}

fn commit_save() -> bool {
    let selected = APP.with(|app| app.borrow().save.as_ref().and_then(|save| save.selected));
    let name = if let Some(index) = selected {
        APP.with(|app| {
            app.borrow()
                .customs
                .get(index)
                .map(|preset| preset.name.clone())
        })
        .unwrap_or_default()
    } else {
        unsafe { save_edit_text() }
    };
    if name.is_empty() {
        return false;
    }
    APP.with(|app| {
        let mut app = app.borrow_mut();
        let preset = CustomPreset {
            name: name.clone(),
            day_kelvin: app.settings.day_kelvin,
            night_kelvin: app.settings.night_kelvin,
            late_kelvin: app.settings.late_kelvin,
            brightness_index: app.settings.brightness_index,
            dim_monitors: app.dim.clone(),
            color_monitors: app.color.clone(),
        };
        if let Some(existing) = app.customs.iter_mut().find(|preset| preset.name == name) {
            *existing = preset;
        } else {
            app.customs.push(preset);
        }
        app.presets_writable = true;
        app.preset_warning = false;
    });
    true
}

unsafe fn save_edit_text() -> String {
    let edit = APP.with(|app| {
        app.borrow()
            .save
            .as_ref()
            .map(|save| save.edit)
            .unwrap_or_default()
    });
    let mut buf = [0u16; 128];
    let length = if edit.0.is_null() {
        0
    } else {
        GetWindowTextW(edit, &mut buf)
    };
    String::from_utf16_lossy(&buf[..length.max(0) as usize])
        .trim()
        .to_string()
}

unsafe fn draw_save_name(dc: HDC, font: HFONT, rect: RECT, label: &str) {
    let brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(LINE));
    let old_brush = SelectObject(dc, brush.into());
    let old_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(dc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);
    let _ = SelectObject(dc, old_brush);
    let _ = SelectObject(dc, old_pen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
    text(dc, font, INK, rect.left + 14, rect.top + 8, label);
    let pen = CreatePen(PS_SOLID, 1, COLORREF(MUTED));
    let old = SelectObject(dc, pen.into());
    let x = rect.right - 22;
    let y = rect.top + 16;
    let _ = MoveToEx(dc, x, y, None);
    let _ = LineTo(dc, x + 4, y + 4);
    let _ = LineTo(dc, x + 8, y);
    let _ = SelectObject(dc, old);
    let _ = DeleteObject(pen.into());
}

fn shell_icon(cx: i32, cy: i32) -> Option<HICON> {
    let cx = cx.max(1);
    let cy = cy.max(1);
    let need = cx.max(cy) as usize;
    let frames: &[(usize, &[u8])] = &[
        (16, ICON_16),
        (24, ICON_24),
        (48, ICON_48),
        (72, ICON_72),
        (256, ICON_256),
    ];
    let (side, pixels) = frames
        .iter()
        .find(|(side, _)| *side >= need)
        .copied()
        .unwrap_or((256, ICON_256));
    icon_from_bgra(pixels, side, cx, cy)
}

fn icon_from_bgra(pixels: &[u8], side: usize, cx: i32, cy: i32) -> Option<HICON> {
    if pixels.len() != side * side * 4 || side == 0 || side > 256 || cx <= 0 || cy <= 0 {
        return None;
    }
    let xor_bytes = side * side * 4;
    let mask_stride = (side + 31) / 32 * 4;
    let mask_bytes = mask_stride * side;
    let mut bytes = vec![0u8; 40 + xor_bytes + mask_bytes];
    bytes[0..4].copy_from_slice(&40u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&(side as i32).to_le_bytes());
    bytes[8..12].copy_from_slice(&((side as i32) * 2).to_le_bytes());
    bytes[12..14].copy_from_slice(&1u16.to_le_bytes());
    bytes[14..16].copy_from_slice(&32u16.to_le_bytes());
    let row = side * 4;
    for y in 0..side {
        let src = (side - 1 - y) * row;
        let dst = 40 + y * row;
        bytes[dst..dst + row].copy_from_slice(&pixels[src..src + row]);
    }
    unsafe { CreateIconFromResourceEx(&bytes, true, 0x0003_0000, cx, cy, IMAGE_FLAGS(0)).ok() }
}

fn open_time(parent: HWND) {
    let open = APP.with(|app| {
        app.borrow()
            .time
            .as_ref()
            .is_some_and(|time| !time.hwnd.0.is_null())
    });
    if open {
        return;
    }
    let draft = APP.with(|app| app.borrow().dim_schedule);
    unsafe {
        let instance = GetModuleHandleA(None).unwrap_or_default();
        APP.with(|app| {
            app.borrow_mut().time = Some(TimeDialog {
                hwnd: HWND::default(),
                draft,
            });
        });
        let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_VISIBLE;
        let mut bounds = RECT {
            left: 0,
            top: 0,
            right: 468,
            bottom: time_client_height(),
        };
        let _ =
            windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        let (x, y) = popup_origin(parent, width, height);
        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE(0x101),
            leak_cstr(CLASS_TIME),
            leak_cstr("Time"),
            style,
            x,
            y,
            width,
            height,
            Some(parent),
            None,
            Some(instance.into()),
            None,
        );
        let Ok(hwnd) = hwnd else {
            APP.with(|app| app.borrow_mut().time = None);
            return;
        };
        let _ = EnableWindow(parent, false);
        APP.with(|app| {
            if let Some(time) = app.borrow_mut().time.as_mut() {
                time.hwnd = hwnd;
            }
        });
    }
}

fn time_transition_revealed() -> bool {
    APP.with(|app| {
        app.borrow()
            .time
            .as_ref()
            .is_some_and(|time| !time.draft.transition_at_sunset)
    })
}

fn time_revert_revealed() -> bool {
    APP.with(|app| {
        app.borrow()
            .time
            .as_ref()
            .is_some_and(|time| !time.draft.revert_at_sunrise)
    })
}

fn time_block_height(revealed: bool) -> i32 {
    if revealed {
        36 + 52
    } else {
        36
    }
}

fn time_transition_top() -> i32 {
    20
}

fn time_revert_top() -> i32 {
    time_transition_top() + time_block_height(time_transition_revealed()) + 8
}

fn time_schedule_top() -> i32 {
    time_revert_top() + time_block_height(time_revert_revealed()) + 8
}

fn time_buttons_top() -> i32 {
    time_schedule_top() + 36 + 16
}

fn time_client_height() -> i32 {
    time_buttons_top() + 52
}

fn time_mark(top: i32) -> RECT {
    RECT {
        left: 24,
        top: top + 4,
        right: 48,
        bottom: top + 28,
    }
}

fn time_hit(top: i32) -> RECT {
    RECT {
        left: 24,
        top,
        right: 280,
        bottom: top + 36,
    }
}

fn time_clock_rect(top: i32) -> RECT {
    let top = top + 40;
    let left = 24;
    let width = time_clock_width() + 16;
    RECT {
        left,
        top,
        right: left + width,
        bottom: top + 44,
    }
}

fn time_earlier_rect(top: i32) -> RECT {
    let time = time_clock_rect(top);
    RECT {
        left: time.right + 8,
        top: time.top + 9,
        right: time.right + 34,
        bottom: time.bottom - 9,
    }
}

fn time_later_rect(top: i32) -> RECT {
    let earlier = time_earlier_rect(top);
    RECT {
        left: earlier.right + 8,
        top: earlier.top,
        right: earlier.right + 34,
        bottom: earlier.bottom,
    }
}

fn time_transition_clock_rect() -> RECT {
    time_clock_rect(time_transition_top())
}
fn time_revert_clock_rect() -> RECT {
    time_clock_rect(time_revert_top())
}

fn time_cancel_rect() -> RECT {
    let top = time_buttons_top();
    RECT {
        left: 248,
        top,
        right: 340,
        bottom: top + 36,
    }
}

fn time_ok_rect() -> RECT {
    let top = time_buttons_top();
    RECT {
        left: 352,
        top,
        right: 444,
        bottom: top + 36,
    }
}

unsafe extern "system" fn time_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            APP.with(|app| {
                if let Some(time) = app.borrow_mut().time.as_mut() {
                    time.hwnd = hwnd;
                }
            });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_CTLCOLOREDIT => {
            let dc = HDC(wparam.0 as *mut core::ffi::c_void);
            let _ = SetTextColor(dc, COLORREF(BLUE_DEEP));
            let _ = SetBkColor(dc, COLORREF(0x00FFFFFF));
            LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
        }
        WM_PAINT => {
            paint_time(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            time_click(hwnd, lparam_point(lparam));
            LRESULT(0)
        }
        WM_CLOSE => {
            finish_time_edit(true, true);
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            let parent = APP.with(|app| app.borrow().hwnd);
            APP.with(|app| {
                let mut app = app.borrow_mut();
                app.time_edit = HWND::default();
                app.time_field = None;
                app.time = None;
            });
            if !parent.0.is_null() {
                let _ = EnableWindow(parent, true);
                let _ = SetForegroundWindow(parent);
                let _ = InvalidateRect(Some(parent), None, false);
            }
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

fn time_click(hwnd: HWND, point: POINT) {
    let editing = APP.with(|app| !app.borrow().time_edit.0.is_null());
    if editing {
        let field = APP.with(|app| app.borrow().time_field);
        let current = match field {
            Some(TimeField::Transition) => time_transition_clock_rect(),
            Some(TimeField::Revert) => time_revert_clock_rect(),
            None => RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
        };
        if !point_in(point, current) {
            finish_time_edit(true, true);
        }
    }
    if time_transition_revealed() && point_in(point, time_transition_clock_rect()) {
        open_time_edit(hwnd, TimeField::Transition);
        return;
    }
    if time_revert_revealed() && point_in(point, time_revert_clock_rect()) {
        open_time_edit(hwnd, TimeField::Revert);
        return;
    }
    let id = if point_in(point, time_hit(time_transition_top())) {
        TIME_TRANSITION
    } else if time_transition_revealed()
        && point_in(point, time_earlier_rect(time_transition_top()))
    {
        TIME_TRANSITION_EARLIER
    } else if time_transition_revealed() && point_in(point, time_later_rect(time_transition_top()))
    {
        TIME_TRANSITION_LATER
    } else if point_in(point, time_hit(time_revert_top())) {
        TIME_REVERT
    } else if time_revert_revealed() && point_in(point, time_earlier_rect(time_revert_top())) {
        TIME_REVERT_EARLIER
    } else if time_revert_revealed() && point_in(point, time_later_rect(time_revert_top())) {
        TIME_REVERT_LATER
    } else if point_in(point, time_hit(time_schedule_top())) {
        TIME_SCHEDULE
    } else if point_in(point, time_ok_rect()) {
        TIME_OK
    } else if point_in(point, time_cancel_rect()) {
        TIME_CANCEL
    } else {
        return;
    };
    match id {
        TIME_TRANSITION | TIME_REVERT => {
            let field = if id == TIME_TRANSITION {
                TimeField::Transition
            } else {
                TimeField::Revert
            };
            let editing_field = APP.with(|app| app.borrow().time_field);
            if editing_field == Some(field) {
                finish_time_edit(true, true);
            }
            APP.with(|app| {
                if let Some(time) = app.borrow_mut().time.as_mut() {
                    match field {
                        TimeField::Transition => {
                            time.draft.transition_at_sunset = !time.draft.transition_at_sunset
                        }
                        TimeField::Revert => {
                            time.draft.revert_at_sunrise = !time.draft.revert_at_sunrise
                        }
                    }
                }
            });
            unsafe {
                resize_time_dialog(hwnd);
            }
        }
        TIME_SCHEDULE => {
            APP.with(|app| {
                if let Some(time) = app.borrow_mut().time.as_mut() {
                    time.draft.dim_with_schedule = !time.draft.dim_with_schedule;
                }
            });
            unsafe {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }
        TIME_TRANSITION_EARLIER
        | TIME_TRANSITION_LATER
        | TIME_REVERT_EARLIER
        | TIME_REVERT_LATER => {
            let steps = if id == TIME_TRANSITION_EARLIER || id == TIME_REVERT_EARLIER {
                -1
            } else {
                1
            };
            let field = if id == TIME_TRANSITION_EARLIER || id == TIME_TRANSITION_LATER {
                TimeField::Transition
            } else {
                TimeField::Revert
            };
            APP.with(|app| {
                if let Some(time) = app.borrow_mut().time.as_mut() {
                    match field {
                        TimeField::Transition => {
                            time.draft.transition_minute =
                                step_wake_minute(time.draft.transition_minute, steps);
                        }
                        TimeField::Revert => {
                            time.draft.revert_minute =
                                step_wake_minute(time.draft.revert_minute, steps);
                        }
                    }
                }
            });
            unsafe {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }
        TIME_OK => {
            finish_time_edit(true, true);
            if commit_time() {
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
            }
        }
        TIME_CANCEL => {
            finish_time_edit(false, true);
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }
        _ => {}
    }
}

fn commit_time() -> bool {
    let draft = APP.with(|app| app.borrow().time.as_ref().map(|time| time.draft));
    let Some(draft) = draft else { return false };
    let (settings, mode) = APP.with(|app| {
        let mut app = app.borrow_mut();
        app.dim_schedule = draft;
        app.dim_override = DimOverride::Schedule;
        (app.settings.clone(), app.dim_override)
    });
    let _ = write_settings(&settings, draft, mode);
    unsafe {
        sync_lenses();
        let hwnd = APP.with(|app| app.borrow().hwnd);
        if !hwnd.0.is_null() {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
    true
}

unsafe fn resize_time_dialog(hwnd: HWND) {
    let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_VISIBLE;
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: 468,
        bottom: time_client_height(),
    };
    let _ = windows::Win32::UI::WindowsAndMessaging::AdjustWindowRect(&mut bounds, style, false);
    let mut window = RECT::default();
    let _ = GetWindowRect(hwnd, &mut window);
    let _ = SetWindowPos(
        hwnd,
        None,
        window.left,
        window.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
        SET_WINDOW_POS_FLAGS(0x4),
    );
    let _ = InvalidateRect(Some(hwnd), None, false);
}

unsafe fn paint_time(hwnd: HWND) {
    let mut paint = PAINTSTRUCT::default();
    let window_dc = BeginPaint(hwnd, &mut paint);
    if window_dc.is_invalid() {
        return;
    }
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);
    let buffer = CreateCompatibleDC(Some(window_dc));
    let bitmap = CreateCompatibleBitmap(window_dc, client.right.max(1), client.bottom.max(1));
    let previous = SelectObject(buffer, bitmap.into());
    fill(buffer, client, BG);
    let _ = SetBkMode(buffer, TRANSPARENT);
    let (body, clock_font, draft, editing) = APP.with(|app| {
        let app = app.borrow();
        let draft = app
            .time
            .as_ref()
            .map(|time| time.draft)
            .unwrap_or_else(DimSchedule::solar);
        (app.font_body, app.font_wake, draft, app.time_field)
    });
    draw_checkbox(
        buffer,
        body,
        time_mark(time_transition_top()),
        "Transition at sunset",
        draft.transition_at_sunset,
    );
    if !draft.transition_at_sunset {
        paint_time_clock(
            buffer,
            body,
            clock_font,
            time_transition_top(),
            draft.transition_minute,
            editing == Some(TimeField::Transition),
        );
    }
    draw_checkbox(
        buffer,
        body,
        time_mark(time_revert_top()),
        "Revert at sunrise",
        draft.revert_at_sunrise,
    );
    if !draft.revert_at_sunrise {
        paint_time_clock(
            buffer,
            body,
            clock_font,
            time_revert_top(),
            draft.revert_minute,
            editing == Some(TimeField::Revert),
        );
    }
    draw_checkbox(
        buffer,
        body,
        time_mark(time_schedule_top()),
        "Dim with schedule",
        draft.dim_with_schedule,
    );
    draw_command_button(buffer, body, time_cancel_rect(), "Cancel");
    draw_pill(buffer, body, screen_from(time_ok_rect()), "Save", true);
    let _ = BitBlt(
        window_dc,
        0,
        0,
        client.right,
        client.bottom,
        Some(buffer),
        0,
        0,
        SRCCOPY,
    );
    let _ = SelectObject(buffer, previous);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(buffer);
    let _ = EndPaint(hwnd, &paint);
}

unsafe fn paint_time_clock(
    dc: HDC,
    body: HFONT,
    clock_font: HFONT,
    top: i32,
    minute: i32,
    editing: bool,
) {
    let time_box = time_clock_rect(top);
    if editing {
        fill(dc, time_box, 0x00FFFFFF);
        stroke_rect(dc, time_box, 0x00000000);
    } else {
        let clock = format_clock(minute);
        let size = text_size(dc, clock_font, &clock);
        text(
            dc,
            clock_font,
            BLUE_DEEP,
            time_box.left + (time_box.right - time_box.left - size.cx) / 2,
            time_box.top + (time_box.bottom - time_box.top - size.cy) / 2,
            &clock,
        );
    }
    draw_step_button(dc, body, time_earlier_rect(top), "−");
    draw_step_button(dc, body, time_later_rect(top), "+");
}
