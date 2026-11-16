//! `beacon-daemon`: the background service. It owns the clocks and sends
//! the notifications. It links only GLib/GIO (no GTK), keeps a single timer
//! armed for the next reminder, and is otherwise idle.
//!
//! It is a D-Bus activatable `GApplication` (`--gapplication-service`), so
//! the window, a notification button, or the login autostart can start it.

mod app;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use beacon::config::{Config, Reminder};
use beacon::scheduler::Scheduler;
use beacon::{DAEMON_ID, DBUS_IFACE, DBUS_PATH, DBUS_XML, i18n, paths};
use gio::prelude::*;
use glib::Variant;

struct Service {
    app: app::DaemonApp,
    cfg: Config,
    sched: Scheduler,
    timer: Option<glib::SourceId>,
    hold: Option<gio::ApplicationHoldGuard>,
}

type Shared = Rc<RefCell<Service>>;

fn main() -> glib::ExitCode {
    let app = app::DaemonApp::new(DAEMON_ID);
    let service: Shared = Rc::new(RefCell::new(Service {
        app: app.clone(),
        cfg: Config::default(),
        sched: Scheduler::default(),
        timer: None,
        hold: None,
    }));

    let s = service.clone();
    app.connect_dbus_register(move |conn| export_interface(conn, &s));
    let s = service.clone();
    app.connect_startup(move |app| startup(app, &s));
    let s = service.clone();
    app.connect_activate(move |_| open_window(&s));
    app.run()
}

fn startup(app: &app::DaemonApp, service: &Shared) {
    {
        let mut s = service.borrow_mut();
        s.hold = Some(app.hold());
        s.cfg = Config::load().into_config();
        let now = SystemTime::now();
        let Service { sched, cfg, .. } = &mut *s;
        sched.restart_all(cfg, now);
    }

    // Notification buttons.
    let open = gio::SimpleAction::new("open", None);
    let s = service.clone();
    open.connect_activate(move |_, _| open_window(&s));
    app.add_action(&open);

    let snooze = gio::SimpleAction::new("snooze", Some(glib::VariantTy::UINT32));
    let s = service.clone();
    snooze.connect_activate(move |_, param| {
        if let Some(id) = param.and_then(Variant::get::<u32>) {
            snooze_reminder(&s, id);
        }
    });
    app.add_action(&snooze);

    watch_sleep(service);
    reschedule(service);
}

/// The interface the window talks to.
fn export_interface(conn: &gio::DBusConnection, service: &Shared) {
    let node = gio::DBusNodeInfo::for_xml(DBUS_XML).expect("valid introspection XML");
    let iface = node.lookup_interface(DBUS_IFACE).expect("interface in XML");
    let s = service.clone();
    let registered = conn
        .register_object(DBUS_PATH, &iface)
        .method_call(
            move |_conn, _sender, _path, _iface, method, params, invocation| match method {
                "GetDeadlines" => {
                    let deadlines = s.borrow().sched.export();
                    invocation.return_value(Some(&(deadlines,).to_variant()));
                }
                "Reload" => {
                    reload(&s);
                    invocation.return_value(None);
                }
                "Snooze" => {
                    if let Some((id,)) = params.get::<(u32,)>() {
                        snooze_reminder(&s, id);
                    }
                    invocation.return_value(None);
                }
                "Quit" => {
                    invocation.return_value(None);
                    quit(&s);
                }
                _ => invocation.return_dbus_error(
                    "org.freedesktop.DBus.Error.UnknownMethod",
                    &format!("No method {method}"),
                ),
            },
        )
        .build();
    if let Err(err) = registered {
        eprintln!("beacon-daemon: could not export {DBUS_PATH}: {err}");
    }
}

fn emit_changed(service: &Shared) {
    let app = service.borrow().app.clone();
    if let Some(conn) = app.dbus_connection() {
        let _ = conn.emit_signal(None, DBUS_PATH, DBUS_IFACE, "Changed", None);
    }
}

/// Arms one timer for the next deadline (no per-second ticking).
fn reschedule(service: &Shared) {
    let mut s = service.borrow_mut();
    if let Some(old) = s.timer.take() {
        old.remove();
    }
    let Some(at) = s.sched.next_deadline() else {
        return;
    };
    let wait = at
        .duration_since(SystemTime::now())
        .unwrap_or_default()
        // Land just after the deadline, never just before it.
        + Duration::from_millis(20);
    let weak = Rc::downgrade(service);
    s.timer = Some(glib::timeout_add_local_once(wait, move || {
        if let Some(service) = weak.upgrade() {
            service.borrow_mut().timer = None;
            fire(&service);
        }
    }));
}

