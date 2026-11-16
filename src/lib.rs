//! Shared core of Beacon: the reminder list, the clocks, translations and
//! paths. Toolkit-free, so the background service never loads GTK.

pub mod autostart;
pub mod config;
pub mod i18n;
pub mod paths;
pub mod scheduler;

/// The window's application id (and desktop file, icon and resource path).
pub const APP_ID: &str = "io.github.mauneven.Beacon";
/// The background service's application id and D-Bus name.
pub const DAEMON_ID: &str = "io.github.mauneven.Beacon.Daemon";
pub const DBUS_PATH: &str = "/io/github/mauneven/Beacon/Daemon";
pub const DBUS_IFACE: &str = "io.github.mauneven.Beacon.Daemon1";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Introspection data of the service's D-Bus interface.
pub const DBUS_XML: &str = r#"
<node>
  <interface name="io.github.mauneven.Beacon.Daemon1">
    <!-- Every running clock: (reminder id, deadline in Unix milliseconds). -->
    <method name="GetDeadlines">
      <arg type="a(ux)" name="deadlines" direction="out"/>
    </method>
    <!-- Re-read the config file after the window saved it. -->
    <method name="Reload"/>
    <method name="Snooze">
      <arg type="u" name="id" direction="in"/>
    </method>
    <method name="Quit"/>
    <!-- The clocks changed: fetch them again. -->
    <signal name="Changed"/>
  </interface>
</node>
"#;

#[cfg(test)]
pub(crate) mod testutil {
    use std::path::{Path, PathBuf};

    pub struct TempDir(PathBuf);

    impl TempDir {
        pub fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("beacon-test-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
