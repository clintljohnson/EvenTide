//! Color temperature, gamma ramps, and the solar clock.
//!
//! The numeric tables and the solar formulas are carried over unchanged.
//! This module does not call Win32 and does not read or write the registry.
#![allow(dead_code)]

/// CIE 1931 2-degree color-matching samples used by `0x428DF0`.
/// 95 rows, 360 nm through 830 nm, step 5 nm, three little-endian `f32`s
/// (X, Y, Z). The loop's first sample is VA `0x50D280`. The 12 bytes at
/// `0x50D274` are a zero, a denormal, and the sRGB matrix pointer.
const CIE_CMF: &[u8] = include_bytes!("../assets/cie-cmf-5nm.bin");

/// First radiation constant c1 = 2 π h c², VA `0x4D4660`.
const PLANCK_C1: f64 = 3.74183e-16;
/// Second radiation constant c2 in meter-kelvin, VA `0x4D4658`.
const PLANCK_C2: f64 = 0.014388;
/// Wavelengths below this kelvin are replaced with this kelvin (`0x42A1C0`).
pub const MIN_KELVIN: i32 = 800;
/// Labeled slider stops run from 1200K through 6500K.
pub const SLIDER_MIN_KELVIN: i32 = 1200;
pub const SLIDER_MAX_KELVIN: i32 = 6500;

/// sRGB D65 matrix at VA `0x4C50EC`, row-major, applied to XYZ chromaticity.
const RGB_MATRIX: [[f64; 3]; 3] = [
    [
        3.2404541969299316,
        -1.5371384620666504,
        -0.49853140115737915,
    ],
    [-0.9692659974098206, 1.876010775566101, 0.04155600070953369],
    [0.05564339831471443, -0.20402589440345764, 1.057225227355957],
];

/// `c - (1-c)/1500` uses this constant (VA `0x4D49A8` and the threshold at `0x50D6F4`).
const ONE_OVER_1500: f64 = 0.0006666666595265269;
/// Exponent `1/2.2` at VA `0x4D49B8`.
const INV_GAMMA_22: f64 = 0.4545454680919647;

/// Minutes after midnight used when a saved wake time is missing. 420 is 7:00 AM.
pub const DEFAULT_WAKE_MINUTE: i32 = 420;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub latitude_hundredths: i32,
    pub longitude_hundredths: i32,
    pub zip_code: i32,
    pub day_kelvin: i32,
    pub night_kelvin: i32,
    pub late_kelvin: i32,
    /// Minutes after midnight. 585 is 9:45 AM.
    pub wake_minute: i32,
    pub basic: i32,
}

impl Settings {
    pub fn latitude_deg(self) -> f64 {
        f64::from(self.latitude_hundredths) / 100.0
    }

