//! `~/.config/beacon/config.toml`: the reminder list and a few settings.
//!
//! The window is the only writer; the background service reads the file
//! when the window asks it to (`Reload`).
//!
//! A built-in reminder (`preset = "water"`) has no stored name, so it shows
//! in the desktop's language. Once renamed, the text is stored in `name` and
//! never translated again.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

use crate::{i18n, paths};

pub const MIN_MINUTES: u32 = 1;
pub const MAX_MINUTES: u32 = 120;
pub const MIN_SNOOZE: u32 = 1;
pub const MAX_SNOOZE: u32 = 60;

fn yes() -> bool {
    true
}

fn default_snooze() -> u32 {
    5
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Reminder {
    pub id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub minutes: u32,
    #[serde(default = "yes")]
    pub enabled: bool,
}

impl Reminder {
    pub fn preset(id: u32, preset: &str, minutes: u32, enabled: bool) -> Self {
        Reminder {
            id,
            preset: Some(preset.to_string()),
            name: None,
            minutes,
            enabled,
        }
    }

    pub fn custom(id: u32, name: &str, minutes: u32) -> Self {
        Reminder {
            id,
            preset: None,
            name: Some(name.to_string()),
            minutes: minutes.clamp(MIN_MINUTES, MAX_MINUTES),
            enabled: true,
        }
    }

    /// The user's own name, or the preset's name in the current language.
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.preset.as_deref().and_then(i18n::preset_name))
            .unwrap_or_else(|| "Beacon".into())
    }

    /// Text typed in the name field. Typing the preset's own translation
    /// back keeps the reminder localized; anything else is a custom name.
    pub fn rename(&mut self, text: &str) {
        let text = text.trim();
        let localized = self.preset.as_deref().and_then(i18n::preset_name);
        if localized.as_deref() == Some(text) {
            self.name = None;
        } else {
            self.name = Some(text.to_string());
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub paused: bool,
    #[serde(default = "default_snooze")]
    pub snooze_minutes: u32,
    #[serde(default, rename = "reminder")]
    pub reminders: Vec<Reminder>,
}

impl Default for Config {
    fn default() -> Self {
        let reminders = i18n::PRESETS
            .iter()
            .enumerate()
            .map(|(i, (preset, minutes, enabled))| {
                Reminder::preset(i as u32 + 1, preset, *minutes, *enabled)
            })
            .collect();
        Config {
            paused: false,
            snooze_minutes: default_snooze(),
            reminders,
        }
    }
}

pub enum Loaded {
    /// No config existed; defaults were written.
    FirstRun(Config),
    Existing(Config),
}

impl Loaded {
    pub fn into_config(self) -> Config {
        match self {
            Loaded::FirstRun(c) | Loaded::Existing(c) => c,
        }
    }
}

impl Config {
    pub fn load() -> Loaded {
        Config::load_from(&paths::config_file())
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&paths::config_file())
    }

    /// Reads a config file. A missing file is a first run: defaults are
    /// written. A broken file is kept aside as `config.toml.broken`.
    pub fn load_from(path: &Path) -> Loaded {
        match std::fs::read_to_string(path) {
            Ok(raw) => match toml::from_str::<Config>(&raw) {
                Ok(mut cfg) => {
                    cfg.sanitize();
                    Loaded::Existing(cfg)
                }
                Err(err) => {
                    let backup = path.with_extension("toml.broken");
                    let _ = std::fs::rename(path, &backup);
                    eprintln!(
                        "beacon: {} is invalid ({err}); moved it to {} and started fresh",
                        path.display(),
                        backup.display()
                    );
                    let cfg = Config::default();
                    let _ = cfg.save_to(path);
                    Loaded::Existing(cfg)
                }
            },
            Err(_) => {
                let cfg = Config::default();
                if let Err(err) = cfg.save_to(path) {
                    eprintln!("beacon: could not write {}: {err}", path.display());
                }
                Loaded::FirstRun(cfg)
            }
        }
    }

    fn sanitize(&mut self) {
        self.snooze_minutes = self.snooze_minutes.clamp(MIN_SNOOZE, MAX_SNOOZE);
        let mut seen = HashSet::new();
        let mut next = self.reminders.iter().map(|r| r.id).max().unwrap_or(0);
        for r in &mut self.reminders {
            r.minutes = r.minutes.clamp(MIN_MINUTES, MAX_MINUTES);
            if r.id == 0 || !seen.insert(r.id) {
                next += 1;
                r.id = next;
                seen.insert(r.id);
            }
        }
    }

    /// Atomic write: a crash mid-save never leaves a half-written file.
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let body = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension("toml.tmp");
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(
                b"# Beacon reminders: every `minutes`, a notification, then the clock restarts.\n\
                  # `preset` reminders follow the desktop language until you give them a `name`.\n\n",
            )?;
            f.write_all(body.as_bytes())?;
            f.sync_all()?;
        }
        std::fs::rename(tmp, path)
    }

    pub fn next_id(&self) -> u32 {
        self.reminders.iter().map(|r| r.id).max().unwrap_or(0) + 1
    }

    pub fn get(&self, id: u32) -> Option<&Reminder> {
        self.reminders.iter().find(|r| r.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Reminder> {
        self.reminders.iter_mut().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn roundtrip_never_stores_preset_names() {
        let cfg = Config::default();
        let text = toml::to_string_pretty(&cfg).unwrap();
        assert!(!text.contains("name ="));
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn rename_keeps_or_breaks_localization() {
        let mut r = Reminder::preset(1, "water", 20, true);
        let localized = r.display_name();
        r.rename(&format!(" {localized} "));
        assert_eq!(r.name, None);
        r.rename("Agua!");
        assert_eq!(r.name.as_deref(), Some("Agua!"));
        assert_eq!(r.display_name(), "Agua!");
    }

    #[test]
    fn sanitize_fixes_ids_and_ranges() {
        let mut cfg: Config = toml::from_str(
            r#"
            snooze_minutes = 0
            [[reminder]]
            id = 1
            name = "a"
            minutes = 0
            [[reminder]]
            id = 1
            name = "b"
            minutes = 999
            "#,
        )
        .unwrap();
        cfg.sanitize();
        assert!(!cfg.paused);
        assert_eq!(cfg.snooze_minutes, MIN_SNOOZE);
        assert_eq!(cfg.reminders[0].minutes, MIN_MINUTES);
        assert_eq!(cfg.reminders[1].minutes, MAX_MINUTES);
        assert_ne!(cfg.reminders[0].id, cfg.reminders[1].id);
        assert!(cfg.reminders[1].enabled);
    }

    #[test]
    fn first_run_writes_the_defaults() {
        let dir = TempDir::new("first-run");
        let path = dir.path().join("beacon/config.toml");
        let Loaded::FirstRun(cfg) = Config::load_from(&path) else {
            panic!("a missing file is a first run");
        };
        assert_eq!(cfg, Config::default());
        assert!(path.is_file());
        let Loaded::Existing(again) = Config::load_from(&path) else {
            panic!("the second start is not a first run");
        };
        assert_eq!(again, cfg);
    }

    #[test]
    fn edits_survive_a_restart() {
        let dir = TempDir::new("edits");
        let path = dir.path().join("config.toml");
        let mut cfg = Config {
            paused: true,
            snooze_minutes: 10,
            ..Config::default()
        };
        cfg.reminders[0].minutes = 55;
        cfg.reminders[1].enabled = true;
        cfg.reminders[2].rename("Walk the dog");
        cfg.reminders.remove(3);
        let id = cfg.next_id();
        cfg.reminders.push(Reminder::custom(id, "Feed the cat", 90));
        cfg.save_to(&path).unwrap();
        let Loaded::Existing(back) = Config::load_from(&path) else {
            panic!()
        };
        assert_eq!(back, cfg);
        assert!(!dir.path().join("config.toml.tmp").exists());
    }

    #[test]
    fn a_broken_file_is_kept_aside() {
        let dir = TempDir::new("broken");
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is [not toml").unwrap();
        let cfg = Config::load_from(&path).into_config();
        assert_eq!(cfg, Config::default());
        let kept = dir.path().join("config.toml.broken");
        assert_eq!(std::fs::read_to_string(kept).unwrap(), "this is [not toml");
    }

    #[test]
    fn next_id_never_reuses_an_id() {
        let mut cfg = Config::default();
        let max = cfg.reminders.iter().map(|r| r.id).max().unwrap();
        assert_eq!(cfg.next_id(), max + 1);
        cfg.reminders.clear();
        assert_eq!(cfg.next_id(), 1);
    }
}
