//! Where Beacon keeps its files, following the XDG base directories.

use std::path::PathBuf;

fn xdg_config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/".into())).join(".config")
        })
}

/// `~/.config/beacon/config.toml`
pub fn config_file() -> PathBuf {
    xdg_config_home().join("beacon").join("config.toml")
}

/// `~/.config/autostart/io.github.mauneven.Beacon.Daemon.desktop`
pub fn autostart_file() -> PathBuf {
    xdg_config_home()
        .join("autostart")
        .join(format!("{}.desktop", crate::DAEMON_ID))
}

/// A program installed next to the running one (`beacon` and
/// `beacon-daemon` live in the same directory), or the bare name so `PATH`
/// is searched.
pub fn sibling(name: &str) -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}