    pub fn longitude_deg(self) -> f64 {
        f64::from(self.longitude_hundredths) / 100.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SunStatus {
    /// `The sun is up. ` including the trailing space, mode 0 at `0x40CE10`.
    Up,
    /// `The sun has set. `, mode 1.
    Set,
}

/// Header for the top-right corner. Dawn and dusk are civil twilight:
/// the sun is below the horizon and no more than 6° down. Morning twilight
/// is dawn. Evening twilight is dusk. The color ramp still follows `SunStatus`.
pub fn sky_label(altitude_deg: f64, solar_hours: f64) -> &'static str {
    if altitude_deg >= 0.0 {
        "Daylight"
    } else if altitude_deg > -6.0 {
        if solar_hours < 12.0 {
            "Dawn"
        } else {
            "Dusk"
        }
    } else {
        "Nighttime"
    }
}

/// Channel gains from `0x42A1C0` (blackbody, chromaticity, matrix, `1/2.2` encode).
/// One channel is 1 and the others are at most 1. Values are the multipliers for the ramp.
pub fn kelvin_to_rgb(kelvin: i32) -> [f32; 3] {
    let kelvin = kelvin.max(MIN_KELVIN);
    let [x, y, z] = blackbody_xyz(f64::from(kelvin));
    let sum = x + y + z;
    if sum == 0.0 {
        return [1.0, 1.0, 1.0];
    }
    let chroma = [x / sum, y / sum, z / sum];
    let mut rgb = [0.0_f64; 3];
    for row in 0..3 {
        rgb[row] = RGB_MATRIX[row][0] * chroma[0]
            + RGB_MATRIX[row][1] * chroma[1]
            + RGB_MATRIX[row][2] * chroma[2];
    }
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    if max > 0.0 {
        for channel in &mut rgb {
            *channel /= max;
        }
    }
    for channel in &mut rgb {
        if *channel < 0.0 {
            *channel = 0.0;
        }
        *channel = encode_channel(*channel);
    }
    [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32]
}

fn cie_row(index: usize) -> [f32; 3] {
    let start = index * 12;
    let mut out = [0.0_f32; 3];
    for channel in 0..3 {
        let bytes: [u8; 4] = CIE_CMF[start + channel * 4..start + channel * 4 + 4]
            .try_into()
            .unwrap();
        out[channel] = f32::from_le_bytes(bytes);
    }
    out
}

/// `0x428DF0`. Wavelengths 360..=830 step 5, integrated against the CIE table.
pub fn blackbody_xyz(kelvin: f64) -> [f64; 3] {
    let mut xyz = [0.0_f64; 3];
    for i in 0..95 {
        let nm = 360.0 + (i as f64) * 5.0;
        let lambda = nm * 1e-9;
        let power = PLANCK_C1 * lambda.powf(-5.0);
        let radiance = power / ((PLANCK_C2 / (lambda * kelvin)).exp() - 1.0);
        let cmf = cie_row(i);
        xyz[0] += radiance * f64::from(cmf[0]);
        xyz[1] += radiance * f64::from(cmf[1]);
        xyz[2] += radiance * f64::from(cmf[2]);
    }
    xyz
}

/// Default path of `0x429F39` (the sRGB flag at `0x56805C` is clear in a fresh image).
fn encode_channel(linear: f64) -> f64 {
    if linear < ONE_OVER_1500 {
        return 0.0;
    }
    let adjusted = linear - (1.0 - linear) * ONE_OVER_1500;
    if adjusted <= 0.0 {
        return 0.0;
    }
    adjusted.powf(INV_GAMMA_22)
}

/// Identity sample `i * 0x101` from the filler at `0x45BAB2`, scaled by one channel gain.
/// `i * 257` is `i / 255 * 65535`. Gain 1 reproduces that identity.
pub fn gamma_channel(gain: f32) -> [u16; 256] {
    let mut out = [0_u16; 256];
    let gain = if gain.is_finite() { gain.max(0.0) } else { 0.0 };
    for i in 0..256 {
        let scaled = (i as f32) * 257.0 * gain;
        let clamped = scaled.clamp(0.0, 65535.0);
        out[i] = clamped as u16;
    }
    out
}

#[derive(Clone, Copy, Debug)]
pub struct SolarPosition {
    pub altitude_deg: f64,
    /// Hours after midnight, apparent solar time at `longitude_deg` (east positive).
    pub solar_hours: f64,
}

/// Paul Schlyter's solar position. The constants at `0x4D4768`, `0x4D4990`,
/// `0x4D4978`, `0x4D4968`, and `0x4D4980` are that algorithm's coefficients.
/// `when` is UTC.
pub fn solar_position(when: UtcTime, latitude_deg: f64, longitude_deg: f64) -> SolarPosition {
    let jd = julian_day(when);
    let d = jd - 2_451_543.5;
    let w = wrap_deg(282.9404 + 4.70935e-5 * d);
    let e = 0.016709 - 1.151e-9 * d;
    let mean = wrap_deg(356.0470 + 0.9856002585 * d);
    let mean_long = wrap_deg(w + mean);
    let mean_rad = mean.to_radians();
    let eccentric = mean + (e * mean_rad.sin() * (1.0 + e * mean_rad.cos())).to_degrees();
    let eccentric_rad = eccentric.to_radians();
    let xv = eccentric_rad.cos() - e;
    let yv = eccentric_rad.sin() * (1.0 - e * e).sqrt();
    let true_anom = yv.atan2(xv).to_degrees();
    let sun_lon = wrap_deg(true_anom + w);
    let obl = 23.4393 - 3.563e-7 * d;
    let lon_rad = sun_lon.to_radians();
    let obl_rad = obl.to_radians();
    let ra = wrap_deg(
        lon_rad
            .sin()
            .mul_add(obl_rad.cos(), 0.0)
            .atan2(lon_rad.cos())
            .to_degrees(),
    );
    let dec = lon_rad
        .sin()
        .mul_add(obl_rad.sin(), 0.0)
        .atan2(lon_rad.cos().hypot(lon_rad.sin() * obl_rad.cos()))
        .to_degrees();
    let ut = when.hour_fraction();
    let sidereal = wrap_deg(mean_long + 180.0 + ut * 15.0 + longitude_deg);
    let mut hour_angle = sidereal - ra;
    if hour_angle > 180.0 {
        hour_angle -= 360.0;
    } else if hour_angle < -180.0 {
        hour_angle += 360.0;
    }
    let lat = latitude_deg.to_radians();
    let dec_r = dec.to_radians();
    let ha = hour_angle.to_radians();
    let altitude = (lat.sin() * dec_r.sin() + lat.cos() * dec_r.cos() * ha.cos())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    let mut eot = mean_long - ra;
    if eot > 180.0 {
        eot -= 360.0;
    } else if eot < -180.0 {
        eot += 360.0;
    }
    let eot_min = eot * 4.0;
    let mut solar_hours = ut + longitude_deg / 15.0 + eot_min / 60.0;
    solar_hours = (solar_hours % 24.0 + 24.0) % 24.0;
    SolarPosition {
        altitude_deg: altitude,
        solar_hours,
    }
}

fn wrap_deg(value: f64) -> f64 {
    let mut wrapped = value % 360.0;
    if wrapped < 0.0 {
        wrapped += 360.0;
    }
    wrapped
}

/// Meeus Julian day for a UTC timestamp.
pub fn julian_day(when: UtcTime) -> f64 {
    let mut year = when.year;
    let mut month = when.month;
    let day = f64::from(when.day) + when.hour_fraction() / 24.0;
    if month <= 2 {
        year -= 1;
        month += 12;
    }
    let a = (f64::from(year) / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * f64::from(year + 4716)).floor() + (30.6001 * f64::from(month + 1)).floor() + day + b
        - 1524.5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UtcTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl UtcTime {
    pub fn hour_fraction(self) -> f64 {
        f64::from(self.hour) + f64::from(self.minute) / 60.0 + f64::from(self.second) / 3600.0
    }

    pub fn add_minutes(self, minutes: i32) -> Self {
        let mut total = self.hour as i32 * 60 + self.minute as i32 + minutes;
        let mut day = self.day as i32;
        let mut month = self.month;
        let mut year = self.year;
        while total < 0 {
            total += 24 * 60;
            day -= 1;
            if day < 1 {
                month -= 1;
                if month < 1 {
                    month = 12;
                    year -= 1;
                }
                day = days_in_month(year, month) as i32;
            }
        }
        while total >= 24 * 60 {
            total -= 24 * 60;
            day += 1;
            let dim = days_in_month(year, month) as i32;
            if day > dim {
                day = 1;
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
            }
        }
        Self {
            year,
            month,
            day: day as u32,
            hour: (total / 60) as u32,
            minute: (total % 60) as u32,
            second: self.second,
        }
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[derive(Clone, Copy, Debug)]
pub struct Schedule {
    pub status: SunStatus,
    pub kelvin: i32,
    pub solar_hours: f64,
    pub altitude_deg: f64,
    /// Minutes from `local` until the next sunset, or until the next sunrise when the sun is down.
    pub minutes_until_marker: i32,
    pub marker_is_sunset: bool,
    /// Minutes since the sun last dropped below the horizon. Zero while the sun is up.
    pub minutes_since_sunset: i32,
    /// Local minutes from midnight of today's sunset, the upcoming one while the sun is up.
    pub sunset_minute: i32,
    /// Local minutes from midnight of today's sunrise, the upcoming one while the sun is down.
    pub sunrise_minute: i32,
}

/// Day color while the solar altitude is non-negative, night color otherwise.
/// The sign test is the one around the call to `0x42B520` inside `0x42A850`.
/// The bedtime color and the "Getting sleepy?" mode are stored and documented;
/// the clock predicate that selects them was not reduced.
pub fn schedule(local: UtcTime, settings: Settings, utc_offset_minutes: i32) -> Schedule {
    let utc = local.add_minutes(-utc_offset_minutes);
    let solar = solar_position(utc, settings.latitude_deg(), settings.longitude_deg());
    let up = solar.altitude_deg >= 0.0;
    let status = if up { SunStatus::Up } else { SunStatus::Set };
    let kelvin = if up {
        settings.day_kelvin
    } else {
        settings.night_kelvin
    };
    let (minutes_until_marker, marker_is_sunset) =
        minutes_until_sun_event(local, settings, utc_offset_minutes, up);
    let minutes_since_sunset = if up {
        0
    } else {
        minutes_since_sunset(local, settings, utc_offset_minutes)
    };
    let now_minute = (local.hour * 60 + local.minute) as i32;
    let sunset_offset = if up {
        minutes_until_marker
    } else {
        -minutes_since_sunset
    };
    let sunset_minute = (now_minute + sunset_offset).rem_euclid(24 * 60);
    let sunrise_offset = if up {
        -minutes_since_sunrise(local, settings, utc_offset_minutes)
    } else {
        minutes_until_marker
    };
    let sunrise_minute = (now_minute + sunrise_offset).rem_euclid(24 * 60);
    Schedule {
        status,
        kelvin,
        solar_hours: solar.solar_hours,
        altitude_deg: solar.altitude_deg,
        minutes_until_marker,
        marker_is_sunset,
        minutes_since_sunset,
        sunset_minute,
        sunrise_minute,
    }
}

fn minutes_since_sunrise(local: UtcTime, settings: Settings, utc_offset_minutes: i32) -> i32 {
    for step in 1..(36 * 60) {
        let probe = local.add_minutes(-step);
        let utc = probe.add_minutes(-utc_offset_minutes);
        let alt =
            solar_position(utc, settings.latitude_deg(), settings.longitude_deg()).altitude_deg;
        if alt < 0.0 {
            return step;
        }
    }
    0
}

fn minutes_since_sunset(local: UtcTime, settings: Settings, utc_offset_minutes: i32) -> i32 {
    for step in 1..(36 * 60) {
        let probe = local.add_minutes(-step);
        let utc = probe.add_minutes(-utc_offset_minutes);
        let alt =
            solar_position(utc, settings.latitude_deg(), settings.longitude_deg()).altitude_deg;
        if alt >= 0.0 {
            return step;
        }
    }
    0
}

fn minutes_until_sun_event(
    local: UtcTime,
    settings: Settings,
    utc_offset_minutes: i32,
    sun_up: bool,
) -> (i32, bool) {
    for step in 1..(36 * 60) {
        let probe = local.add_minutes(step);
        let utc = probe.add_minutes(-utc_offset_minutes);
        let alt =
            solar_position(utc, settings.latitude_deg(), settings.longitude_deg()).altitude_deg;
        let up = alt >= 0.0;
        if up != sun_up {
            return (step, sun_up);
        }
    }
    (0, sun_up)
}

pub fn format_clock(total_minutes: i32) -> String {
    let minutes = total_minutes.rem_euclid(24 * 60);
    let hour24 = minutes / 60;
    let minute = minutes % 60;
    let suffix = if hour24 < 12 { "AM" } else { "PM" };
    let mut hour12 = hour24 % 12;
    if hour12 == 0 {
        hour12 = 12;
    }
    format!("{hour12}:{minute:02} {suffix}")
}

/// The next horizon crossing comes first. While the sun is up:
/// `Sunset should occur around 6:17 PM. Sunrise will occur at 7:15 AM tomorrow.`
/// After dark: `Sunrise will occur at 7:15 AM tomorrow. Sunset should occur around 6:17 PM tomorrow.`
/// A crossing on the next local calendar day is marked `tomorrow`. Before dawn both
/// crossings are later today, so neither one is.
pub fn status_line(schedule: &Schedule) -> String {
    let marker = if schedule.marker_is_sunset {
        schedule.sunset_minute
    } else {
        schedule.sunrise_minute
    };
    let now = (marker - schedule.minutes_until_marker).rem_euclid(24 * 60);
    let sun_up = schedule.status == SunStatus::Up;
    let sunset_tomorrow = if sun_up {
        schedule.sunset_minute < now
    } else {
        schedule.sunset_minute <= now
    };
    let sunrise_tomorrow = if sun_up {
        schedule.sunrise_minute <= now
    } else {
        schedule.sunrise_minute < now
    };
    let sunset = format!(
        "Sunset should occur around {}.",
        clock_with_day(schedule.sunset_minute, sunset_tomorrow)
    );
    let sunrise = format!(
        "Sunrise will occur at {}.",
        clock_with_day(schedule.sunrise_minute, sunrise_tomorrow)
    );
    if schedule.marker_is_sunset {
        format!("{sunset} {sunrise}")
    } else {
        format!("{sunrise} {sunset}")
    }
}

fn clock_with_day(minute: i32, tomorrow: bool) -> String {
    let when = format_clock(minute);
    if tomorrow {
        format!("{when} tomorrow")
    } else {
        when
    }
}

pub fn location_text(latitude_hundredths: i32, longitude_hundredths: i32) -> String {
    let lat = f64::from(latitude_hundredths) / 100.0;
    let lon = f64::from(longitude_hundredths) / 100.0;
    let ns = if lat >= 0.0 { "N" } else { "S" };
    let ew = if lon >= 0.0 { "E" } else { "W" };
    format!("Location: {:.1}°{ns}, {:.1}°{ew}", lat.abs(), lon.abs())
}

pub fn slider_fraction(kelvin: i32) -> f64 {
    let kelvin = kelvin.clamp(SLIDER_MIN_KELVIN, SLIDER_MAX_KELVIN);
    f64::from(kelvin - SLIDER_MIN_KELVIN) / f64::from(SLIDER_MAX_KELVIN - SLIDER_MIN_KELVIN)
}

pub fn kelvin_from_fraction(fraction: f64) -> i32 {
    let fraction = fraction.clamp(0.0, 1.0);
    let span = f64::from(SLIDER_MAX_KELVIN - SLIDER_MIN_KELVIN);
    let raw = f64::from(SLIDER_MIN_KELVIN) + fraction * span;
    (raw / 50.0).round() as i32 * 50
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(
        latitude_hundredths: i32,
        longitude_hundredths: i32,
        day_kelvin: i32,
        night_kelvin: i32,
        late_kelvin: i32,
        wake_minute: i32,
    ) -> Settings {
        Settings {
            latitude_hundredths,
            longitude_hundredths,
            zip_code: 0,
            day_kelvin,
            night_kelvin,
            late_kelvin,
            wake_minute,
            basic: 0,
        }
    }

    #[test]
    fn kelvin_rgb_matches_the_emulated_function() {
        let cases = [
            (1200, [1.0_f32, 0.27174, 0.0]),
            (1900, [1.0, 0.51472, 0.0]),
            (2700, [1.0, 0.67043, 0.34879]),
            (3400, [1.0, 0.76350, 0.51925]),
            (4800, [1.0, 0.88620, 0.77917]),
            (5900, [1.0, 0.94796, 0.92948]),
            (6500, [1.0, 0.97358, 0.99650]),
            (10000, [0.79816, 0.84748, 1.0]),
        ];
        for (kelvin, expected) in cases {
            let got = kelvin_to_rgb(kelvin);
            for i in 0..3 {
                assert!(
                    (got[i] - expected[i]).abs() < 0.0008,
                    "{kelvin} ch {i}: got {} expected {}",
                    got[i],
                    expected[i]
                );
            }
        }
        assert_eq!(kelvin_to_rgb(100), kelvin_to_rgb(800));
    }

    #[test]
    fn identity_ramp_is_i_times_257() {
        let ramp = gamma_channel(1.0);
        assert_eq!(ramp[0], 0);
        assert_eq!(ramp[1], 257);
        assert_eq!(ramp[255], 65535);
        let warm = gamma_channel(0.5);
        assert_eq!(warm[255], 32767);
    }

    #[test]
    fn hundredths_match_the_saved_location() {
        let settings = settings(3240, -9000, 6500, 3400, 1900, 585);
        assert!((settings.latitude_deg() - 32.4).abs() < 1e-9);
        assert!((settings.longitude_deg() - -90.0).abs() < 1e-9);
        assert_eq!(format_clock(settings.wake_minute), "9:45 AM");
        assert_eq!(location_text(3240, -9000), "Location: 32.4°N, 90.0°W");
    }

    #[test]
    fn afternoon_at_the_saved_location_is_daylight() {
        // 2026-10-24 15:17 CDT, which is 20:17 UTC. The handoff screenshot's
        // "Solar time: 2:32 PM" and "Sunset in 3 hours" sit on this afternoon.
        let local = UtcTime {
            year: 2026,
            month: 10,
            day: 24,
            hour: 15,
            minute: 17,
            second: 0,
        };
        let settings = settings(3240, -9000, 5900, 4800, 4800, 585);
        let planned = schedule(local, settings, -5 * 60);
        assert_eq!(planned.status, SunStatus::Up);
        assert_eq!(planned.kelvin, 5900);
        let line = status_line(&planned);
        let sunset =
            format_clock((15 * 60 + 17 + planned.minutes_until_marker).rem_euclid(24 * 60));
        let sunrise = format_clock(planned.sunrise_minute);
        assert!(planned.marker_is_sunset);
        assert_eq!(
            line,
            format!(
                "Sunset should occur around {sunset}. Sunrise will occur at {sunrise} tomorrow."
            )
        );
        assert!(!line.contains("Solar time"), "{line}");
        assert!(!line.contains("Wake"), "{line}");

        let evening = UtcTime {
            hour: 19,
            minute: 36,
            ..local
        };
        let night = schedule(evening, settings, -5 * 60);
        assert_eq!(night.status, SunStatus::Set);
        let night_line = status_line(&night);
        let sunset = format_clock((19 * 60 + 36 - night.minutes_since_sunset).rem_euclid(24 * 60));
        let sunrise = format_clock(night.sunrise_minute);
        assert!(!night.marker_is_sunset);
        assert_eq!(
            night_line,
            format!("Sunrise will occur at {sunrise} tomorrow. Sunset should occur around {sunset} tomorrow.")
        );

        let predawn = schedule(
            UtcTime {
                hour: 5,
                minute: 0,
                ..local
            },
            settings,
            -5 * 60,
        );
        assert_eq!(predawn.status, SunStatus::Set);
        assert!(!predawn.marker_is_sunset);
        let predawn_line = status_line(&predawn);
        assert_eq!(
            predawn_line,
            format!(
                "Sunrise will occur at {}. Sunset should occur around {}.",
                format_clock(predawn.sunrise_minute),
                format_clock(predawn.sunset_minute)
            )
        );
        assert_eq!(
            sky_label(planned.altitude_deg, planned.solar_hours),
            "Daylight"
        );
        assert_eq!(
            sky_label(night.altitude_deg, night.solar_hours),
            "Nighttime"
        );

        let mut saw_dawn = false;
        let mut saw_dusk = false;
        for minute in (0..24 * 60).step_by(2) {
            let sample = UtcTime {
                hour: minute / 60,
                minute: minute % 60,
                ..local
            };
            let planned = schedule(sample, settings, -5 * 60);
            match sky_label(planned.altitude_deg, planned.solar_hours) {
                "Dawn" => saw_dawn = true,
                "Dusk" => saw_dusk = true,
                _ => {}
            }
        }
        assert!(saw_dawn, "civil dawn did not appear");
        assert!(saw_dusk, "civil dusk did not appear");
    }

    #[test]
    fn sky_label_names_civil_twilight() {
        assert_eq!(sky_label(0.0, 18.0), "Daylight");
        assert_eq!(sky_label(-3.0, 6.5), "Dawn");
        assert_eq!(sky_label(-3.0, 18.5), "Dusk");
        assert_eq!(sky_label(-6.0, 19.0), "Nighttime");
        assert_eq!(sky_label(-12.0, 5.0), "Nighttime");
    }

    #[test]
    fn slider_quantizes_to_fifty_kelvin() {
        assert_eq!(kelvin_from_fraction(0.0), 1200);
        assert_eq!(kelvin_from_fraction(1.0), 6500);
        assert_eq!(slider_fraction(6500), 1.0);
        assert_eq!(slider_fraction(1200), 0.0);
    }
}
