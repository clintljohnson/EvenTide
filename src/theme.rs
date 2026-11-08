//! Colors for the EvenTide window.
//!
//! The palettes are plain data. The host decides [`ColorScheme`] from the
//! operating system, then painting reads [`palette`]. Nothing here calls Win32,
//! AppKit, or GTK.
//!
//! Windows reads `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`
//! `AppsUseLightTheme` (`0` is dark). A macOS port would read `AppleInterfaceStyle`
//! (`Dark` means dark). A Linux port would read the Freedesktop color-scheme
//! (`org.freedesktop.appearance`, `1` means dark) or GTK's
//! `gtk-application-prefer-dark-theme`.

/// Which palette the window is using.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

/// `COLORREF` values (`0x00BBGGRR`) for one appearance.
///
/// Light values are the original window. Dark values keep the same roles:
/// page, text, cards, buttons, the accent, and the day curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub bg: u32,
    pub ink: u32,
    pub accent: u32,
    pub accent_text: u32,
    pub muted: u32,
    pub tile: u32,
    pub tile_on: u32,
    pub line: u32,
    pub track: u32,
    pub status: u32,
    pub night_handle: u32,
    pub day_handle: u32,
    /// Buttons, fields, and the temperature bubble.
    pub surface: u32,
    /// Text and marks drawn on [`Self::accent`].
    pub on_accent: u32,
    /// Marks drawn on [`Self::ink`], such as an enabled temperature control.
    pub on_ink: u32,
    pub curve_lo: u32,
    pub curve_hi: u32,
    pub marker_ring: u32,
    pub menu_bg: u32,
    pub menu_text: u32,
    pub menu_hot: u32,
    pub menu_hot_text: u32,
}

const LIGHT: Palette = Palette {
    bg: 0x00F3_F1EF,
    ink: 0x0038_3332,
    accent: 0x00C4_9172,
    accent_text: 0x0094_624D,
    muted: 0x0098_8880,
    tile: 0x00FF_FFFF,
    tile_on: 0x00E4_DDD6,
    line: 0x00D5_D0CB,
    track: 0x0080_0000,
    status: 0x0080_0000,
    night_handle: 0x0028_8CF2,
    day_handle: 0x00F0_C67E,
    surface: 0x00FF_FFFF,
    on_accent: 0x00FF_FFFF,
    on_ink: 0x00FF_FFFF,
    curve_lo: 0x0040_2828,
    curve_hi: 0x0064_B4F0,
    marker_ring: 0x00FF_FFFF,
    menu_bg: 0x00F0_F0F0,
    menu_text: 0x0000_0000,
    menu_hot: 0x00C4_9172,
    menu_hot_text: 0x00FF_FFFF,
};

const DARK: Palette = Palette {
    bg: 0x001E_1C1A,
    ink: 0x00E4_E2DE,
    accent: 0x00C4_9172,
    accent_text: 0x00D4_B896,
    muted: 0x009C_9890,
    tile: 0x0054_4A42,
    tile_on: 0x003A_332C,
    line: 0x0052_4C48,
    track: 0x00C4_783A,
    status: 0x00E2_B48C,
    night_handle: 0x0028_8CF2,
    day_handle: 0x00F0_C67E,
    surface: 0x0029_2624,
    on_accent: 0x00FF_FFFF,
    on_ink: 0x001E_1C1A,
    curve_lo: 0x0098_8870,
    curve_hi: 0x0064_B4F0,
    marker_ring: 0x00F4_F2F0,
    menu_bg: 0x0029_2624,
    menu_text: 0x00E4_E2DE,
    menu_hot: 0x00C4_9172,
    menu_hot_text: 0x00FF_FFFF,
};

pub fn palette(scheme: ColorScheme) -> Palette {
    match scheme {
        ColorScheme::Light => LIGHT,
        ColorScheme::Dark => DARK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(color: u32) -> u32 {
        let red = color & 0xFF;
        let green = (color >> 8) & 0xFF;
        let blue = (color >> 16) & 0xFF;
        red * 3 + green * 6 + blue
    }

    #[test]
    fn light_palette_keeps_the_original_window_colors() {
        let colors = palette(ColorScheme::Light);
        assert_eq!(colors.bg, 0x00F3_F1EF);
        assert_eq!(colors.ink, 0x0038_3332);
        assert_eq!(colors.accent, 0x00C4_9172);
        assert_eq!(colors.surface, 0x00FF_FFFF);
        assert_eq!(colors.on_ink, 0x00FF_FFFF);
        assert_eq!(colors.curve_hi, 0x0064_B4F0);
    }

    #[test]
    fn dark_text_sits_on_a_darker_page() {
        let colors = palette(ColorScheme::Dark);
        assert!(luma(colors.ink) > luma(colors.bg));
        assert!(luma(colors.accent_text) > luma(colors.bg));
        assert!(luma(colors.status) > luma(colors.bg));
        assert_ne!(colors.surface, colors.bg);
        assert_ne!(colors.tile, colors.bg);
        assert!(luma(colors.tile) > luma(colors.tile_on));
        assert_ne!(colors.track, colors.bg);
    }
}
