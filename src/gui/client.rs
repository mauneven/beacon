//! Talks to `beacon-daemon` over the session bus. The service is D-Bus
//! activatable, so the first call starts it; when Beacon runs from a build
//! tree (no service file installed) the window starts it by hand.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use beacon::scheduler::from_millis;
use beacon::{DAEMON_ID, DBUS_IFACE, DBUS_PATH, paths};
use gtk::{gio, glib};

pub type Deadlines = HashMap<u32, SystemTime>;

#[derive(Clone)]
pub struct Client {
    conn: gio::DBusConnection,
}

impl Client {
    pub fn new(conn: gio::DBusConnection) -> Self {
        Client { conn }
    }

    fn call(
        &self,
        method: &str,
        args: Option<&glib::Variant>,
        reply: Option<&glib::VariantTy>,
        done: impl FnOnce(Result<glib::Variant, glib::Error>) + 'static,
    ) {
        self.conn.call(
            Some(DAEMON_ID),
            DBUS_PATH,
            DBUS_IFACE,
            method,
            args,
            reply,
            gio::DBusCallFlags::NONE,
            5000,
            gio::Cancellable::NONE,
            done,
        );
    }

    pub fn fetch(&self, done: impl FnOnce(Result<Deadlines, glib::Error>) + 'static) {
        let ty = glib::VariantTy::new("(a(ux))").expect("valid type");
        self.call("GetDeadlines", None, Some(ty), move |res| {
            done(res.map(|v| {
                let (list,) = v.get::<(Vec<(u32, i64)>,)>().unwrap_or_default();
                list.into_iter()
                    .map(|(id, ms)| (id, from_millis(ms)))
                    .collect()
            }))
        });
    }

    pub fn reload(&self) {
        self.call("Reload", None, None, |res| {
            if let Err(err) = res {
                eprintln!("beacon: reload failed: {err}");
            }
        });
    }

    pub fn quit(&self, done: impl FnOnce() + 'static) {
        self.call("Quit", None, None, move |_| done());
    }

    /// Calls `f` whenever the service's clocks change.
    pub fn on_changed(&self, f: impl Fn() + 'static) {
        #[allow(deprecated)]
        self.conn.signal_subscribe(
            Some(DAEMON_ID),
            Some(DBUS_IFACE),
            Some("Changed"),
            Some(DBUS_PATH),
            None,
            gio::DBusSignalFlags::NONE,
            move |_, _, _, _, _, _| f(),
        );
    }

    /// Calls `f` whenever the service (re)appears on the bus.
    pub fn on_started(&self, f: impl Fn() + 'static) {
        // The watch lives as long as the window, so it is never removed.
        let _ = gio::bus_watch_name_on_connection(
            &self.conn,
            DAEMON_ID,
            gio::BusNameWatcherFlags::NONE,
            move |_, _, _| f(),
            |_, _| {},
        );
    }

    /// Starts the service from next to this binary.
    pub fn spawn_daemon() -> std::io::Result<()> {
        let mut child = Command::new(paths::sibling("beacon-daemon"))
            .arg("--gapplication-service")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;
        std::thread::spawn(move || child.wait());
        Ok(())
    }
}
