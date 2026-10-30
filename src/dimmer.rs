//! Dimmer alpha and the nine brightness steps.
//!
//! The lens is a layered window filled with black. `layered_alpha` is that
//! window's opacity, so the black fill darkens the desktop. A gamma ramp
//! leaves a black pixel black, which is why the scheduled color temperature
//! does not dim on its own. This module does not call Win32 and does not
//! read or write the registry.

/// Overlay darkness for the nine menu steps, from full brightness down to 20%.
pub const BRIGHTNESS_LEVELS: [u32; 9] = [0x00, 0x1A, 0x33, 0x4C, 0x66, 0x80, 0x9A, 0xB3, 0xCC];

pub const BRIGHTNESS_LABELS: [&str; 9] = [
    "100% (full brightness)",
    "90%",
    "80%",
    "70%",
    "60%",
    "50%",
    "40%",
    "30%",
    "20% (minimum brightness)",
];

/// `COLORREF` painted into every lens. Black darkens without a second hue.
pub const LENS_FILL: u32 = 0;

pub const DEFAULT_ALPHA_INDEX: i32 = 3;

/// Byte passed to `SetLayeredWindowAttributes`.
pub fn layered_alpha(level: u32) -> u8 {
    if level < 0x10 {
        0
    } else if level > 0xF0 {
        0xFF
    } else {
        (level - 0x0F) as u8
    }
}

/// Out-of-range indexes become index 3, then the level table is applied.
pub fn level_from_saved_index(index: i32) -> u32 {
    if (0..BRIGHTNESS_LEVELS.len() as i32).contains(&index) {
        BRIGHTNESS_LEVELS[index as usize]
    } else {
        BRIGHTNESS_LEVELS[DEFAULT_ALPHA_INDEX as usize]
    }
}

/// A lens exists when the level is non-zero and that monitor is enabled.
/// A single attached monitor is forced on.
pub fn overlay_should_exist(level: u32, enabled: bool, monitor_count: usize) -> bool {
    let enabled = if monitor_count == 1 { true } else { enabled };
    level != 0 && enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_levels_match_saved_indexes() {
        for (index, level) in BRIGHTNESS_LEVELS.iter().copied().enumerate() {
            assert_eq!(level_from_saved_index(index as i32), level);
        }
    }

    #[test]
    fn saved_index_outside_0_through_8_uses_the_default_level() {
        assert_eq!(level_from_saved_index(-1), 0x4C);
        assert_eq!(level_from_saved_index(9), 0x4C);
        assert_eq!(level_from_saved_index(3), 0x4C);
    }

    #[test]
    fn layered_alpha_matches_the_clamped_subtraction() {
        assert_eq!(layered_alpha(0), 0);
        assert_eq!(layered_alpha(0x0F), 0);
        assert_eq!(layered_alpha(0x1A), 0x0B);
        assert_eq!(layered_alpha(0x4C), 0x3D);
        assert_eq!(layered_alpha(0xCC), 0xBD);
        assert_eq!(layered_alpha(0xF1), 0xFF);
    }

    #[test]
    fn a_single_monitor_forces_a_lens_and_a_zero_level_does_not() {
        assert!(overlay_should_exist(0x4C, false, 1));
        assert!(!overlay_should_exist(0, true, 2));
        assert!(!overlay_should_exist(0x4C, false, 2));
    }
}
