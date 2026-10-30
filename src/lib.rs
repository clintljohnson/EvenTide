//! Layout and settings behavior for the EvenTide window.
//!
//! Dim alpha comes from `dimmer::layered_alpha`. Color ramps come from
//! `color::kelvin_to_rgb`, `color::gamma_channel`, and `color::schedule`.
//! This module does not open a window or call Win32.

mod color;
mod dimmer;

pub use color::{
    format_clock, gamma_channel, kelvin_from_fraction, kelvin_to_rgb, location_text, schedule,
    sky_label, slider_fraction, solar_position, status_line, Schedule, Settings as ColorSettings,
    SunStatus, UtcTime, DEFAULT_WAKE_MINUTE, SLIDER_MAX_KELVIN,
};
pub use dimmer::{
    layered_alpha, level_from_saved_index, overlay_should_exist, BRIGHTNESS_LABELS, LENS_FILL,
};

pub const CLIENT_WIDTH: i32 = 691;
pub const CLIENT_HEIGHT: i32 = 672;
pub const MAX_TILES: usize = 100;

/// `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`.
pub const LENS_EXTENDED_STYLE: u32 = 0x800A8;
/// `SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE`.
pub const LENS_RAISE_FLAGS: u32 = 0x13;

pub const APP_NAME: &str = "EvenTide";
pub const REG_KEY: &str = "Software\\EvenTide";
/// Read when `REG_KEY` has no `BrightnessIndex` or `City`. EvenTide never writes this key.
pub const PRIOR_REG_KEY: &str = "Software\\EvenTide\\Lens";
/// Read when the keys above have no settings values. EvenTide never writes this key.
pub const EARLIER_REG_KEY: &str = "Software\\Eventide\\Lens";
/// Read when the keys above have no settings values. EvenTide never writes this key.
pub const LEGACY_REG_KEY: &str = "Software\\Pangobright\\Hybrid";
/// Clock shown when **Transition at sunset** is unchecked. 8:00 PM.
pub const DEFAULT_TRANSITION_MINUTE: i32 = 20 * 60;
/// Clock shown when **Revert at sunrise** is unchecked. 7:00 AM.
pub const DEFAULT_REVERT_MINUTE: i32 = 7 * 60;

/// `%LOCALAPPDATA%\EvenTide`. Per-user JSON settings live here so Program Files installs stay writable.
/// Local AppData (not Roaming) fits machine-specific monitor layouts.
pub fn config_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|root| std::path::PathBuf::from(root).join(APP_NAME))
}

pub fn preset_path() -> Option<std::path::PathBuf> {
    config_dir().map(|dir| dir.join("presets.json"))
}

pub fn monitor_path() -> Option<std::path::PathBuf> {
    config_dir().map(|dir| dir.join("monitors.json"))
}

/// Create `%LOCALAPPDATA%\EvenTide` and seed missing `presets.json` / `monitors.json` with defaults.
pub fn ensure_config_files() -> std::io::Result<()> {
    let dir = config_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "LOCALAPPDATA is not set")
    })?;
    std::fs::create_dir_all(&dir)?;
    let presets = dir.join("presets.json");
    if !presets.exists() {
        std::fs::write(&presets, presets_to_json(&[]))?;
    }
    let monitors = dir.join("monitors.json");
    if !monitors.exists() {
        std::fs::write(
            &monitors,
            monitors_to_json(&MonitorSet::All, &MonitorSet::All),
        )?;
    }
    Ok(())
}

pub const SEED_LATITUDE_HUNDREDTHS: i32 = 3240;
pub const SEED_LONGITUDE_HUNDREDTHS: i32 = -9000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinPreset {
    pub name: &'static str,
    pub day: i32,
    pub night: i32,
    pub late: i32,
}

pub const BUILTIN_PRESETS: &[BuiltinPreset] = &[
    BuiltinPreset {
        name: "Day and Candle",
        day: 6500,
        night: 3400,
        late: 1900,
    },
    BuiltinPreset {
        name: "Easy on the Eyes",
        day: 5900,
        night: 3600,
        late: 2500,
    },
    BuiltinPreset {
        name: "Warm All Night",
        day: 6500,
        night: 3400,
        late: 3400,
    },
    BuiltinPreset {
        name: "Desk After Dark",
        day: 6500,
        night: 6500,
        late: 2300,
    },
    BuiltinPreset {
        name: "Light Touch",
        day: 6500,
        night: 5500,
        late: 1900,
    },
    BuiltinPreset {
        name: "Hearth",
        day: 2700,
        night: 2300,
        late: 1500,
    },
    BuiltinPreset {
        name: "True Color",
        day: 4950,
        night: 4450,
        late: 3400,
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonitorSet {
    All,
    Only(Vec<u32>),
}

impl MonitorSet {
    pub fn contains(&self, number: u32) -> bool {
        match self {
            MonitorSet::All => true,
            MonitorSet::Only(list) => list.contains(&number),
        }
    }

    /// Turn one attached display on or off. `"all"` means every display attached now.
    pub fn toggle(&self, number: u32, attached: &[u32]) -> Self {
        let mut list: Vec<u32> = attached
            .iter()
            .copied()
            .filter(|n| self.contains(*n))
            .collect();
        if let Some(index) = list.iter().position(|n| *n == number) {
            list.remove(index);
        } else {
            list.push(number);
        }
        list.sort_unstable();
        list.dedup();
        if !attached.is_empty() && attached.iter().all(|n| list.contains(n)) {
            MonitorSet::All
        } else {
            MonitorSet::Only(list)
        }
    }
}

/// Global dimming window. It is not stored on a profile.
/// A checked sunset or sunrise edge follows the sun. An unchecked edge uses its clock minute.
/// **Dim with schedule** lets that window run. Unchecked, the window stays closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimSchedule {
    pub transition_at_sunset: bool,
    pub transition_minute: i32,
    pub revert_at_sunrise: bool,
    pub revert_minute: i32,
    pub dim_with_schedule: bool,
}

impl DimSchedule {
    /// Both edges follow the sun. The revealed clocks stay at 8:00 PM and 7:00 AM.
    pub fn solar() -> Self {
        Self {
            transition_at_sunset: true,
            transition_minute: DEFAULT_TRANSITION_MINUTE,
            revert_at_sunrise: true,
            revert_minute: DEFAULT_REVERT_MINUTE,
            dim_with_schedule: true,
        }
    }
}

/// Where the sun and the clock are for one dimming decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimMoment {
    pub minute_of_day: i32,
    pub altitude_deg: f64,
    /// Before solar noon. Dawn is the morning side of civil twilight.
    pub morning: bool,
    pub sunset_minute: i32,
    pub sunrise_minute: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomPreset {
    pub name: String,
    pub day_kelvin: i32,
    pub night_kelvin: i32,
    pub late_kelvin: i32,
    pub brightness_index: u32,
    pub dim_monitors: MonitorSet,
    pub color_monitors: MonitorSet,
}

/// Manual dimming gate. `Schedule` lets `DimSchedule` decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimOverride {
    Schedule,
    On,
    Off,
}

/// `true` means the dim membership is applied. `false` means no lens, on every monitor.
pub fn dimming_enabled(schedule: DimSchedule, moment: DimMoment, mode: DimOverride) -> bool {
    match mode {
        DimOverride::On => true,
        DimOverride::Off => false,
        DimOverride::Schedule => scheduled_dim_fraction(schedule, moment) > 0.0,
    }
}

/// How far **Dim with sunset** has faded in.
/// `0` while the sun is up, `1` once it is 6° or more below the horizon.
/// Dawn runs the same span backward. That 6° is civil twilight, the dusk and dawn label.
pub fn sunset_dim_fraction(altitude_deg: f64) -> f64 {
    if !altitude_deg.is_finite() || altitude_deg >= 0.0 {
        0.0
    } else if altitude_deg <= -6.0 {
        1.0
    } else {
        -altitude_deg / 6.0
    }
}

/// Opacity byte for the black lens. Sunset and sunrise edges scale the chosen
/// brightness across civil twilight. A clock edge snaps. A forced state uses the full level or none.
pub fn lens_alpha(schedule: DimSchedule, moment: DimMoment, mode: DimOverride, level: u32) -> u8 {
    let full = layered_alpha(level);
    match mode {
        DimOverride::Off => 0,
        DimOverride::On => full,
        DimOverride::Schedule => {
            (f64::from(full) * scheduled_dim_fraction(schedule, moment)).round() as u8
        }
    }
}

/// How far the global dimming window has faded in. `0` is off and `1` is the full brightness.
/// Sunset plus sunrise follows solar altitude. A clock edge is a hard start or stop.
pub fn scheduled_dim_fraction(schedule: DimSchedule, moment: DimMoment) -> f64 {
    if !schedule.dim_with_schedule {
        return 0.0;
    }
    if schedule.transition_at_sunset && schedule.revert_at_sunrise {
        return sunset_dim_fraction(moment.altitude_deg);
    }
    if schedule.revert_at_sunrise && moment.morning && moment.altitude_deg >= 0.0 {
        return 0.0;
    }
    if schedule.transition_at_sunset
        && !moment.morning
        && moment.altitude_deg >= 0.0
        && before_sunset_start(schedule, moment)
    {
        return 0.0;
    }
    let start = if schedule.transition_at_sunset {
        moment.sunset_minute
    } else {
        schedule.transition_minute
    };
    let end = if schedule.revert_at_sunrise {
        moment.sunrise_minute
    } else {
        schedule.revert_minute
    };
    if !minute_in_window(moment.minute_of_day, start, end) {
        return 0.0;
    }
    if schedule.transition_at_sunset
        && !moment.morning
        && moment.altitude_deg < 0.0
        && moment.altitude_deg > -6.0
    {
        return sunset_dim_fraction(moment.altitude_deg);
    }
    if schedule.revert_at_sunrise && moment.morning && moment.altitude_deg < 0.0 {
        return sunset_dim_fraction(moment.altitude_deg);
    }
    1.0
}

/// Daylight before today's sunset is outside a sunset start, except the part of a
/// wrapping clock end that is still before that end.
fn before_sunset_start(schedule: DimSchedule, moment: DimMoment) -> bool {
    let now = moment.minute_of_day.rem_euclid(24 * 60);
    let start = moment.sunset_minute.rem_euclid(24 * 60);
    if now >= start {
        return true;
    }
    let end = if schedule.revert_at_sunrise {
        moment.sunrise_minute
    } else {
        schedule.revert_minute
    }
    .rem_euclid(24 * 60);
    !(start > end && now < end)
}

/// `true` from `start` up to but not including `end`. The window wraps midnight.
/// Equal endpoints are an empty window.
pub fn minute_in_window(now: i32, start: i32, end: i32) -> bool {
    let now = now.rem_euclid(24 * 60);
    let start = start.rem_euclid(24 * 60);
    let end = end.rem_euclid(24 * 60);
    if start == end {
        false
    } else if start < end {
        now >= start && now < end
    } else {
        now >= start || now < end
    }
}

/// Older registry values. A sunset schedule and a missing clock both become the solar defaults.
/// A saved clock minute becomes the transition time and still reverts at sunrise.
pub fn dim_schedule_from_legacy(follows: bool, minute: Option<u32>) -> DimSchedule {
    let mut schedule = DimSchedule::solar();
    if follows {
        return schedule;
    }
    if let Some(minute) = minute {
        if minute < 24 * 60 {
            schedule.transition_at_sunset = false;
            schedule.transition_minute = minute as i32;
        }
    }
    schedule
}