fn fire(service: &Shared) {
    let (due, app, snooze) = {
        let mut s = service.borrow_mut();
        let Service { sched, cfg, .. } = &mut *s;
        let due: Vec<Reminder> = sched
            .tick(cfg, SystemTime::now())
            .into_iter()
            .filter_map(|id| cfg.get(id).cloned())
            .collect();
        (due, s.app.clone(), s.cfg.snooze_minutes)
    };
    for r in &due {
        notify(&app, r, snooze);
    }
    reschedule(service);
    emit_changed(service);
}

fn notification_id(id: u32) -> String {
    format!("reminder-{id}")
}

fn notify(app: &app::DaemonApp, r: &Reminder, snooze_minutes: u32) {
    let n = gio::Notification::new(&r.display_name());
    n.set_body(Some(&i18n::trf(
        "notification_body",
        &[("n", &r.minutes.to_string())],
    )));
    n.set_default_action("app.open");
    n.add_button_with_target_value(
        &i18n::trf("snooze", &[("n", &snooze_minutes.to_string())]),
        "app.snooze",
        Some(&r.id.to_variant()),
    );
    app.send_notification(Some(&notification_id(r.id)), &n);
}

fn reload(service: &Shared) {
    {
        let mut s = service.borrow_mut();
        let new = Config::load().into_config();
        let old = std::mem::replace(&mut s.cfg, new);
        let Service {
            sched, cfg, app, ..
        } = &mut *s;
        sched.apply(&old, cfg, SystemTime::now());
        // Withdraw what can no longer be snoozed (deleted, turned off, paused).
        for r in &old.reminders {
            if cfg.paused || cfg.get(r.id).is_none_or(|n| !n.enabled) {
                app.withdraw_notification(&notification_id(r.id));
            }
        }
    }
    reschedule(service);
    emit_changed(service);
}

fn snooze_reminder(service: &Shared, id: u32) {
    {
        let mut s = service.borrow_mut();
        let by = Duration::from_secs(u64::from(s.cfg.snooze_minutes) * 60);
        s.sched.snooze(id, by, SystemTime::now());
        s.app.withdraw_notification(&notification_id(id));
    }
    reschedule(service);
    emit_changed(service);
}

/// After waking from sleep, every clock starts over quietly.
fn watch_sleep(service: &Shared) {
    let Ok(system) = gio::bus_get_sync(gio::BusType::System, gio::Cancellable::NONE) else {
        return;
    };
    let weak = Rc::downgrade(service);
    #[allow(deprecated)]
    system.signal_subscribe(
        Some("org.freedesktop.login1"),
        Some("org.freedesktop.login1.Manager"),
        Some("PrepareForSleep"),
        Some("/org/freedesktop/login1"),
        None,
        gio::DBusSignalFlags::NONE,
        move |_, _, _, _, _, params| {
            let Some(service) = weak.upgrade() else {
                return;
            };
            if let Some((false,)) = params.get::<(bool,)>() {
                {
                    let mut s = service.borrow_mut();
                    let Service { sched, cfg, .. } = &mut *s;
                    sched.restart_all(cfg, SystemTime::now());
                }
                reschedule(&service);
                emit_changed(&service);
            }
        },
    );
}

/// Opens the window, handing over the activation token so GNOME focuses it.
fn open_window(service: &Shared) {
    let app = service.borrow().app.clone();
    let token = app.take_activation_token();
    let sibling = paths::sibling("beacon");
    let mut cmd = std::process::Command::new(&sibling);
    if let Some(token) = &token {
        cmd.env("XDG_ACTIVATION_TOKEN", token);
        cmd.env("DESKTOP_STARTUP_ID", token);
    }
    match cmd.spawn() {
        Ok(mut child) => {
            // Reap it whenever it closes.
            std::thread::spawn(move || child.wait());
        }
        Err(err) => eprintln!("beacon-daemon: could not open the window: {err}"),
    }
}

fn quit(service: &Shared) {
    let mut s = service.borrow_mut();
    if let Some(timer) = s.timer.take() {
        timer.remove();
    }
    s.hold = None;
    s.app.quit();
}
