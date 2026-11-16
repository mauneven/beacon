//! Translations. Each language is a small TOML table in `i18n/`, compiled
//! into the binaries. The language comes from the usual locale variables
//! (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`), like any GNOME app.
//! Missing keys fall back to English.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

const LANGUAGES: &[(&str, &str)] = &[
    ("en", include_str!("../i18n/en.toml")),
    ("de", include_str!("../i18n/de.toml")),
    ("es", include_str!("../i18n/es.toml")),
    ("fr", include_str!("../i18n/fr.toml")),
    ("it", include_str!("../i18n/it.toml")),
    ("pt", include_str!("../i18n/pt.toml")),
];

/// Built-in reminders: (preset, minutes, on by default).
pub const PRESETS: &[(&str, u32, bool)] = &[
    ("water", 20, true),
    ("hands", 25, false),
    ("legs", 45, true),
    ("eyes", 20, true),
    ("posture", 30, true),
];

type Table = HashMap<String, String>;

fn parse(code: &str) -> Table {
    LANGUAGES
        .iter()
        .find(|(c, _)| *c == code)
        .and_then(|(_, src)| toml::from_str(src).ok())
        .unwrap_or_default()
}

static ENGLISH: LazyLock<Table> = LazyLock::new(|| parse("en"));
static CURRENT: LazyLock<RwLock<(String, Table)>> =
    LazyLock::new(|| RwLock::new(load(&from_env())));

fn load(code: &str) -> (String, Table) {
    (code.to_string(), parse(code))
}

/// The first supported language in a list of locale names such as
/// `es_CO.UTF-8` or `pt_BR`.
pub fn pick<'a>(locales: impl IntoIterator<Item = &'a str>) -> &'static str {
    for locale in locales {
        let base = locale
            .split(['.', '@'])
            .next()
            .unwrap_or("")
            .split(['_', '-'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if base == "c" || base == "posix" || base.is_empty() {
            continue;
        }
        if let Some((code, _)) = LANGUAGES.iter().find(|(c, _)| *c == base) {
            return code;
        }
    }
    "en"
}

fn from_env() -> String {
    let mut locales = Vec::new();
    // LANGUAGE is a priority list, but only when a locale is set at all.
    let lang_set = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .any(|v| std::env::var(v).is_ok_and(|s| !s.is_empty() && s != "C"));
    if lang_set && let Ok(list) = std::env::var("LANGUAGE") {
        locales.extend(list.split(':').map(str::to_owned));
    }
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(var)
            && !v.is_empty()
        {
            locales.push(v);
            break;
        }
    }
    pick(locales.iter().map(String::as_str)).to_string()
}

/// Switches the language (tests, mostly).
pub fn set_language(code: &str) {
    *CURRENT.write().unwrap() = load(pick([code]));
}

pub fn language() -> String {
    CURRENT.read().unwrap().0.clone()
}

fn lookup(key: &str) -> Option<String> {
    CURRENT
        .read()
        .unwrap()
        .1
        .get(key)
        .or_else(|| ENGLISH.get(key))
        .cloned()
}

pub fn tr(key: &str) -> String {
    lookup(key).unwrap_or_else(|| key.to_string())
}

/// `tr` with `{name}` placeholders filled in.
pub fn trf(key: &str, args: &[(&str, &str)]) -> String {
    let mut text = tr(key);
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

pub fn preset_name(preset: &str) -> Option<String> {
    lookup(&format!("preset_{preset}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_from_locale_names() {
        assert_eq!(pick(["es_CO.UTF-8"]), "es");
        assert_eq!(pick(["pt_BR"]), "pt");
        assert_eq!(pick(["C", "de_DE@euro"]), "de");
        assert_eq!(pick(["ja_JP.UTF-8"]), "en");
        assert_eq!(pick([]), "en");
    }

    #[test]
    fn every_language_has_every_key() {
        let english = parse("en");
        for (code, _) in LANGUAGES {
            let table = parse(code);
            assert!(!table.is_empty(), "{code} does not parse");
            for key in english.keys() {
                assert!(table.contains_key(key), "{code} is missing {key}");
                let want: Vec<_> = english[key].matches('{').collect();
                let got: Vec<_> = table[key].matches('{').collect();
                assert_eq!(want.len(), got.len(), "{code}.{key} placeholders");
            }
        }
    }

    #[test]
    fn every_preset_has_a_name() {
        for (preset, _, _) in PRESETS {
            assert!(ENGLISH.contains_key(&format!("preset_{preset}")));
        }
    }
}
