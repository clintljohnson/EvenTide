//! Compiled Fluent catalogs. English is the source. A missing or empty
//! message falls back to English one string at a time.

use std::cell::RefCell;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

/// UI languages shipped with the program, in picker order.
/// The second column is the name shown in every language.
pub const UI_LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("es", "Español"),
    ("zh-CN", "中文"),
    ("pt-BR", "Português"),
    ("fr", "Français"),
    ("de", "Deutsch"),
    ("ja", "日本語"),
];

const EN: &str = include_str!("../assets/locale/en/main.ftl");
const ES: &str = include_str!("../assets/locale/es/main.ftl");
const ZH_CN: &str = include_str!("../assets/locale/zh-CN/main.ftl");
const PT_BR: &str = include_str!("../assets/locale/pt-BR/main.ftl");
const FR: &str = include_str!("../assets/locale/fr/main.ftl");
const DE: &str = include_str!("../assets/locale/de/main.ftl");
const JA: &str = include_str!("../assets/locale/ja/main.ftl");

struct Catalogs {
    tag: String,
    current: FluentBundle<FluentResource>,
    english: FluentBundle<FluentResource>,
}

thread_local! {
    static CATALOGS: RefCell<Option<Catalogs>> = const { RefCell::new(None) };
}

/// Map a Windows or saved tag onto a catalog we ship.
pub fn match_language(tag: &str) -> &'static str {
    let tag = tag.trim().replace('_', "-");
    let lower = tag.to_ascii_lowercase();
    if lower.starts_with("zh") {
        return "zh-CN";
    }
    if lower.starts_with("pt") {
        return "pt-BR";
    }
    for (code, _) in UI_LANGUAGES {
        if lower == code.to_ascii_lowercase() || lower.starts_with(&format!("{code}-").to_ascii_lowercase())
        {
            return code;
        }
    }
    let primary = lower.split('-').next().unwrap_or("");
    for (code, _) in UI_LANGUAGES {
        if code.to_ascii_lowercase() == primary {
            return code;
        }
    }
    "en"
}

pub fn current_language() -> String {
    with_catalogs(|catalogs| catalogs.tag.clone())
}

pub fn set_language(tag: &str) {
    let matched = match_language(tag);
    with_catalogs(|catalogs| {
        if catalogs.tag == matched {
            return;
        }
        catalogs.tag = matched.to_string();
        catalogs.current = bundle(matched);
    });
}

pub fn tr(id: &str) -> String {
    format_message(id, None)
}

pub fn tr_1(id: &str, key: &str, value: &str) -> String {
    let mut args = FluentArgs::new();
    args.set(key, value);
    format_message(id, Some(&args))
}

pub fn tr_2(id: &str, key: &str, value: &str, key_b: &str, value_b: &str) -> String {
    let mut args = FluentArgs::new();
    args.set(key, value);
    args.set(key_b, value_b);
    format_message(id, Some(&args))
}

fn format_message(id: &str, args: Option<&FluentArgs>) -> String {
    with_catalogs(|catalogs| {
        if catalogs.tag != "en" {
            if let Some(text) = format_from(&catalogs.current, id, args) {
                if !text.is_empty() {
                    return text;
                }
            }
        }
        if let Some(text) = format_from(&catalogs.english, id, args) {
            if !text.is_empty() {
                return text;
            }
        }
        id.to_string()
    })
}

fn format_from(
    bundle: &FluentBundle<FluentResource>,
    id: &str,
    args: Option<&FluentArgs>,
) -> Option<String> {
    let message = bundle.get_message(id)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, args, &mut errors);
    if !errors.is_empty() {
        return None;
    }
    Some(text.to_string())
}

fn with_catalogs<T>(body: impl FnOnce(&mut Catalogs) -> T) -> T {
    CATALOGS.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(Catalogs {
                tag: "en".to_string(),
                current: bundle("en"),
                english: bundle("en"),
            });
        }
        body(slot.as_mut().expect("catalogs"))
    })
}

fn bundle(tag: &str) -> FluentBundle<FluentResource> {
    let (source, lang) = match tag {
        "es" => (ES, "es"),
        "zh-CN" => (ZH_CN, "zh-CN"),
        "pt-BR" => (PT_BR, "pt-BR"),
        "fr" => (FR, "fr"),
        "de" => (DE, "de"),
        "ja" => (JA, "ja"),
        _ => (EN, "en"),
    };
    let langid: LanguageIdentifier = lang.parse().unwrap_or_else(|_| "en".parse().expect("en"));
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.set_use_isolating(false);
    let resource = FluentResource::try_new(source.to_string()).unwrap_or_else(|(_, errors)| {
        panic!("locale {tag} failed to parse: {errors:?}")
    });
    bundle.add_resource(resource).unwrap_or_else(|errors| {
        panic!("locale {tag} failed to load: {errors:?}")
    });
    bundle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_tags_map_onto_shipped_catalogs() {
        assert_eq!(match_language("en-US"), "en");
        assert_eq!(match_language("de-DE"), "de");
        assert_eq!(match_language("pt-PT"), "pt-BR");
        assert_eq!(match_language("zh-TW"), "zh-CN");
        assert_eq!(match_language("ja"), "ja");
        assert_eq!(match_language("sv-SE"), "en");
    }

    #[test]
    fn a_missing_message_uses_the_english_line() {
        set_language("ja");
        assert_eq!(tr("fallback-probe"), "English probe");
        assert_eq!(tr("not-a-real-message"), "not-a-real-message");
        assert_ne!(tr("sky-daylight"), "Daylight");
        set_language("en");
        assert_eq!(tr("sky-daylight"), "Daylight");
        assert_eq!(tr("fallback-probe"), "English probe");
    }
}