/// The Dimming On / Dimming Off button. The result replaces the schedule until it is cleared.
pub fn toggled_dim_override(enabled: bool) -> DimOverride {
    if enabled {
        DimOverride::Off
    } else {
        DimOverride::On
    }
}

pub fn dim_override_from_saved(value: u32) -> DimOverride {
    match value {
        1 => DimOverride::On,
        2 => DimOverride::Off,
        _ => DimOverride::Schedule,
    }
}

pub fn dim_override_to_saved(mode: DimOverride) -> u32 {
    match mode {
        DimOverride::Schedule => 0,
        DimOverride::On => 1,
        DimOverride::Off => 2,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LensSettings {
    pub brightness_index: u32,
    pub day_kelvin: i32,
    pub night_kelvin: i32,
    pub late_kelvin: i32,
    pub wake_minute: i32,
    pub latitude_hundredths: i32,
    pub longitude_hundredths: i32,
    pub city: String,
}

impl LensSettings {
    pub fn seed() -> Self {
        let city = nearest_city(SEED_LATITUDE_HUNDREDTHS, SEED_LONGITUDE_HUNDREDTHS);
        Self {
            brightness_index: 3,
            day_kelvin: 6500,
            night_kelvin: 3400,
            late_kelvin: 1900,
            wake_minute: DEFAULT_WAKE_MINUTE,
            latitude_hundredths: city.latitude_hundredths,
            longitude_hundredths: city.longitude_hundredths,
            city: city.label(),
        }
    }

    pub fn color_settings(&self) -> ColorSettings {
        ColorSettings {
            latitude_hundredths: self.latitude_hundredths,
            longitude_hundredths: self.longitude_hundredths,
            zip_code: 0,
            day_kelvin: self.day_kelvin,
            night_kelvin: self.night_kelvin,
            late_kelvin: self.late_kelvin,
            wake_minute: self.wake_minute,
            basic: 0,
        }
    }
}

pub fn set_segment_kelvin(settings: &mut LensSettings, sun_up: bool, kelvin: i32) {
    let kelvin = kelvin_from_fraction(slider_fraction(kelvin));
    if sun_up {
        settings.day_kelvin = kelvin;
    } else {
        settings.night_kelvin = kelvin;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderThumb {
    Day,
    Night,
}

pub fn slider_track_edges(
    outer_left: i32,
    outer_right: i32,
    left_label: i32,
    right_label: i32,
    gap: i32,
) -> (i32, i32) {
    (
        outer_left + left_label + gap,
        outer_right - right_label - gap,
    )
}

pub fn thumb_x(track_left: i32, track_right: i32, kelvin: i32) -> i32 {
    let span = (track_right - track_left).max(1);
    track_left + (slider_fraction(kelvin) * f64::from(span)).round() as i32
}

pub fn kelvin_from_track_x(x: i32, track_left: i32, track_right: i32) -> i32 {
    let span = (track_right - track_left).max(1);
    let fraction = f64::from(x - track_left) / f64::from(span);
    kelvin_from_fraction(fraction.clamp(0.0, 1.0))
}

pub fn day_kelvin_from_drag(x: i32, track_left: i32, track_right: i32, night_kelvin: i32) -> i32 {
    kelvin_from_track_x(x, track_left, track_right).max(night_kelvin)
}

pub fn night_kelvin_from_drag(x: i32, track_left: i32, track_right: i32, day_kelvin: i32) -> i32 {
    kelvin_from_track_x(x, track_left, track_right).min(day_kelvin)
}

pub fn nearest_slider_thumb(x: i32, day_x: i32, night_x: i32) -> SliderThumb {
    let day_dist = (x - day_x).abs();
    let night_dist = (x - night_x).abs();
    if night_dist < day_dist || (night_dist == day_dist && x <= night_x) {
        SliderThumb::Night
    } else {
        SliderThumb::Day
    }
}

/// Temperature printed on a monitor tile.
/// Color-on shows the temperature in effect. Color-off shows the other half of the day:
/// the day setpoint at night, and the night setpoint while the sun is up.
pub fn tile_kelvin(color_on: bool, sun_up: bool, active: i32, day: i32, night: i32) -> i32 {
    if color_on {
        active
    } else if sun_up {
        night
    } else {
        day
    }
}

/// Kelvin to show while the other half of the day is being dragged.
/// `None` keeps the scheduled temperature.
pub fn slider_preview_kelvin(
    sun_up: bool,
    thumb: SliderThumb,
    day: i32,
    night: i32,
) -> Option<i32> {
    match thumb {
        SliderThumb::Night if sun_up => Some(night),
        SliderThumb::Day if !sun_up => Some(day),
        _ => None,
    }
}

/// Kelvin for one monitor.
/// A held slider is previewed on every monitor, so the color toggle is ignored until release.
/// With no hold, color-on follows the schedule and color-off uses the other half of the day.
pub fn monitor_kelvin(
    color_on: bool,
    sun_up: bool,
    day: i32,
    night: i32,
    held: Option<SliderThumb>,
) -> i32 {
    if let Some(thumb) = held {
        return match thumb {
            SliderThumb::Day => day,
            SliderThumb::Night => night,
        };
    }
    tile_kelvin(
        color_on,
        sun_up,
        if sun_up { day } else { night },
        day,
        night,
    )
}

pub fn step_wake_minute(minute: i32, steps: i32) -> i32 {
    let base = minute.rem_euclid(24 * 60);
    (base + steps * 15).rem_euclid(24 * 60)
}

/// The preset the user last chose from a menu. Kelvin matching does not replace it
/// while those three temperatures still belong to that preset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresetChoice {
    Builtin(&'static str),
    Custom(String),
}

pub fn preset_label(
    day: i32,
    night: i32,
    late: i32,
    customs: &[CustomPreset],
    choice: Option<&PresetChoice>,
) -> String {
    if let Some(choice) = choice {
        if preset_choice_matches(choice, day, night, late, customs) {
            return match choice {
                PresetChoice::Builtin(name) => (*name).to_string(),
                PresetChoice::Custom(name) => name.clone(),
            };
        }
    }
    if let Some(preset) = BUILTIN_PRESETS
        .iter()
        .find(|preset| preset.day == day && preset.night == night && preset.late == late)
    {
        return preset.name.to_string();
    }
    if let Some(preset) = customs.iter().find(|preset| {
        preset.day_kelvin == day && preset.night_kelvin == night && preset.late_kelvin == late
    }) {
        return preset.name.clone();
    }
    "Custom".to_string()
}

fn preset_choice_matches(
    choice: &PresetChoice,
    day: i32,
    night: i32,
    late: i32,
    customs: &[CustomPreset],
) -> bool {
    match choice {
        PresetChoice::Builtin(name) => BUILTIN_PRESETS.iter().any(|preset| {
            preset.name == *name
                && preset.day == day
                && preset.night == night
                && preset.late == late
        }),
        PresetChoice::Custom(name) => customs.iter().any(|preset| {
            preset.name == *name
                && preset.day_kelvin == day
                && preset.night_kelvin == night
                && preset.late_kelvin == late
        }),
    }
}

/// Copies the profile's temperatures, brightness, and per-monitor dim and color switches.
/// Wake, location, and the dimming clock stay as they are.
pub fn apply_custom(
    settings: &mut LensSettings,
    dim: &mut MonitorSet,
    color: &mut MonitorSet,
    preset: &CustomPreset,
) {
    settings.day_kelvin = preset.day_kelvin;
    settings.night_kelvin = preset.night_kelvin;
    settings.late_kelvin = preset.late_kelvin;
    settings.brightness_index = preset.brightness_index.min(8);
    *dim = preset.dim_monitors.clone();
    *color = preset.color_monitors.clone();
}

/// Dropdown order: saved profiles, then the built-in names.
pub fn preset_menu_names(customs: &[CustomPreset]) -> Vec<String> {
    let mut names = customs
        .iter()
        .map(|preset| preset.name.clone())
        .collect::<Vec<_>>();
    names.extend(BUILTIN_PRESETS.iter().map(|preset| preset.name.to_string()));
    names
}

pub fn scheduled_ramp(kelvin: i32) -> [[u16; 256]; 3] {
    let rgb = kelvin_to_rgb(kelvin);
    [
        gamma_channel(rgb[0]),
        gamma_channel(rgb[1]),
        gamma_channel(rgb[2]),
    ]
}

pub fn identity_ramp() -> [[u16; 256]; 3] {
    let channel = gamma_channel(1.0);
    [channel, channel, channel]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedRamp {
    pub device_key: String,
    pub display_number: u32,
    pub ramp: [[u16; 256]; 3],
}

pub fn plan_ramps(
    devices: &[(String, u32)],
    color: &MonitorSet,
    sun_up: bool,
    day: i32,
    night: i32,
    held: Option<SliderThumb>,
) -> Vec<PlannedRamp> {
    devices
        .iter()
        .map(|(key, number)| PlannedRamp {
            device_key: key.clone(),
            display_number: *number,
            ramp: scheduled_ramp(monitor_kelvin(
                color.contains(*number),
                sun_up,
                day,
                night,
                held,
            )),
        })
        .collect()
}

pub fn lens_enabled(
    number: Option<u32>,
    dim: &MonitorSet,
    level: u32,
    attached_count: usize,
) -> bool {
    let enabled = match number {
        Some(n) => dim.contains(n),
        None => matches!(dim, MonitorSet::All),
    };
    overlay_should_exist(level, enabled, attached_count)
}

pub fn plan_dim_lenses(attached: &[u32], dim: &MonitorSet, level: u32) -> Vec<u32> {
    attached
        .iter()
        .copied()
        .filter(|number| lens_enabled(Some(*number), dim, level, attached.len()))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct City {
    pub name: &'static str,
    pub state: &'static str,
    pub latitude_hundredths: i32,
    pub longitude_hundredths: i32,
}

impl City {
    pub fn label(self) -> String {
        format!("{}, {}", self.state, self.name)
    }
}

/// `ST, City`, including a saved `City, ST` name from an older build.
pub fn canonical_city(stored: &str) -> String {
    let trimmed = stored.trim();
    for city in CITIES {
        let legacy = format!("{}, {}", city.name, city.state);
        if trimmed == city.label() || trimmed == legacy {
            return city.label();
        }
    }
    trimmed.to_string()
}

/// `7:00 AM`, `7:00 PM`, `19:00`, or `7 AM`. Minutes are 0 through 59.
pub fn parse_clock(text: &str) -> Option<i32> {
    let mut text = text.trim().to_ascii_lowercase();
    text.retain(|c| c != '.');
    if text.is_empty() {
        return None;
    }
    let mut words = text.split_whitespace();
    let clock = words.next()?;
    let mut suffix = words.next();
    if words.next().is_some() {
        return None;
    }
    let clock = if suffix.is_none() {
        if let Some(rest) = clock.strip_suffix("am") {
            suffix = Some("am");
            rest
        } else if let Some(rest) = clock.strip_suffix("pm") {
            suffix = Some("pm");
            rest
        } else {
            clock
        }
    } else {
        clock
    };
    let suffix = match suffix {
        Some("am" | "a") => Some(false),
        Some("pm" | "p") => Some(true),
        Some(_) => return None,
        None => None,
    };
    let mut parts = clock.split(':');
    let hour: i32 = parts.next()?.parse().ok()?;
    let minute: i32 = match parts.next() {
        Some(minute) => minute.parse().ok()?,
        None => 0,
    };
    if let Some(second) = parts.next() {
        let second: i32 = second.parse().ok()?;
        if !(0..60).contains(&second) || parts.next().is_some() {
            return None;
        }
    }
    if !(0..60).contains(&minute) {
        return None;
    }
    let hour24 = match suffix {
        Some(false) => {
            if !(1..=12).contains(&hour) {
                return None;
            }
            if hour == 12 {
                0
            } else {
                hour
            }
        }
        Some(true) => {
            if !(1..=12).contains(&hour) {
                return None;
            }
            if hour == 12 {
                12
            } else {
                hour + 12
            }
        }
        None => {
            if !(0..=23).contains(&hour) {
                return None;
            }
            hour
        }
    };
    Some(hour24 * 60 + minute)
}

/// One city in each state, plus the District of Columbia. Coordinates are hundredths of a degree.
pub const CITIES: &[City] = &[
    City {
        name: "Anchorage",
        state: "AK",
        latitude_hundredths: 6118,
        longitude_hundredths: -14990,
    },
    City {
        name: "Birmingham",
        state: "AL",
        latitude_hundredths: 3352,
        longitude_hundredths: -8680,
    },
    City {
        name: "Little Rock",
        state: "AR",
        latitude_hundredths: 3474,
        longitude_hundredths: -9229,
    },
    City {
        name: "Phoenix",
        state: "AZ",
        latitude_hundredths: 3345,
        longitude_hundredths: -11207,
    },
    City {
        name: "Los Angeles",
        state: "CA",
        latitude_hundredths: 3405,
        longitude_hundredths: -11824,
    },
    City {
        name: "Denver",
        state: "CO",
        latitude_hundredths: 3974,
        longitude_hundredths: -10499,
    },
    City {
        name: "Hartford",
        state: "CT",
        latitude_hundredths: 4176,
        longitude_hundredths: -7267,
    },
    City {
        name: "Washington",
        state: "DC",
        latitude_hundredths: 3891,
        longitude_hundredths: -7704,
    },
    City {
        name: "Wilmington",
        state: "DE",
        latitude_hundredths: 3974,
        longitude_hundredths: -7555,
    },
    City {
        name: "Miami",
        state: "FL",
        latitude_hundredths: 2576,
        longitude_hundredths: -8019,
    },
    City {
        name: "Atlanta",
        state: "GA",
        latitude_hundredths: 3375,
        longitude_hundredths: -8439,
    },
    City {
        name: "Honolulu",
        state: "HI",
        latitude_hundredths: 2131,
        longitude_hundredths: -15786,
    },
    City {
        name: "Des Moines",
        state: "IA",
        latitude_hundredths: 4159,
        longitude_hundredths: -9362,
    },
    City {
        name: "Boise",
        state: "ID",
        latitude_hundredths: 4362,
        longitude_hundredths: -11620,
    },
    City {
        name: "Chicago",
        state: "IL",
        latitude_hundredths: 4188,
        longitude_hundredths: -8763,
    },
    City {
        name: "Indianapolis",
        state: "IN",
        latitude_hundredths: 3977,
        longitude_hundredths: -8616,
    },
    City {
        name: "Wichita",
        state: "KS",
        latitude_hundredths: 3769,
        longitude_hundredths: -9734,
    },
    City {
        name: "Louisville",
        state: "KY",
        latitude_hundredths: 3825,
        longitude_hundredths: -8576,
    },
    City {
        name: "Baton Rouge",
        state: "LA",
        latitude_hundredths: 3045,
        longitude_hundredths: -9119,
    },
    City {
        name: "Boston",
        state: "MA",
        latitude_hundredths: 4236,
        longitude_hundredths: -7106,
    },
    City {
        name: "Baltimore",
        state: "MD",
        latitude_hundredths: 3929,
        longitude_hundredths: -7661,
    },
    City {
        name: "Portland",
        state: "ME",
        latitude_hundredths: 4366,
        longitude_hundredths: -7026,
    },
    City {
        name: "Detroit",
        state: "MI",
        latitude_hundredths: 4233,
        longitude_hundredths: -8305,
    },
    City {
        name: "Minneapolis",
        state: "MN",
        latitude_hundredths: 4498,
        longitude_hundredths: -9327,
    },
    City {
        name: "St. Louis",
        state: "MO",
        latitude_hundredths: 3863,
        longitude_hundredths: -9020,
    },
    City {
        name: "Jackson",
        state: "MS",
        latitude_hundredths: 3230,
        longitude_hundredths: -9018,
    },
    City {
        name: "Billings",
        state: "MT",
        latitude_hundredths: 4578,
        longitude_hundredths: -10850,
    },
    City {
        name: "Charlotte",
        state: "NC",
        latitude_hundredths: 3523,
        longitude_hundredths: -8084,
    },
    City {
        name: "Fargo",
        state: "ND",
        latitude_hundredths: 4688,
        longitude_hundredths: -9679,
    },
    City {
        name: "Omaha",
        state: "NE",
        latitude_hundredths: 4126,
        longitude_hundredths: -9594,
    },
    City {
        name: "Manchester",
        state: "NH",
        latitude_hundredths: 4300,
        longitude_hundredths: -7146,
    },
    City {
        name: "Newark",
        state: "NJ",
        latitude_hundredths: 4074,
        longitude_hundredths: -7417,
    },
    City {
        name: "Albuquerque",
        state: "NM",
        latitude_hundredths: 3511,
        longitude_hundredths: -10661,
    },
    City {
        name: "Las Vegas",
        state: "NV",
        latitude_hundredths: 3617,
        longitude_hundredths: -11514,
    },
    City {
        name: "New York",
        state: "NY",
        latitude_hundredths: 4071,
        longitude_hundredths: -7401,
    },
    City {
        name: "Columbus",
        state: "OH",
        latitude_hundredths: 3996,
        longitude_hundredths: -8300,
    },
    City {
        name: "Oklahoma City",
        state: "OK",
        latitude_hundredths: 3547,
        longitude_hundredths: -9752,
    },
    City {
        name: "Portland",
        state: "OR",
        latitude_hundredths: 4552,
        longitude_hundredths: -12268,
    },
    City {
        name: "Philadelphia",
        state: "PA",
        latitude_hundredths: 3995,
        longitude_hundredths: -7517,
    },
    City {
        name: "Providence",
        state: "RI",
        latitude_hundredths: 4182,
        longitude_hundredths: -7141,
    },
    City {
        name: "Charleston",
        state: "SC",
        latitude_hundredths: 3278,
        longitude_hundredths: -7993,
    },
    City {
        name: "Sioux Falls",
        state: "SD",
        latitude_hundredths: 4355,
        longitude_hundredths: -9673,
    },
    City {
        name: "Memphis",
        state: "TN",
        latitude_hundredths: 3515,
        longitude_hundredths: -9005,
    },
    City {
        name: "Houston",
        state: "TX",
        latitude_hundredths: 2976,
        longitude_hundredths: -9537,
    },
    City {
        name: "Salt Lake City",
        state: "UT",
        latitude_hundredths: 4076,
        longitude_hundredths: -11189,
    },
    City {
        name: "Richmond",
        state: "VA",
        latitude_hundredths: 3754,
        longitude_hundredths: -7746,
    },
    City {
        name: "Burlington",
        state: "VT",
        latitude_hundredths: 4448,
        longitude_hundredths: -7321,
    },
    City {
        name: "Seattle",
        state: "WA",
        latitude_hundredths: 4761,
        longitude_hundredths: -12233,
    },
    City {
        name: "Milwaukee",
        state: "WI",
        latitude_hundredths: 4304,
        longitude_hundredths: -8791,
    },
    City {
        name: "Charleston",
        state: "WV",
        latitude_hundredths: 3835,
        longitude_hundredths: -8163,
    },
    City {
        name: "Cheyenne",
        state: "WY",
        latitude_hundredths: 4114,
        longitude_hundredths: -10482,
    },
];

pub fn nearest_city(latitude_hundredths: i32, longitude_hundredths: i32) -> &'static City {
    CITIES
        .iter()
        .min_by_key(|city| {
            let dlat = i64::from(city.latitude_hundredths - latitude_hundredths);
            let dlon = i64::from(city.longitude_hundredths - longitude_hundredths);
            dlat * dlat + dlon * dlon
        })
        .expect("city table is not empty")
}

pub fn select_city(settings: &mut LensSettings, city: &City) {
    settings.latitude_hundredths = city.latitude_hundredths;
    settings.longitude_hundredths = city.longitude_hundredths;
    settings.city = city.label();
}

/// First-run color, brightness, wake, and monitor membership. The chosen city stays.
pub fn restore_first_run(
    settings: &mut LensSettings,
    dim: &mut MonitorSet,
    color: &mut MonitorSet,
) {
    let city = settings.city.clone();
    let latitude_hundredths = settings.latitude_hundredths;
    let longitude_hundredths = settings.longitude_hundredths;
    *settings = LensSettings::seed();
    settings.city = city;
    settings.latitude_hundredths = latitude_hundredths;
    settings.longitude_hundredths = longitude_hundredths;
    *dim = MonitorSet::All;
    *color = MonitorSet::All;
}

/// Settings shows the primary monitor as 1. A `\\.\DISPLAYn` source keeps `n` when `n` is
/// above the primary's GDI index, and becomes `n + 1` when `n` is below it.
pub fn settings_number(gdi_index: Option<u32>, primary_gdi_index: Option<u32>) -> Option<u32> {
    let gdi = gdi_index?;
    let Some(primary) = primary_gdi_index else {
        return Some(gdi);
    };
    Some(if gdi == primary {
        1
    } else if gdi < primary {
        gdi + 1
    } else {
        gdi
    })
}

pub fn gdi_index_from_device_name(name: &str) -> Option<u32> {
    let base = name.rsplit(['\\', '/']).next().unwrap_or(name);
    let upper = base.to_ascii_uppercase();
    let rest = upper.strip_prefix("DISPLAY")?;
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u32>().ok().filter(|n| *n > 0)
}

/// Shown when a monitor rectangle has no Settings number.
pub fn tile_label(number: Option<u32>) -> String {
    match number {
        Some(n) => n.to_string(),
        None => "?".to_string(),
    }
}

/// Text drawn in a tile. One monitor reads `Display 1`. Several keep the number.
pub fn tile_caption(number: Option<u32>, count: usize) -> String {
    match number {
        Some(n) if count == 1 => format!("Display {n}"),
        _ => tile_label(number),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    pub fn width(self) -> i32 {
        self.right - self.left
    }

    pub fn height(self) -> i32 {
        self.bottom - self.top
    }

    pub fn area(self) -> i64 {
        i64::from(self.width()) * i64::from(self.height())
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && y >= self.top && x < self.right && y < self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorInput {
    pub number: Option<u32>,
    pub bounds: ScreenRect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub number: Option<u32>,
    pub bounds: ScreenRect,
}

pub fn monitor_band() -> ScreenRect {
    ScreenRect {
        left: 12,
        top: 364,
        right: CLIENT_WIDTH - 12,
        bottom: CLIENT_HEIGHT - 12,
    }
}

/// Centered above the tiles. `content_top` is the top of the monitor array.
/// The button sits halfway between its resting place (8px above the band) and that top.
pub fn dim_toggle_rect(content_top: i32) -> ScreenRect {
    let band = monitor_band();
    let width = 168;
    let height = 32;
    let resting_bottom = band.top - 8;
    let gap = (content_top - resting_bottom).max(0);
    let bottom = resting_bottom + gap / 2;
    let center = (band.left + band.right) / 2;
    ScreenRect {
        left: center - width / 2,
        top: bottom - height,
        right: center + width / 2,
        bottom,
    }
}

/// Brightness, same height as the dimming toggle, 8px to its left.
pub fn brightness_rect_beside(dim: ScreenRect) -> ScreenRect {
    let width = 116;
    let gap = 8;
    ScreenRect {
        left: dim.left - gap - width,
        top: dim.top,
        right: dim.left - gap,
        bottom: dim.bottom,
    }
}

/// Brightness and the dimming toggle, centered together above the tiles.
pub fn dim_control_pair(content_top: i32) -> (ScreenRect, ScreenRect) {
    let anchor = dim_toggle_rect(content_top);
    let band = monitor_band();
    let brightness_width = 116;
    let gap = 8;
    let dim_width = anchor.right - anchor.left;
    let group_width = brightness_width + gap + dim_width;
    let mut group_left = (band.left + band.right) / 2 - group_width / 2;
    if group_left < band.left {
        group_left = band.left;
    }
    if group_left + group_width > band.right {
        group_left = (band.right - group_width).max(band.left);
    }
    let brightness = ScreenRect {
        left: group_left,
        top: anchor.top,
        right: group_left + brightness_width,
        bottom: anchor.bottom,
    };
    let dim = ScreenRect {
        left: brightness.right + gap,
        top: anchor.top,
        right: brightness.right + gap + dim_width,
        bottom: anchor.bottom,
    };
    (brightness, dim)
}

/// City list, in screen coordinates. It sits to the right of the window and of the
/// location hit, and its vertical center matches the window when it fits.
/// A tight virtual screen pulls it back, which may overlap the window.
pub fn city_menu_origin(
    window: ScreenRect,
    hit_right: i32,
    menu_width: i32,
    menu_height: i32,
    virtual_screen: ScreenRect,
) -> (i32, i32) {
    let menu_width = menu_width.max(1);
    let menu_height = menu_height.max(1);
    let mut left = hit_right.max(window.right);
    let mut top = window.top + (window.bottom - window.top - menu_height) / 2;
    if left + menu_width > virtual_screen.right {
        left = virtual_screen.right - menu_width;
    }
    if left < virtual_screen.left {
        left = virtual_screen.left;
    }
    if top + menu_height > virtual_screen.bottom {
        top = virtual_screen.bottom - menu_height;
    }
    if top < virtual_screen.top {
        top = virtual_screen.top;
    }
    (left, top)
}

/// Top-left of a dialog centered on `parent`. A tight virtual screen pulls it back
/// onto the desktop, which can leave it slightly off the parent's center.
pub fn dialog_origin(
    parent: ScreenRect,
    dialog_width: i32,
    dialog_height: i32,
    virtual_screen: ScreenRect,
) -> (i32, i32) {
    let dialog_width = dialog_width.max(1);
    let dialog_height = dialog_height.max(1);
    let mut left = parent.left + (parent.width() - dialog_width) / 2;
    let mut top = parent.top + (parent.height() - dialog_height) / 2;
    if left + dialog_width > virtual_screen.right {
        left = virtual_screen.right - dialog_width;
    }
    if left < virtual_screen.left {
        left = virtual_screen.left;
    }
    if top + dialog_height > virtual_screen.bottom {
        top = virtual_screen.bottom - dialog_height;
    }
    if top < virtual_screen.top {
        top = virtual_screen.top;
    }
    (left, top)
}

pub fn tile_controls_are_labeled(count: usize) -> bool {
    (1..=6).contains(&count)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilePart {
    Dim,
    Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileZones {
    pub dim: ScreenRect,
    pub color: ScreenRect,
    pub labeled: bool,
    /// Top of the monitor numeral.
    pub number_y: i32,
}

/// Horizontal padding inside the temperature button, around the kelvin text.
pub const COLOR_BUTTON_PAD_X: i32 = 10;
/// Vertical padding inside the temperature button.
pub const COLOR_BUTTON_PAD_Y: i32 = 6;
/// Space between the monitor number and the temperature button.
pub const COLOR_BUTTON_NUMBER_GAP: i32 = 8;

/// Measured pieces of the temperature button. `number_height` is the monitor numeral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorControlSize {
    pub text_width: i32,
    pub text_height: i32,
    pub icon_width: i32,
    pub icon_height: i32,
    pub number_height: i32,
}

pub fn tile_zones(bounds: ScreenRect, labeled: bool, control: ColorControlSize) -> TileZones {
    if labeled {
        let button = color_button_size(control);
        let margin = 8;
        let mut top = bounds.bottom - button.height() - margin;
        if top < bounds.top + 2 {
            top = bounds.top + 2;
        }
        let left = bounds.left + (bounds.width() - button.width()).max(0) / 2;
        let color = ScreenRect {
            left,
            top,
            right: (left + button.width()).min(bounds.right - 2),
            bottom: (top + button.height()).min(bounds.bottom - 2),
        };
        let number_h = control.number_height.max(1);
        let space_top = bounds.top + 4;
        let space_bottom = (color.top - COLOR_BUTTON_NUMBER_GAP).max(space_top + number_h);
        let number_y = space_top + (space_bottom - space_top - number_h).max(0) / 2;
        TileZones {
            dim: bounds,
            color,
            labeled: true,
            number_y,
        }
    } else {
        let mark = (bounds.width().min(bounds.height()) / 3).clamp(8, 22);
        TileZones {
            dim: bounds,
            color: ScreenRect {
                left: bounds.right - mark - 3,
                top: bounds.top + 3,
                right: bounds.right - 3,
                bottom: bounds.top + 3 + mark,
            },
            labeled: false,
            number_y: bounds.top + 4,
        }
    }
}

fn color_button_size(control: ColorControlSize) -> ScreenRect {
    let inner_w = control.text_width.max(1);
    let inner_h = control.text_height.max(1);
    ScreenRect {
        left: 0,
        top: 0,
        right: inner_w + COLOR_BUTTON_PAD_X * 2,
        bottom: inner_h + COLOR_BUTTON_PAD_Y * 2,
    }
}

pub fn zone_at(zones: &TileZones, x: i32, y: i32) -> Option<TilePart> {
    if zones.color.contains(x, y) {
        Some(TilePart::Color)
    } else if zones.dim.contains(x, y) {
        Some(TilePart::Dim)
    } else {
        None
    }
}

/// Extra margin around the only tile. Multi-monitor tiles do not use this.
pub const SOLO_TILE_INSET: i32 = 20;

/// Place up to 100 monitors into the bottom band, preserving virtual-screen arrangement.
/// Placement order is top to bottom, then left to right. Monitors past 100 are omitted.
pub fn place_tiles(monitors: &[MonitorInput], band: ScreenRect) -> Vec<Tile> {
    let mut ordered: Vec<MonitorInput> = monitors.to_vec();
    ordered.sort_by(|a, b| {
        a.bounds
            .top
            .cmp(&b.bounds.top)
            .then(a.bounds.left.cmp(&b.bounds.left))
            .then(a.number.cmp(&b.number))
    });
    ordered.truncate(MAX_TILES);
    if ordered.is_empty() {
        return Vec::new();
    }
    let min_l = ordered.iter().map(|m| m.bounds.left).min().unwrap();
    let min_t = ordered.iter().map(|m| m.bounds.top).min().unwrap();
    let max_r = ordered.iter().map(|m| m.bounds.right).max().unwrap();
    let max_b = ordered.iter().map(|m| m.bounds.bottom).max().unwrap();
    let span_w = (max_r - min_l).max(1);
    let span_h = (max_b - min_t).max(1);
    let band_w = (band.right - band.left).max(1);
    let band_h = (band.bottom - band.top).max(1);
    let scale = (f64::from(band_w) / f64::from(span_w)).min(f64::from(band_h) / f64::from(span_h));
    let used_w = (f64::from(span_w) * scale).floor() as i32;
    let used_h = (f64::from(span_h) * scale).floor() as i32;
    let origin_x = band.left + (band_w - used_w) / 2;
    let origin_y = band.top + (band_h - used_h) / 2;
    // One tile is pulled in by `SOLO_TILE_INSET`. Several tiles are inset 3px, a 6px gap.
    let inset = if ordered.len() == 1 {
        SOLO_TILE_INSET
    } else {
        3
    };
    ordered
        .into_iter()
        .map(|monitor| {
            let left = origin_x + ((f64::from(monitor.bounds.left - min_l) * scale).round() as i32);
            let top = origin_y + ((f64::from(monitor.bounds.top - min_t) * scale).round() as i32);
            let right =
                origin_x + ((f64::from(monitor.bounds.right - min_l) * scale).round() as i32);
            let bottom =
                origin_y + ((f64::from(monitor.bounds.bottom - min_t) * scale).round() as i32);
            let fitted = clamp_rect(
                ScreenRect {
                    left,
                    top,
                    right,
                    bottom,
                },
                band,
            );
            Tile {
                number: monitor.number,
                bounds: shrink_rect(fitted, inset, band),
            }
        })
        .collect()
}

fn clamp_rect(rect: ScreenRect, band: ScreenRect) -> ScreenRect {
    let left = rect.left.clamp(band.left, band.right - 1);
    let top = rect.top.clamp(band.top, band.bottom - 1);
    let right = rect.right.clamp(left + 1, band.right);
    let bottom = rect.bottom.clamp(top + 1, band.bottom);
    ScreenRect {
        left,
        top,
        right,
        bottom,
    }
}

fn shrink_rect(rect: ScreenRect, inset: i32, band: ScreenRect) -> ScreenRect {
    if inset <= 0 || rect.width() <= inset * 2 + 4 || rect.height() <= inset * 2 + 4 {
        return clamp_rect(rect, band);
    }
    clamp_rect(
        ScreenRect {
            left: rect.left + inset,
            top: rect.top + inset,
            right: rect.right - inset,
            bottom: rect.bottom - inset,
        },
        band,
    )
}

pub fn tile_rows(tiles: &[Tile]) -> Vec<Vec<Tile>> {
    let mut items = tiles.to_vec();
    items.sort_by_key(|tile| (tile.bounds.top, tile.bounds.left, tile.number));
    let mut rows: Vec<Vec<Tile>> = Vec::new();
    for tile in items {
        let cy = (tile.bounds.top + tile.bounds.bottom) / 2;
        let same_row = rows.last().is_some_and(|row| {
            let sample = &row[0];
            let sy = (sample.bounds.top + sample.bounds.bottom) / 2;
            (cy - sy).abs() <= sample.bounds.height().max(1) / 3
        });
        if same_row {
            rows.last_mut().unwrap().push(tile);
        } else {
            rows.push(vec![tile]);
        }
    }
    for row in &mut rows {
        row.sort_by_key(|tile| tile.bounds.left);
    }
    rows
}

#[derive(Debug)]
pub struct FileError;

pub fn parse_preset_file(text: &str) -> Result<Vec<CustomPreset>, FileError> {
    let root = parse_json(text).map_err(|_| FileError)?;
    let presets = object_get(root.as_object().ok_or(FileError)?, "presets")
        .and_then(Json::as_array)
        .ok_or(FileError)?;
    presets.iter().map(parse_preset).collect()
}

pub fn presets_to_json(presets: &[CustomPreset]) -> String {
    let mut out = String::from("{\n  \"presets\": [");
    if presets.is_empty() {
        out.push_str("]\n}\n");
        return out;
    }
    out.push('\n');
    for (index, preset) in presets.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": {},\n", json_string(&preset.name)));
        out.push_str(&format!("      \"day_kelvin\": {},\n", preset.day_kelvin));
        out.push_str(&format!(
            "      \"night_kelvin\": {},\n",
            preset.night_kelvin
        ));
        out.push_str(&format!("      \"late_kelvin\": {},\n", preset.late_kelvin));
        out.push_str(&format!(
            "      \"brightness_index\": {},\n",
            preset.brightness_index
        ));
        out.push_str(&format!(
            "      \"dim_monitors\": {},\n",
            json_monitors(&preset.dim_monitors)
        ));
        out.push_str(&format!(
            "      \"color_monitors\": {}\n",
            json_monitors(&preset.color_monitors)
        ));
        out.push_str("    }");
        if index + 1 != presets.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

pub fn parse_monitor_file(text: &str) -> Result<(MonitorSet, MonitorSet), FileError> {
    let root = parse_json(text).map_err(|_| FileError)?;
    let object = root.as_object().ok_or(FileError)?;
    let dim = parse_monitor_set(object_get(object, "dim_monitors").ok_or(FileError)?)?;
    let color = parse_monitor_set(object_get(object, "color_monitors").ok_or(FileError)?)?;
    Ok((dim, color))
}

pub fn monitors_to_json(dim: &MonitorSet, color: &MonitorSet) -> String {
    format!(
        "{{\n  \"dim_monitors\": {},\n  \"color_monitors\": {}\n}}\n",
        json_monitors(dim),
        json_monitors(color)
    )
}

fn parse_preset(value: &Json) -> Result<CustomPreset, FileError> {
    let object = value.as_object().ok_or(FileError)?;
    let name = object_get(object, "name")
        .and_then(Json::as_str)
        .ok_or(FileError)?;
    if name.is_empty() {
        return Err(FileError);
    }
    Ok(CustomPreset {
        name: name.to_string(),
        day_kelvin: json_i32(object_get(object, "day_kelvin").ok_or(FileError)?)?,
        night_kelvin: json_i32(object_get(object, "night_kelvin").ok_or(FileError)?)?,
        late_kelvin: json_i32(object_get(object, "late_kelvin").ok_or(FileError)?)?,
        brightness_index: {
            let index = json_i32(object_get(object, "brightness_index").ok_or(FileError)?)?;
            if (0..=8).contains(&index) {
                index as u32
            } else {
                return Err(FileError);
            }
        },
        dim_monitors: parse_monitor_set(object_get(object, "dim_monitors").ok_or(FileError)?)?,
        color_monitors: parse_monitor_set(object_get(object, "color_monitors").ok_or(FileError)?)?,
    })
}

fn parse_monitor_set(value: &Json) -> Result<MonitorSet, FileError> {
    if let Some(text) = value.as_str() {
        if text == "all" {
            return Ok(MonitorSet::All);
        }
        return Err(FileError);
    }
    let array = value.as_array().ok_or(FileError)?;
    let mut numbers = Vec::new();
    for item in array {
        let number = json_i32(item)?;
        if number < 0 {
            return Err(FileError);
        }
        numbers.push(number as u32);
    }
    numbers.sort_unstable();
    numbers.dedup();
    Ok(MonitorSet::Only(numbers))
}

fn json_monitors(set: &MonitorSet) -> String {
    match set {
        MonitorSet::All => "\"all\"".to_string(),
        MonitorSet::Only(list) => {
            let body = list
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{body}]")
        }
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_i32(value: &Json) -> Result<i32, FileError> {
    let number = value.as_i64().ok_or(FileError)?;
    i32::try_from(number).map_err(|_| FileError)
}

#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Number(i64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Object(fields) => Some(fields),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Number(number) => Some(*number),
            _ => None,
        }
    }
}

fn object_get<'a>(object: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    object
        .iter()
        .rev()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
}

fn parse_json(text: &str) -> Result<Json, ()> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        index: 0,
    };
    let value = parser.parse_value()?;
    parser.skip();
    if parser.index != parser.bytes.len() {
        return Err(());
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Parser<'a> {
    fn skip(&mut self) {
        while self.index < self.bytes.len() && self.bytes[self.index].is_ascii_whitespace() {
            self.index += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip();
        self.bytes.get(self.index).copied()
    }

    fn eat(&mut self, expected: u8) -> Result<(), ()> {
        if self.peek() == Some(expected) {
            self.index += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    fn parse_value(&mut self) -> Result<Json, ()> {
        match self.peek().ok_or(())? {
            b'n' => self.parse_literal(b"null", Json::Null),
            b't' => self.parse_literal(b"true", Json::Number(1)),
            b'f' => self.parse_literal(b"false", Json::Number(0)),
            b'"' => Ok(Json::String(self.parse_string()?)),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(()),
        }
    }

    fn parse_literal(&mut self, literal: &[u8], value: Json) -> Result<Json, ()> {
        if self.bytes[self.index..].starts_with(literal) {
            self.index += literal.len();
            Ok(value)
        } else {
            Err(())
        }
    }

    fn parse_number(&mut self) -> Result<Json, ()> {
        let start = self.index;
        if self.bytes[self.index] == b'-' {
            self.index += 1;
        }
        if self.index >= self.bytes.len() || !self.bytes[self.index].is_ascii_digit() {
            return Err(());
        }
        while self.index < self.bytes.len() && self.bytes[self.index].is_ascii_digit() {
            self.index += 1;
        }
        if self.index < self.bytes.len() && matches!(self.bytes[self.index], b'.' | b'e' | b'E') {
            return Err(());
        }
        let text = std::str::from_utf8(&self.bytes[start..self.index]).map_err(|_| ())?;
        let number = text.parse::<i64>().map_err(|_| ())?;
        Ok(Json::Number(number))
    }

    fn parse_string(&mut self) -> Result<String, ()> {
        self.eat(b'"')?;
        let mut out = String::new();
        while self.index < self.bytes.len() {
            let byte = self.bytes[self.index];
            self.index += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let escape = self.bytes.get(self.index).copied().ok_or(())?;
                    self.index += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            if self.index + 4 > self.bytes.len() {
                                return Err(());
                            }
                            let hex = std::str::from_utf8(&self.bytes[self.index..self.index + 4])
                                .map_err(|_| ())?;
                            let code = u32::from_str_radix(hex, 16).map_err(|_| ())?;
                            out.push(char::from_u32(code).ok_or(())?);
                            self.index += 4;
                        }
                        _ => return Err(()),
                    }
                }
                b if b < 0x20 => return Err(()),
                _ => {
                    let width = utf8_width(byte).ok_or(())?;
                    if self.index - 1 + width > self.bytes.len() {
                        return Err(());
                    }
                    let text =
                        std::str::from_utf8(&self.bytes[self.index - 1..self.index - 1 + width])
                            .map_err(|_| ())?;
                    out.push_str(text);
                    self.index = self.index - 1 + width;
                }
            }
        }
        Err(())
    }

    fn parse_array(&mut self) -> Result<Json, ()> {
        self.eat(b'[')?;
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.index += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            match self.peek().ok_or(())? {
                b']' => {
                    self.index += 1;
                    return Ok(Json::Array(items));
                }
                b',' => self.index += 1,
                _ => return Err(()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, ()> {
        self.eat(b'{')?;
        let mut fields = Vec::new();
        if self.peek() == Some(b'}') {
            self.index += 1;
            return Ok(Json::Object(fields));
        }
        loop {
            if self.peek() != Some(b'"') {
                return Err(());
            }
            let key = self.parse_string()?;
            self.eat(b':')?;
            let value = self.parse_value()?;
            fields.push((key, value));
            match self.peek().ok_or(())? {
                b'}' => {
                    self.index += 1;
                    return Ok(Json::Object(fields));
                }
                b',' => self.index += 1,
                _ => return Err(()),
            }
        }
    }
}

fn utf8_width(first: u8) -> Option<usize> {
    if first < 0x80 {
        Some(1)
    } else if first & 0xE0 == 0xC0 {
        Some(2)
    } else if first & 0xF0 == 0xE0 {
        Some(3)
    } else if first & 0xF8 == 0xF0 {
        Some(4)
    } else {
        None
    }
}

/// Rows shown under the "United States" heading.
pub const CITY_MENU_ROWS: i32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CityEffect {
    Stay,
    Close,
    Choose(i32),
}

/// The button release that finishes the click which opened the list.
/// `row_index` is ignored: that release must not choose a city or close the list.
pub fn city_opening_release(row_index: i32) -> CityEffect {
    let _ = row_index;
    CityEffect::Stay
}

/// A later press. `over_list` is the window rectangle, including its border.
pub fn city_press(over_list: bool) -> CityEffect {
    if over_list {
        CityEffect::Stay
    } else {
        CityEffect::Close
    }
}

/// A later release. The opening release is handled by `city_opening_release`,
/// so `armed` is false only when that release was delivered straight to the list.
pub fn city_release(armed: bool, row_index: i32) -> CityEffect {
    if armed && row_index >= 0 {
        CityEffect::Choose(row_index)
    } else {
        CityEffect::Stay
    }
}

/// Wheel movement never closes the list. The caller scrolls only while the
/// pointer is over the list.
pub fn city_wheel(over_list: bool) -> CityEffect {
    let _ = over_list;
    CityEffect::Stay
}

pub fn city_escape() -> CityEffect {
    CityEffect::Close
}

/// City index under a client Y, or -1 on the heading, above the list, or past the last city.
pub fn city_row_index(y: i32, row_px: i32, scroll: i32, city_count: i32) -> i32 {
    if row_px <= 0 || y < row_px {
        return -1;
    }
    let index = scroll + (y - row_px) / row_px;
    if (0..city_count).contains(&index) {
        index
    } else {
        -1
    }
}

/// `raw_delta` is the Windows wheel delta. Positive (rolled away from the user)
/// moves toward the first city. `pending` keeps a partial notch.
pub fn scroll_city_wheel(
    scroll: i32,
    pending: &mut i32,
    raw_delta: i32,
    lines_per_notch: i32,
    city_count: i32,
    visible_rows: i32,
) -> i32 {
    if raw_delta == 0 {
        return scroll;
    }
    *pending += -raw_delta;
    let notches = *pending / 120;
    *pending %= 120;
    let steps = notches * lines_per_notch.max(0);
    if steps == 0 {
        return scroll;
    }
    let max_scroll = (city_count - visible_rows).max(0);
    (scroll + steps).clamp(0, max_scroll)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> ScreenRect {
        ScreenRect {
            left,
            top,
            right,
            bottom,
        }
    }

    fn moment(minute: i32, altitude: f64, morning: bool, sunset: i32, sunrise: i32) -> DimMoment {
        DimMoment {
            minute_of_day: minute,
            altitude_deg: altitude,
            morning,
            sunset_minute: sunset,
            sunrise_minute: sunrise,
        }
    }

    fn this_pc() -> Vec<MonitorInput> {
        vec![
            MonitorInput {
                number: Some(6),
                bounds: rect(0, -1082, 1920, -2),
            },
            MonitorInput {
                number: Some(5),
                bounds: rect(1920, -1073, 3840, 7),
            },
            MonitorInput {
                number: Some(4),
                bounds: rect(3840, -1076, 5760, 4),
            },
            MonitorInput {
                number: Some(1),
                bounds: rect(0, 0, 1920, 1080),
            },
            MonitorInput {
                number: Some(2),
                bounds: rect(1925, 7, 3845, 1087),
            },
            MonitorInput {
                number: Some(3),
                bounds: rect(3845, 4, 5765, 1084),
            },
        ]
    }

    #[test]
    fn six_monitor_tiles_match_the_settings_arrangement() {
        let primary = gdi_index_from_device_name(r"\\.\DISPLAY3");
        let numbered = [
            (r"\\.\DISPLAY6", rect(0, -1082, 1920, -2)),
            (r"\\.\DISPLAY5", rect(1920, -1073, 3840, 7)),
            (r"\\.\DISPLAY4", rect(3840, -1076, 5760, 4)),
            (r"\\.\DISPLAY3", rect(0, 0, 1920, 1080)),
            (r"\\.\DISPLAY1", rect(1925, 7, 3845, 1087)),
            (r"\\.\DISPLAY2", rect(3845, 4, 5765, 1084)),
        ];
        let inputs: Vec<MonitorInput> = numbered
            .into_iter()
            .map(|(name, bounds)| MonitorInput {
                number: settings_number(gdi_index_from_device_name(name), primary),
                bounds,
            })
            .collect();
        let tiles = place_tiles(&inputs, monitor_band());
        let rows = tile_rows(&tiles);
        assert_eq!(rows.len(), 2);
        let numbers = |row: &Vec<Tile>| row.iter().map(|tile| tile.number).collect::<Vec<_>>();
        assert_eq!(numbers(&rows[0]), vec![Some(6), Some(5), Some(4)]);
        assert_eq!(numbers(&rows[1]), vec![Some(1), Some(2), Some(3)]);
        let bottom_left = rows[1][0];
        assert_eq!(bottom_left.number, Some(1));
        assert!(bottom_left.bounds.top > rows[0][0].bounds.top);
        assert!(rows[0][0].bounds.top >= monitor_band().top);
        assert!(tile_label(None) == "?");
        assert_eq!(settings_number(None, primary), None);
    }

    #[test]
    fn hundred_tiles_fit_and_one_monitor_is_larger() {
        let mut grid = Vec::new();
        let mut number = 1u32;
        for row in 0..10 {
            for col in 0..10 {
                grid.push(MonitorInput {
                    number: Some(number),
                    bounds: rect(col * 100, row * 80, col * 100 + 100, row * 80 + 80),
                });
                number += 1;
            }
        }
        let band = monitor_band();
        let tiles = place_tiles(&grid, band);
        assert_eq!(tiles.len(), 100);
        for tile in &tiles {
            assert!(tile.bounds.left >= band.left);
            assert!(tile.bounds.top >= band.top);
            assert!(tile.bounds.right <= band.right);
            assert!(tile.bounds.bottom <= band.bottom);
            assert!(tile.bounds.width() > 0);
            assert!(tile.bounds.height() > 0);
        }
        let one = place_tiles(
            &[MonitorInput {
                number: Some(1),
                bounds: rect(0, 0, 1920, 1080),
            }],
            band,
        );
        assert!(one[0].bounds.area() > tiles[0].bounds.area());
        assert_eq!(one[0].bounds.top, band.top + SOLO_TILE_INSET);
        assert_eq!(one[0].bounds.bottom, band.bottom - SOLO_TILE_INSET);
        assert!(one[0].bounds.left >= band.left + SOLO_TILE_INSET);
        assert!(one[0].bounds.right <= band.right - SOLO_TILE_INSET);
        assert_eq!(tile_caption(Some(1), 1), "Display 1");
        assert_eq!(tile_caption(Some(4), 6), "4");
        assert_eq!(tile_caption(None, 1), "?");
        let mut pair = place_tiles(
            &[
                MonitorInput {
                    number: Some(1),
                    bounds: rect(0, 0, 1920, 1080),
                },
                MonitorInput {
                    number: Some(2),
                    bounds: rect(1920, 0, 3840, 1080),
                },
            ],
            band,
        );
        pair.sort_by_key(|tile| tile.bounds.left);
        assert_eq!(pair[1].bounds.left - pair[0].bounds.right, 6);
        assert_eq!(tile_caption(pair[0].number, pair.len()), "1");
        assert!(tile_controls_are_labeled(1));
        assert!(tile_controls_are_labeled(6));
        assert!(!tile_controls_are_labeled(7));
        assert!(!tile_controls_are_labeled(100));
    }

    #[test]
    fn default_config_json_matches_the_schema() {
        assert_eq!(
            parse_preset_file(&presets_to_json(&[])).unwrap(),
            Vec::<CustomPreset>::new()
        );
        let (dim, color) =
            parse_monitor_file(&monitors_to_json(&MonitorSet::All, &MonitorSet::All)).unwrap();
        assert_eq!(dim, MonitorSet::All);
        assert_eq!(color, MonitorSet::All);
        let expected = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
            .join("EvenTide");
        assert_eq!(config_dir().unwrap(), expected);
        assert!(expected
            .components()
            .any(|c| c.as_os_str() == std::ffi::OsStr::new("Local")));
        assert_eq!(preset_path().unwrap(), expected.join("presets.json"));
        assert_eq!(monitor_path().unwrap(), expected.join("monitors.json"));
    }

    #[test]
    fn preset_file_round_trips_and_a_broken_file_keeps_builtins() {
        let text = r#"{
  "presets": [
    {
      "name": "Evening desks",
      "day_kelvin": 6500,
      "night_kelvin": 5000,
      "late_kelvin": 3400,
      "brightness_index": 3,
      "tint": 0,
      "dim_monitors": [1, 2, 3],
      "color_monitors": "all",
      "trigger": { "kind": "clock", "minute": 1080 }
    }
  ]
}"#;
        let loaded = parse_preset_file(text).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Evening desks");
        assert_eq!(loaded[0].dim_monitors, MonitorSet::Only(vec![1, 2, 3]));
        assert_eq!(loaded[0].color_monitors, MonitorSet::All);
        assert_eq!(loaded[0].day_kelvin, 6500);
        assert_eq!(loaded[0].night_kelvin, 5000);
        let again = parse_preset_file(&presets_to_json(&loaded)).unwrap();
        assert_eq!(again, loaded);
        let saved = presets_to_json(&loaded);
        assert!(!saved.contains("\"wake_minute\""));
        assert!(!saved.contains("\"dim_mode\""));
        assert!(!saved.contains("\"city\""));
        assert!(!saved.contains("\"tint\""));

        let mut both = loaded.clone();
        both.push(CustomPreset {
            name: "Later".to_string(),
            day_kelvin: 5000,
            night_kelvin: 3400,
            late_kelvin: 1900,
            brightness_index: 4,
            dim_monitors: MonitorSet::All,
            color_monitors: MonitorSet::Only(vec![4]),
        });
        let written = parse_preset_file(&presets_to_json(&both)).unwrap();
        assert_eq!(written[1].brightness_index, 4);
        assert_eq!(written[1].color_monitors, MonitorSet::Only(vec![4]));
        assert!(!presets_to_json(&written).contains("\"tint\""));
        assert_eq!(
            preset_menu_names(&written),
            vec![
                "Evening desks".to_string(),
                "Later".to_string(),
                "Day and Candle".to_string(),
                "Easy on the Eyes".to_string(),
                "Warm All Night".to_string(),
                "Desk After Dark".to_string(),
                "Light Touch".to_string(),
                "Hearth".to_string(),
                "True Color".to_string(),
            ]
        );
        let without_tint = r#"{
  "presets": [
    {
      "name": "No tint",
      "day_kelvin": 6500,
      "night_kelvin": 3400,
      "late_kelvin": 1900,
      "brightness_index": 3,
      "dim_monitors": "all",
      "color_monitors": "all",
      "dim_mode": "manual"
    }
  ]
}"#;
        assert_eq!(parse_preset_file(without_tint).unwrap()[0].name, "No tint");
        let mut settings = LensSettings::seed();
        settings.wake_minute = 600;
        settings.city = "OR, Portland".to_string();
        let mut dim = MonitorSet::All;
        let mut color = MonitorSet::All;
        apply_custom(&mut settings, &mut dim, &mut color, &written[1]);
        assert_eq!(settings.wake_minute, 600);
        assert_eq!(settings.city, "OR, Portland");
        assert_eq!(settings.brightness_index, 4);
        assert_eq!(settings.day_kelvin, 5000);
        assert_eq!(settings.night_kelvin, 3400);
        assert_eq!(color, MonitorSet::Only(vec![4]));
        apply_custom(&mut settings, &mut dim, &mut color, &written[0]);
        assert_eq!(settings.wake_minute, 600);
        assert_eq!(settings.city, "OR, Portland");
        assert_eq!(settings.day_kelvin, 6500);
        assert_eq!(settings.night_kelvin, 5000);

        let legacy_schedule = r#"{
  "presets": [
    {
      "name": "Kept",
      "day_kelvin": 6500,
      "night_kelvin": 3400,
      "late_kelvin": 1900,
      "brightness_index": 3,
      "tint": 0,
      "dim_monitors": "all",
      "color_monitors": "all",
      "dim_follows_schedule": true,
      "trigger": { "kind": "clock", "minute": 1260 }
    }
  ]
}"#;
        assert_eq!(parse_preset_file(legacy_schedule).unwrap()[0].name, "Kept");
        let legacy_solar = legacy_schedule
            .replace(
                "\"dim_follows_schedule\": true",
                "\"dim_follows_schedule\": false",
            )
            .replace(
                "\"kind\": \"clock\", \"minute\": 1260",
                "\"kind\": \"solar\", \"event\": \"sunset\"",
            );
        assert_eq!(parse_preset_file(&legacy_solar).unwrap()[0].name, "Kept");
        assert!(parse_preset_file("{").is_err());
        assert!(parse_preset_file("not json").is_err());
        assert_eq!(BUILTIN_PRESETS.len(), 7);
        assert_eq!(BUILTIN_PRESETS[6].name, "True Color");
        assert_eq!(
            (
                BUILTIN_PRESETS[6].day,
                BUILTIN_PRESETS[6].night,
                BUILTIN_PRESETS[6].late
            ),
            (4950, 4450, 3400)
        );
    }

    #[test]
    fn a_chosen_custom_keeps_its_name_when_its_kelvins_match_a_builtin() {
        let custom = CustomPreset {
            name: "My Custom Preset".to_string(),
            day_kelvin: 4950,
            night_kelvin: 4450,
            late_kelvin: 3400,
            brightness_index: 2,
            dim_monitors: MonitorSet::All,
            color_monitors: MonitorSet::Only(vec![1, 2, 3, 4, 5]),
        };
        let customs = [custom.clone()];
        assert_eq!(preset_label(4950, 4450, 3400, &customs, None), "True Color");
        assert_eq!(
            preset_label(
                4950,
                4450,
                3400,
                &customs,
                Some(&PresetChoice::Custom(custom.name.clone()))
            ),
            "My Custom Preset"
        );
        assert_eq!(
            preset_label(
                4950,
                4450,
                3400,
                &customs,
                Some(&PresetChoice::Builtin("True Color"))
            ),
            "True Color"
        );
        assert_eq!(
            preset_label(
                5000,
                4450,
                3400,
                &customs,
                Some(&PresetChoice::Custom(custom.name.clone()))
            ),
            "Custom"
        );

        let mut settings = LensSettings::seed();
        let mut dim = MonitorSet::Only(vec![6]);
        let mut color = MonitorSet::All;
        apply_custom(&mut settings, &mut dim, &mut color, &custom);
        assert_eq!(settings.day_kelvin, 4950);
        assert_eq!(settings.night_kelvin, 4450);
        assert_eq!(settings.late_kelvin, 3400);
        assert_eq!(settings.brightness_index, 2);
        assert_eq!(dim, MonitorSet::All);
        assert_eq!(color, MonitorSet::Only(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn nearest_city_to_the_seed_point_is_in_mississippi() {
        let city = nearest_city(SEED_LATITUDE_HUNDREDTHS, SEED_LONGITUDE_HUNDREDTHS);
        assert_eq!(city.state, "MS");
        assert_eq!(city.name, "Jackson");
        let mut settings = LensSettings::seed();
        select_city(&mut settings, city);
        assert_eq!(settings.latitude_hundredths, city.latitude_hundredths);
        assert_eq!(settings.longitude_hundredths, city.longitude_hundredths);
        assert_eq!(settings.city, "MS, Jackson");
        assert_eq!(canonical_city("Jackson, MS"), "MS, Jackson");
        let ordered: Vec<_> = CITIES.iter().map(|city| (city.state, city.name)).collect();
        let mut sorted = ordered.clone();
        sorted.sort();
        assert_eq!(ordered, sorted);
        assert_eq!(CITIES[0].label(), "AK, Anchorage");
        assert_eq!(parse_clock("7:00 AM"), Some(420));
        assert_eq!(parse_clock("7:00 PM"), Some(19 * 60));
        assert_eq!(parse_clock("12:00 AM"), Some(0));
        assert_eq!(parse_clock("12:00 PM"), Some(12 * 60));
        assert_eq!(parse_clock("19:05"), Some(19 * 60 + 5));
        assert_eq!(parse_clock("7:00:30 PM"), Some(19 * 60));
        assert!(parse_clock("7:60").is_none());
        assert!(parse_clock("13:00 PM").is_none());
        let tile = rect(10, 20, 210, 160);
        let control = ColorControlSize {
            text_width: 52,
            text_height: 16,
            icon_width: 12,
            icon_height: 20,
            number_height: 22,
        };
        let zones = tile_zones(tile, true, control);
        assert_eq!(zone_at(&zones, 30, 40), Some(TilePart::Dim));
        assert_eq!(
            zone_at(&zones, zones.color.left + 4, zones.color.top + 4),
            Some(TilePart::Color)
        );
        assert!(zones.number_y >= tile.top);
        assert!(
            zones.number_y + control.number_height + COLOR_BUTTON_NUMBER_GAP <= zones.color.top
        );
        assert!(zones.color.left > tile.left && zones.color.right < tile.right);
        assert!(zones.color.bottom <= tile.bottom);
        assert!(tile.bottom - zones.color.bottom <= 12);
        assert!(zones.color.top > tile.top + tile.height() / 2);
        let crowded = tile_zones(tile, false, control);
        assert_eq!(zone_at(&crowded, 30, 40), Some(TilePart::Dim));
        assert_eq!(
            zone_at(&crowded, crowded.color.left + 2, crowded.color.top + 2),
            Some(TilePart::Color)
        );
        let when = UtcTime {
            year: 2026,
            month: 10,
            day: 24,
            hour: 18,
            minute: 0,
            second: 0,
        };
        let position = solar_position(
            when,
            settings.color_settings().latitude_deg(),
            settings.color_settings().longitude_deg(),
        );
        assert!(position.altitude_deg.is_finite());
        assert!(position.solar_hours.is_finite());
        let states: std::collections::BTreeSet<_> = CITIES.iter().map(|city| city.state).collect();
        assert_eq!(states.len(), 51);
    }

    #[test]
    fn color_off_uses_the_other_temperature_and_dim_off_omits_that_lens() {
        let devices = vec![
            (r"\\.\DISPLAY1".to_string(), 1),
            (r"\\.\DISPLAY2".to_string(), 2),
            (r"\\.\DISPLAY3".to_string(), 3),
            (r"\\.\DISPLAY4".to_string(), 4),
            (r"\\.\DISPLAY5".to_string(), 5),
            (r"\\.\DISPLAY6".to_string(), 6),
        ];
        let color = MonitorSet::Only(vec![1, 2, 3, 5, 6]);
        let plans = plan_ramps(&devices, &color, false, 4750, 4100, None);
        let day = scheduled_ramp(4750);
        let night = scheduled_ramp(4100);
        let fourth = plans.iter().find(|plan| plan.display_number == 4).unwrap();
        assert_eq!(fourth.device_key, r"\\.\DISPLAY4");
        assert_eq!(fourth.ramp, day);
        for plan in plans.iter().filter(|plan| plan.display_number != 4) {
            assert_eq!(plan.ramp, night);
        }
        let hold_day = plan_ramps(&devices, &color, false, 4750, 4100, Some(SliderThumb::Day));
        assert!(hold_day.iter().all(|plan| plan.ramp == day));
        let hold_night = plan_ramps(
            &devices,
            &color,
            false,
            4750,
            4100,
            Some(SliderThumb::Night),
        );
        assert!(hold_night.iter().all(|plan| plan.ramp == night));
        assert_eq!(monitor_kelvin(false, true, 4750, 4100, None), 4100);
        assert_eq!(monitor_kelvin(true, true, 4750, 4100, None), 4750);
        assert_eq!(
            monitor_kelvin(false, true, 4750, 4100, Some(SliderThumb::Day)),
            4750
        );
        let dim = MonitorSet::Only(vec![2, 3, 4, 5, 6]);
        let lenses = plan_dim_lenses(&[1, 2, 3, 4, 5, 6], &dim, level_from_saved_index(3));
        assert!(!lenses.contains(&1));
        assert_eq!(lenses, vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn monitor_band_is_the_bottom_of_the_window() {
        let band = monitor_band();
        assert!(band.top > CLIENT_HEIGHT / 2);
        assert_eq!(band.left, 12);
        assert_eq!(band.right, CLIENT_WIDTH - 12);
        assert_eq!(band.bottom, CLIENT_HEIGHT - 12);
        let button = dim_toggle_rect(band.top);
        assert!(button.bottom <= band.top);
        assert!(button.bottom > CLIENT_HEIGHT / 2);
        assert!(button.left > band.left && button.right < band.right);
        let (brightness, dim) = dim_control_pair(band.top);
        assert_eq!(
            (brightness.left + dim.right) / 2,
            (band.left + band.right) / 2
        );
        assert_eq!(brightness.top, dim.top);
        assert_eq!(brightness.bottom, dim.bottom);
        assert_eq!(brightness.right + 8, dim.left);
        assert!(brightness.left >= band.left);
        assert!(dim.right <= band.right);
    }

    #[test]
    fn slider_puts_the_higher_kelvin_on_the_right() {
        let (track_left, track_right) = slider_track_edges(24, CLIENT_WIDTH - 24, 80, 100, 16);
        let night_label_right = 24 + 80;
        let day_label_left = CLIENT_WIDTH - 24 - 100;
        assert_eq!(track_left - night_label_right, 16);
        assert_eq!(day_label_left - track_right, 16);
        assert_eq!(
            (track_left + track_right) / 2,
            (night_label_right + day_label_left) / 2
        );
        assert!(thumb_x(track_left, track_right, 3400) < thumb_x(track_left, track_right, 6500));
        assert_eq!(
            day_kelvin_from_drag(track_left, track_left, track_right, 3400),
            3400
        );
        assert_eq!(
            day_kelvin_from_drag(track_right, track_left, track_right, 3400),
            6500
        );
        assert_eq!(
            night_kelvin_from_drag(track_right, track_left, track_right, 6500),
            6500
        );
        assert_eq!(
            night_kelvin_from_drag(track_left, track_left, track_right, 6500),
            1200
        );
        assert_eq!(
            nearest_slider_thumb(
                track_left,
                thumb_x(track_left, track_right, 6500),
                thumb_x(track_left, track_right, 3400)
            ),
            SliderThumb::Night
        );
        let shared = thumb_x(track_left, track_right, 5000);
        assert_eq!(
            nearest_slider_thumb(shared - 1, shared, shared),
            SliderThumb::Night
        );
        assert_eq!(
            nearest_slider_thumb(shared + 1, shared, shared),
            SliderThumb::Day
        );
        assert_eq!(
            slider_preview_kelvin(true, SliderThumb::Night, 6500, 3400),
            Some(3400)
        );
        assert_eq!(
            slider_preview_kelvin(true, SliderThumb::Day, 6500, 3400),
            None
        );
        assert_eq!(
            slider_preview_kelvin(false, SliderThumb::Day, 6500, 3400),
            Some(6500)
        );
        assert_eq!(
            slider_preview_kelvin(false, SliderThumb::Night, 6500, 3400),
            None
        );
        assert_eq!(tile_kelvin(true, false, 4350, 5200, 4350), 4350);
        assert_eq!(tile_kelvin(false, false, 4350, 5200, 4350), 5200);
        assert_eq!(tile_kelvin(true, true, 5200, 5200, 3400), 5200);
        assert_eq!(tile_kelvin(false, true, 5200, 5200, 3400), 3400);
    }

    #[test]
    fn dimming_and_brightness_split_the_gap_above_the_tiles() {
        let band = monitor_band();
        let tiles = place_tiles(&this_pc(), band);
        let content_top = tiles.iter().map(|tile| tile.bounds.top).min().unwrap();
        let resting_bottom = band.top - 8;
        let dim = dim_toggle_rect(content_top);
        assert!(content_top > resting_bottom);
        assert_eq!(
            dim.bottom,
            resting_bottom + (content_top - resting_bottom) / 2
        );
        assert!(dim.bottom <= content_top);
        let (brightness, dim) = dim_control_pair(content_top);
        assert_eq!(
            (brightness.left + dim.right) / 2,
            (band.left + band.right) / 2
        );
        assert_eq!(brightness.right + 8, dim.left);
        assert!(dim.bottom <= content_top);
        assert!(brightness.bottom < content_top);
    }

    #[test]
    fn city_menu_opens_beside_the_window() {
        let window = rect(100, 80, 860, 820);
        let screen = rect(0, 0, 1920, 1080);
        let (left, top) = city_menu_origin(window, 840, 220, 640, screen);
        assert_eq!(left, window.right);
        assert!(left >= 840);
        let window_mid = window.top + (window.bottom - window.top) / 2;
        let menu_mid = top + 640 / 2;
        assert_eq!(menu_mid, window_mid);
        assert!(top < 400);

        let cramped = rect(0, 0, 1000, 1080);
        let (left, top) = city_menu_origin(window, 840, 220, 640, cramped);
        assert_eq!(left, 780);
        assert!(left < window.right);
        assert_eq!(top, window.top + (window.bottom - window.top - 640) / 2);

        let low = rect(100, 900, 860, 1500);
        let (left, top) = city_menu_origin(low, 840, 220, 700, screen);
        assert_eq!(left, low.right);
        assert_eq!(top, 1080 - 700);
        assert!(top + 700 <= screen.bottom);
    }

    #[test]
    fn dialogs_open_over_the_window() {
        let window = rect(200, 100, 900, 800);
        let screen = rect(0, 0, 1920, 1080);
        let (left, top) = dialog_origin(window, 400, 300, screen);
        assert_eq!(left + 400 / 2, (window.left + window.right) / 2);
        assert_eq!(top + 300 / 2, (window.top + window.bottom) / 2);

        let edge = rect(1800, 900, 2500, 1600);
        let (left, top) = dialog_origin(edge, 400, 300, screen);
        assert_eq!(left, 1920 - 400);
        assert_eq!(top, 1080 - 300);
        assert!(left + 400 <= screen.right);
        assert!(top + 300 <= screen.bottom);
    }

    #[test]
    fn dimming_schedule_yields_to_the_override() {
        let solar = DimSchedule::solar();
        assert!(solar.transition_at_sunset && solar.revert_at_sunrise);
        assert_eq!(solar.transition_minute, 20 * 60);
        assert_eq!(solar.revert_minute, 7 * 60);
        let day = moment(15 * 60, 20.0, false, 18 * 60 + 10, 6 * 60 + 30);
        let dusk = moment(18 * 60 + 30, -3.0, false, 18 * 60 + 10, 6 * 60 + 30);
        let night = moment(23 * 60, -12.0, false, 18 * 60 + 10, 6 * 60 + 30);
        let dawn = moment(6 * 60, -3.0, true, 18 * 60 + 10, 6 * 60 + 30);
        assert!(!dimming_enabled(solar, day, DimOverride::Schedule));
        assert!(dimming_enabled(solar, night, DimOverride::Schedule));
        assert_eq!(toggled_dim_override(true), DimOverride::Off);
        assert_eq!(toggled_dim_override(false), DimOverride::On);
        assert!(dimming_enabled(solar, day, DimOverride::On));
        assert!(!dimming_enabled(solar, night, DimOverride::Off));
        assert_eq!(sunset_dim_fraction(1.0), 0.0);
        assert_eq!(sunset_dim_fraction(0.0), 0.0);
        assert!((sunset_dim_fraction(-3.0) - 0.5).abs() < 1e-9);
        assert_eq!(sunset_dim_fraction(-6.0), 1.0);
        assert_eq!(sunset_dim_fraction(-12.0), 1.0);
        assert_eq!(sunset_dim_fraction(f64::NAN), 0.0);
        let full = layered_alpha(0x4C);
        assert_eq!(lens_alpha(solar, day, DimOverride::Schedule, 0x4C), 0);
        assert_eq!(lens_alpha(solar, night, DimOverride::Schedule, 0x4C), full);
        assert_eq!(
            lens_alpha(solar, dusk, DimOverride::Schedule, 0x4C),
            (f64::from(full) * 0.5).round() as u8
        );
        assert_eq!(
            lens_alpha(solar, dawn, DimOverride::Schedule, 0x4C),
            (f64::from(full) * 0.5).round() as u8
        );
        assert_eq!(lens_alpha(solar, day, DimOverride::On, 0x4C), full);
        assert_eq!(lens_alpha(solar, night, DimOverride::Off, 0x4C), 0);
        let mut manual = solar;
        manual.dim_with_schedule = false;
        assert_eq!(scheduled_dim_fraction(manual, night), 0.0);
        assert_eq!(scheduled_dim_fraction(manual, dusk), 0.0);
        assert!(!dimming_enabled(manual, night, DimOverride::Schedule));
        assert!(dimming_enabled(manual, day, DimOverride::On));
        assert_eq!(lens_alpha(manual, night, DimOverride::Schedule, 0x4C), 0);
        assert_eq!(lens_alpha(manual, night, DimOverride::On, 0x4C), full);

        let mut clock_start = solar;
        clock_start.transition_at_sunset = false;
        clock_start.transition_minute = 20 * 60;
        let before_clock = moment(19 * 60, -8.0, false, 18 * 60 + 10, 6 * 60 + 30);
        let after_clock = moment(21 * 60, -10.0, false, 18 * 60 + 10, 6 * 60 + 30);
        assert_eq!(scheduled_dim_fraction(clock_start, before_clock), 0.0);
        assert_eq!(scheduled_dim_fraction(clock_start, after_clock), 1.0);
        assert!((scheduled_dim_fraction(clock_start, dawn) - 0.5).abs() < 1e-9);
        assert_eq!(scheduled_dim_fraction(clock_start, day), 0.0);

        let mut clock_end = solar;
        clock_end.revert_at_sunrise = false;
        clock_end.revert_minute = 23 * 60;
        assert!((scheduled_dim_fraction(clock_end, dusk) - 0.5).abs() < 1e-9);
        assert_eq!(scheduled_dim_fraction(clock_end, night), 0.0);
        let early_night = moment(22 * 60, -10.0, false, 18 * 60 + 10, 6 * 60 + 30);
        assert_eq!(scheduled_dim_fraction(clock_end, early_night), 1.0);
        clock_end.revert_minute = 9 * 60;
        let morning_day = moment(8 * 60, 5.0, true, 18 * 60 + 10, 6 * 60 + 30);
        assert_eq!(scheduled_dim_fraction(clock_end, morning_day), 1.0);
        assert_eq!(scheduled_dim_fraction(clock_end, dawn), 1.0);
        clock_end.revert_minute = 16 * 60;
        let still_on = moment(15 * 60, 30.0, false, 18 * 60 + 10, 6 * 60 + 30);
        let gap = moment(17 * 60, 10.0, false, 18 * 60 + 10, 6 * 60 + 30);
        assert_eq!(scheduled_dim_fraction(clock_end, still_on), 1.0);
        assert_eq!(scheduled_dim_fraction(clock_end, gap), 0.0);

        let mut both_clocks = DimSchedule::solar();
        both_clocks.transition_at_sunset = false;
        both_clocks.revert_at_sunrise = false;
        both_clocks.transition_minute = 20 * 60;
        both_clocks.revert_minute = 7 * 60;
        assert_eq!(scheduled_dim_fraction(both_clocks, after_clock), 1.0);
        assert_eq!(scheduled_dim_fraction(both_clocks, day), 0.0);
        assert_eq!(scheduled_dim_fraction(both_clocks, dawn), 1.0);
        let after_revert = moment(8 * 60, -2.0, true, 18 * 60 + 10, 6 * 60 + 30);
        assert_eq!(scheduled_dim_fraction(both_clocks, after_revert), 0.0);
        both_clocks.revert_minute = both_clocks.transition_minute;
        assert_eq!(scheduled_dim_fraction(both_clocks, after_clock), 0.0);

        let migrated = dim_schedule_from_legacy(false, Some(20 * 60));
        assert!(!migrated.transition_at_sunset);
        assert!(migrated.revert_at_sunrise);
        assert_eq!(migrated.transition_minute, 20 * 60);
        assert_eq!(
            dim_schedule_from_legacy(true, Some(20 * 60)),
            DimSchedule::solar()
        );
        assert_eq!(
            dim_schedule_from_legacy(false, Some(u32::MAX)),
            DimSchedule::solar()
        );
        assert!(minute_in_window(23 * 60, 20 * 60, 7 * 60));
        assert!(minute_in_window(6 * 60, 20 * 60, 7 * 60));
        assert!(!minute_in_window(12 * 60, 20 * 60, 7 * 60));
        assert!(!minute_in_window(8 * 60, 8 * 60, 8 * 60));

        assert_eq!(dim_override_from_saved(0), DimOverride::Schedule);
        assert_eq!(dim_override_from_saved(1), DimOverride::On);
        assert_eq!(dim_override_from_saved(2), DimOverride::Off);
        assert_eq!(dim_override_from_saved(9), DimOverride::Schedule);
        assert_eq!(dim_override_to_saved(DimOverride::Off), 2);
    }

    #[test]
    fn first_run_restore_keeps_the_chosen_city() {
        let portland = CITIES
            .iter()
            .find(|city| city.state == "OR")
            .expect("oregon");
        let mut settings = LensSettings::seed();
        select_city(&mut settings, portland);
        settings.brightness_index = 8;
        settings.day_kelvin = 2700;
        settings.night_kelvin = 2300;
        settings.late_kelvin = 1500;
        settings.wake_minute = 600;
        let city = settings.city.clone();
        let latitude = settings.latitude_hundredths;
        let longitude = settings.longitude_hundredths;
        let mut dim = MonitorSet::Only(vec![1]);
        let mut color = MonitorSet::Only(vec![2]);
        restore_first_run(&mut settings, &mut dim, &mut color);
        assert_eq!(settings.city, city);
        assert_eq!(settings.latitude_hundredths, latitude);
        assert_eq!(settings.longitude_hundredths, longitude);
        assert_eq!(settings.brightness_index, 3);
        assert_eq!(
            (
                settings.day_kelvin,
                settings.night_kelvin,
                settings.late_kelvin
            ),
            (6500, 3400, 1900)
        );
        assert_eq!(settings.wake_minute, 420);
        assert_eq!(dim, MonitorSet::All);
        assert_eq!(color, MonitorSet::All);
    }

    #[test]
    fn lens_style_matches_the_dim_overlay() {
        assert_eq!(LENS_EXTENDED_STYLE, 0x80000 | 0x20 | 0x80 | 0x8);
        assert_eq!(LENS_RAISE_FLAGS, 0x1 | 0x2 | 0x10);
        let _ = this_pc();
    }

    #[test]
    fn city_list_stays_open_on_the_opening_release_and_the_wheel_scrolls() {
        assert_eq!(city_opening_release(12), CityEffect::Stay);
        assert_eq!(city_opening_release(-1), CityEffect::Stay);
        assert_eq!(city_press(true), CityEffect::Stay);
        assert_eq!(city_press(false), CityEffect::Close);
        assert_eq!(city_release(false, 12), CityEffect::Stay);
        assert_eq!(city_release(true, -1), CityEffect::Stay);
        assert_eq!(city_release(true, 12), CityEffect::Choose(12));
        assert_eq!(city_wheel(true), CityEffect::Stay);
        assert_eq!(city_wheel(false), CityEffect::Stay);
        assert_eq!(city_escape(), CityEffect::Close);

        let count = CITIES.len() as i32;
        assert!(count > CITY_MENU_ROWS);
        assert_eq!(city_row_index(-8, 22, 0, count), -1);
        assert_eq!(city_row_index(10, 22, 0, count), -1);
        assert_eq!(city_row_index(22, 22, 0, count), 0);
        assert_eq!(city_row_index(44, 22, 2, count), 3);

        let mut pending = 0;
        assert_eq!(
            scroll_city_wheel(10, &mut pending, 120, 3, count, CITY_MENU_ROWS),
            7
        );
        assert_eq!(pending, 0);
        assert_eq!(
            scroll_city_wheel(7, &mut pending, -60, 3, count, CITY_MENU_ROWS),
            7
        );
        assert_eq!(pending, 60);
        assert_eq!(
            scroll_city_wheel(7, &mut pending, -60, 3, count, CITY_MENU_ROWS),
            10
        );
        assert_eq!(pending, 0);
        let mut pending = 0;
        let mut scroll = 0;
        for _ in 0..40 {
            scroll = scroll_city_wheel(scroll, &mut pending, -120, 3, count, CITY_MENU_ROWS);
        }
        assert_eq!(scroll, count - CITY_MENU_ROWS);
        assert_eq!(
            scroll_city_wheel(0, &mut pending, 0, 3, count, CITY_MENU_ROWS),
            0
        );
    }
}
