//! Starting the background service at login, through an XDG autostart
//! entry (`~/.config/autostart`), which GNOME runs when the session starts.

use std::path::Path;

use crate::{APP_ID, i18n, paths};

pub fn is_enabled() -> bool {
    is_enabled_at(&paths::autostart_file())
}

pub fn set(enabled: bool) -> std::io::Result<()> {
    set_at(
        &paths::autostart_file(),
        enabled,
        &paths::sibling("beacon-daemon"),
    )
}

fn is_enabled_at(file: &Path) -> bool {
    match std::fs::read_to_string(file) {
        Ok(text) => !text.lines().any(|l| {
            let l = l.trim();
            l == "Hidden=true" || l == "X-GNOME-Autostart-enabled=false"
        }),
        Err(_) => false,
    }
}

/// Desktop-entry quoting for the `Exec` key.
fn quote(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-+".contains(c))
    {
        s.into_owned()
    } else {
        let escaped: String = s
            .chars()
            .flat_map(|c| match c {
                '"' | '`' | '$' | '\\' => vec!['\\', c],
                _ => vec![c],
            })
            .collect();
        format!("\"{escaped}\"")
    }
}

fn set_at(file: &Path, enabled: bool, daemon: &Path) -> std::io::Result<()> {
    if !enabled {
        return match std::fs::remove_file(file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let entry = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Beacon\n\
         Comment={comment}\n\
         Exec={exec} --gapplication-service\n\
         Icon={APP_ID}\n\
         NoDisplay=true\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n\
         X-GNOME-UsesNotifications=true\n",
        comment = i18n::tr("comments"),
        exec = quote(daemon),
    );
    std::fs::write(file, entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn toggles_the_entry() {
        let dir = TempDir::new("autostart");
        let file = dir.path().join("autostart/beacon.desktop");
        assert!(!is_enabled_at(&file));
        set_at(&file, true, Path::new("/usr/bin/beacon-daemon")).unwrap();
        assert!(is_enabled_at(&file));
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("Exec=/usr/bin/beacon-daemon --gapplication-service"));
        set_at(&file, false, Path::new("/x")).unwrap();
        assert!(!is_enabled_at(&file));
        set_at(&file, false, Path::new("/x")).unwrap();
    }

    #[test]
    fn quotes_odd_paths() {
        assert_eq!(
            quote(Path::new("/opt/My Apps/beacon")),
            "\"/opt/My Apps/beacon\""
        );
        assert_eq!(
            quote(Path::new("/usr/bin/beacon-daemon")),
            "/usr/bin/beacon-daemon"
        );
    }

    #[test]
    fn respects_a_disabled_entry() {
        let dir = TempDir::new("autostart-off");
        let file = dir.path().join("beacon.desktop");
        std::fs::write(&file, "[Desktop Entry]\nX-GNOME-Autostart-enabled=false\n").unwrap();
        assert!(!is_enabled_at(&file));
    }
}
