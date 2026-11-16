//! The main window: one switch for everything, then one line per reminder
//! with a check box, its name and a slider for the interval.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use adw::prelude::*;
use beacon::config::{Config, Loaded, MAX_MINUTES, MIN_MINUTES, Reminder};
use beacon::i18n::{tr, trf};
use beacon::scheduler::clock;
use beacon::{APP_ID, autostart};
use gtk::{gio, glib};

use crate::client::{Client, Deadlines};
use crate::{dialogs, editor};

/// Wait this long after the last slider movement before saving.
const SAVE_DELAY: Duration = Duration::from_millis(400);

struct Row {
    id: u32,
    check: gtk::CheckButton,
    name: gtk::Button,
    scale: gtk::Scale,
    value: gtk::Label,
}

pub struct Ui {
    pub app: adw::Application,
    pub window: adw::ApplicationWindow,
    toasts: adw::ToastOverlay,
    title: adw::WindowTitle,
    master: gtk::Switch,
    dnd: adw::Banner,
    heading: gtk::Label,
    grid: gtk::Grid,
    empty: adw::StatusPage,
    rows: RefCell<Vec<Row>>,
    pub cfg: RefCell<Config>,
    deadlines: RefCell<Deadlines>,
    client: RefCell<Option<Client>>,
    save_timer: RefCell<Option<glib::SourceId>>,
    /// Set while widgets are updated from the model, so their handlers
    /// don't write the change back.
    syncing: Cell<bool>,
}

fn minutes_label(minutes: u32) -> String {
    if minutes >= 60 && minutes.is_multiple_of(60) {
        trf("hours_short", &[("n", &(minutes / 60).to_string())])
    } else {
        trf("minutes_short", &[("n", &minutes.to_string())])
    }
}

/// GNOME's Do Not Disturb setting, when the schema is there.
fn notification_settings() -> Option<gio::Settings> {
    let schema = "org.gnome.desktop.notifications";
    gio::SettingsSchemaSource::default()?.lookup(schema, true)?;
    Some(gio::Settings::new(schema))
}

impl Ui {
    pub fn build(app: &adw::Application) -> Rc<Ui> {
        let cfg = match Config::load() {
            Loaded::FirstRun(cfg) => {
                // Start with the session by default; Preferences turns it off.
                if let Err(err) = autostart::set(true) {
                    eprintln!("beacon: could not enable autostart: {err}");
                }
                cfg
            }
            Loaded::Existing(cfg) => cfg,
        };

        // Header bar
        let add = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text(tr("add_reminder"))
            .action_name("win.add")
            .build();
        let menu = gio::Menu::new();
        let main = gio::Menu::new();
        main.append(Some(&tr("preferences")), Some("app.preferences"));
        main.append(Some(&tr("shortcuts")), Some("app.shortcuts"));
        main.append(Some(&tr("about")), Some("app.about"));
        menu.append_section(None, &main);
        let end = gio::Menu::new();
        end.append(Some(&tr("quit")), Some("app.quit"));
        menu.append_section(None, &end);
        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu)
            .primary(true)
            .tooltip_text(tr("main_menu"))
            .build();
        let title = adw::WindowTitle::new("Beacon", "");
        let header = adw::HeaderBar::new();
        header.pack_start(&add);
        header.pack_end(&menu_button);
        header.set_title_widget(Some(&title));

        let dnd = adw::Banner::builder()
            .title(tr("dnd_banner"))
            .button_label(tr("dnd_turn_off"))
            .build();

        // The master switch
        let master_title = gtk::Label::builder()
            .label(tr("reminders_title"))
            .xalign(0.0)
            .css_classes(["title-3"])
            .build();
        let master_subtitle = gtk::Label::builder()
            .label(tr("reminders_subtitle"))
            .xalign(0.0)
            .wrap(true)
            .css_classes(["dim-label"])
            .build();
        let master_text = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();
        master_text.append(&master_title);
        master_text.append(&master_subtitle);
        let master = gtk::Switch::builder()
            .active(!cfg.paused)
            .valign(gtk::Align::Center)
            .build();
        master.update_property(&[gtk::accessible::Property::Label(&tr("reminders_title"))]);
        let master_line = gtk::Box::builder().spacing(18).build();
        master_line.append(&master_text);
        master_line.append(&master);

        let heading = gtk::Label::builder()
            .label(tr("how_often"))
            .xalign(0.0)
            .wrap(true)
            .margin_top(12)
            .css_classes(["heading"])
            .build();

        // The scale's end points, above the sliders.
        let marks = gtk::Box::builder()
            .homogeneous(true)
            .margin_start(12)
            .build();
        for (minutes, xalign) in [
            (MIN_MINUTES, 0.0),
            (MAX_MINUTES / 2, 0.5),
            (MAX_MINUTES, 1.0),
        ] {
            marks.append(
                &gtk::Label::builder()
                    .label(minutes_label(minutes))
                    .xalign(xalign)
                    .css_classes(["dim-label", "numeric"])
                    .build(),
            );
        }
        let grid = gtk::Grid::builder()
            .column_spacing(6)
            .row_spacing(10)
            .build();
        grid.attach(&marks, 2, 0, 1, 1);

        let empty = adw::StatusPage::builder()
            .icon_name(format!("{APP_ID}-symbolic"))
            .title(tr("empty_title"))
            .description(tr("empty_description"))
            .css_classes(["compact"])
            .vexpand(true)
            .child(
                &gtk::Button::builder()
                    .label(tr("add_reminder"))
                    .halign(gtk::Align::Center)
                    .css_classes(["pill", "suggested-action"])
                    .action_name("win.add")
                    .build(),
            )
            .build();

        let note = gtk::Label::builder()
            .label(tr("background_note"))
            .xalign(0.0)
            .wrap(true)
            .margin_top(12)
            .css_classes(["dim-label"])
            .build();

        let column = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();
        column.append(&master_line);
        column.append(&heading);
        column.append(&grid);
        column.append(&empty);
        column.append(&note);
        let clamp = adw::Clamp::builder()
            .maximum_size(640)
            .child(&column)
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&clamp)
            .build();

        let view = adw::ToolbarView::new();
        view.add_top_bar(&header);
        view.add_top_bar(&dnd);
        view.set_content(Some(&scroller));
        let toasts = adw::ToastOverlay::new();
        toasts.set_child(Some(&view));

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Beacon")
            .default_width(600)
            .default_height(580)
            .width_request(360)
            .height_request(294)
            .content(&toasts)
            .build();

        let ui = Rc::new(Ui {
            app: app.clone(),
            window,
            toasts,
            title,
            master,
            dnd,
            heading,
            grid,
            empty,
            rows: RefCell::new(Vec::new()),
            cfg: RefCell::new(cfg),
            deadlines: RefCell::new(Deadlines::new()),
            client: RefCell::new(None),
            save_timer: RefCell::new(None),
            syncing: Cell::new(false),
        });

        ui.connect_signals();
        ui.watch_do_not_disturb();
        ui.add_actions();
        ui.rebuild();
        ui.connect_service();
        ui.start_ticking();
        ui
    }

    fn connect_signals(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.master.connect_active_notify(move |switch| {
            let Some(ui) = weak.upgrade() else { return };
            if ui.syncing.get() {
                return;
            }
            ui.cfg.borrow_mut().paused = !switch.is_active();
            ui.commit();
        });
        // Never lose a slider change that is still waiting to be saved.
        let weak = Rc::downgrade(self);
        self.window.connect_close_request(move |_| {
            if let Some(ui) = weak.upgrade() {
                ui.flush();
            }
            glib::Propagation::Proceed
        });
    }

    /// Shows a banner while Do Not Disturb hides the reminders.
    fn watch_do_not_disturb(self: &Rc<Self>) {
        let Some(settings) = notification_settings() else {
            return;
        };
        let banner = self.dnd.clone();
        let update = move |s: &gio::Settings| banner.set_revealed(!s.boolean("show-banners"));
        update(&settings);
        settings.connect_changed(Some("show-banners"), move |s, _| update(s));
        let s = settings.clone();
        self.dnd.connect_button_clicked(move |_| {
            let _ = s.set_boolean("show-banners", true);
        });
        // Keep the settings object (and its signal) alive with the window.
        unsafe { self.window.set_data("notification-settings", settings) };
    }

    fn add_actions(self: &Rc<Self>) {
        let add = gio::SimpleAction::new("add", None);
        let weak = Rc::downgrade(self);
        add.connect_activate(move |_, _| {
            if let Some(ui) = weak.upgrade() {
                editor::open(&ui, None);
            }
        });
        self.window.add_action(&add);

        let toggle = gio::SimpleAction::new("toggle-pause", None);
        let master = self.master.clone();
        toggle.connect_activate(move |_, _| master.set_active(!master.is_active()));
        self.window.add_action(&toggle);

        let app_action = |name: &str, f: fn(&Rc<Ui>)| {
            let action = gio::SimpleAction::new(name, None);
            let weak = Rc::downgrade(self);
            action.connect_activate(move |_, _| {
                if let Some(ui) = weak.upgrade() {
                    f(&ui);
                }
            });
            self.app.add_action(&action);
        };
        app_action("preferences", dialogs::preferences);
        app_action("shortcuts", dialogs::shortcuts);
        app_action("about", dialogs::about);
        app_action("quit", |ui| {
            // Quit stops the reminders too (closing the window doesn't).
            ui.flush();
            let app = ui.app.clone();
            match ui.client.borrow().clone() {
                Some(client) => client.quit(move || app.quit()),
                None => app.quit(),
            }
        });
    }

    /// Hooks up to the background service, starting it if needed.
    fn connect_service(self: &Rc<Self>) {
        let Some(conn) = self.app.dbus_connection() else {
            self.toast(&tr("service_error"));
            return;
        };
        let client = Client::new(conn);
        let weak = Rc::downgrade(self);
        client.on_changed(move || {
            if let Some(ui) = weak.upgrade() {
                ui.fetch();
            }
        });
        let weak = Rc::downgrade(self);
        client.on_started(move || {
            if let Some(ui) = weak.upgrade() {
                ui.fetch();
            }
        });
        self.client.replace(Some(client.clone()));

        // The first call D-Bus-activates the service when it's installed.
        let weak = Rc::downgrade(self);
        client.fetch(move |res| {
            let Some(ui) = weak.upgrade() else { return };
            match res {
                Ok(deadlines) => {
                    ui.set_deadlines(deadlines);
                    // Picks up hand edits made to the config file.
                    if let Some(client) = ui.client.borrow().as_ref() {
                        client.reload();
                    }
                }
                Err(_) => {
                    if let Err(err) = Client::spawn_daemon() {
                        eprintln!("beacon: could not start beacon-daemon: {err}");
                        ui.toast(&tr("service_error"));
                    }
                    // on_started fetches once it is up.
                }
            }
        });
    }

    fn fetch(self: &Rc<Self>) {
        let Some(client) = self.client.borrow().clone() else {
            return;
        };
        let weak = Rc::downgrade(self);
        client.fetch(move |res| {
            if let (Some(ui), Ok(deadlines)) = (weak.upgrade(), res) {
                ui.set_deadlines(deadlines);
            }
        });
    }

    fn set_deadlines(&self, deadlines: Deadlines) {
        self.deadlines.replace(deadlines);
        self.refresh();
    }

    /// Updates the countdown once a second, skipped while GNOME reports
    /// the window as not visible.
    fn start_ticking(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_secs(1), move || {
            let Some(ui) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if !ui.window.is_suspended() {
                ui.refresh();
            }
            glib::ControlFlow::Continue
        });
    }

    /// Saves the config and tells the service to pick it up.
    pub fn commit(&self) {
        if let Some(timer) = self.save_timer.take() {
            timer.remove();
        }
        if let Err(err) = self.cfg.borrow().save() {
            eprintln!("beacon: could not save the config: {err}");
            self.toast(&err.to_string());
        }
        if let Some(client) = self.client.borrow().as_ref() {
            client.reload();
        }
        self.refresh();
    }

    /// Commits once the slider has been still for a moment.
    fn commit_soon(self: &Rc<Self>) {
        if let Some(timer) = self.save_timer.take() {
            timer.remove();
        }
        let weak = Rc::downgrade(self);
        let timer = glib::timeout_add_local_once(SAVE_DELAY, move || {
            if let Some(ui) = weak.upgrade() {
                // The source is finishing on its own: forget it, don't remove it.
                ui.save_timer.replace(None);
                ui.commit();
            }
        });
        self.save_timer.replace(Some(timer));
    }

    /// Saves now if a change is still waiting.
    fn flush(&self) {
        if self.save_timer.borrow().is_some() {
            self.commit();
        }
    }

    pub fn toast(&self, text: &str) {
        self.toasts
            .add_toast(adw::Toast::builder().title(text).use_markup(false).build());
    }

    /// Recreates the lines after reminders were added, removed or renamed.
    pub fn rebuild(self: &Rc<Self>) {
        for row in self.rows.take() {
            self.grid.remove(&row.check);
            self.grid.remove(&row.name);
            self.grid.remove(&row.scale);
            self.grid.remove(&row.value);
        }
        let reminders = self.cfg.borrow().reminders.clone();
        let rows: Vec<Row> = reminders
            .iter()
            .enumerate()
            .map(|(i, r)| self.make_row(r, i as i32 + 1))
            .collect();
        let empty = rows.is_empty();
        self.rows.replace(rows);
        self.heading.set_visible(!empty);
        self.grid.set_visible(!empty);
        self.empty.set_visible(empty);
        self.refresh();
    }

    fn make_row(self: &Rc<Self>, r: &Reminder, line: i32) -> Row {
        let check = gtk::CheckButton::builder()
            .active(r.enabled)
            .valign(gtk::Align::Center)
            .tooltip_text(tr("toggle_reminder"))
            .build();
        check.update_property(&[gtk::accessible::Property::Label(&r.display_name())]);

        // The name is a flat button: click it to rename or delete.
        let name = gtk::Button::builder()
            .child(
                &gtk::Label::builder()
                    .label(r.display_name())
                    .xalign(0.0)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .build(),
            )
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .tooltip_text(tr("edit_reminder"))
            .css_classes(["flat", "reminder-name"])
            .build();

        let scale = gtk::Scale::with_range(
            gtk::Orientation::Horizontal,
            f64::from(MIN_MINUTES),
            f64::from(MAX_MINUTES),
            1.0,
        );
        scale.set_value(f64::from(r.minutes));
        scale.set_round_digits(0);
        scale.set_draw_value(false);
        scale.set_hexpand(true);
        scale.set_valign(gtk::Align::Center);
        scale.set_margin_start(12);
        scale.update_property(&[gtk::accessible::Property::Label(&r.display_name())]);

        let value = gtk::Label::builder()
            .label(minutes_label(r.minutes))
            .xalign(1.0)
            .width_chars(6)
            .css_classes(["numeric", "dim-label"])
            .build();

        self.grid.attach(&check, 0, line, 1, 1);
        self.grid.attach(&name, 1, line, 1, 1);
        self.grid.attach(&scale, 2, line, 1, 1);
        self.grid.attach(&value, 3, line, 1, 1);

        let id = r.id;
        let weak = Rc::downgrade(self);
        check.connect_toggled(move |check| {
            let Some(ui) = weak.upgrade() else { return };
            if ui.syncing.get() {
                return;
            }
            if let Some(r) = ui.cfg.borrow_mut().get_mut(id) {
                r.enabled = check.is_active();
            }
            ui.commit();
        });
        let weak = Rc::downgrade(self);
        name.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                editor::open(&ui, Some(id));
            }
        });
        let weak = Rc::downgrade(self);
        let label = value.clone();
        scale.connect_value_changed(move |scale| {
            let Some(ui) = weak.upgrade() else { return };
            let minutes = scale.value().round() as u32;
            label.set_label(&minutes_label(minutes));
            if ui.syncing.get() {
                return;
            }
            let changed = match ui.cfg.borrow_mut().get_mut(id) {
                Some(r) if r.minutes != minutes => {
                    r.minutes = minutes;
                    true
                }
                _ => false,
            };
            if changed {
                ui.commit_soon();
            }
        });

        Row {
            id,
            check,
            name,
            scale,
            value,
        }
    }

    /// Brings the widgets in line with the model and the clocks.
    pub fn refresh(&self) {
        let now = SystemTime::now();
        let cfg = self.cfg.borrow();
        let deadlines = self.deadlines.borrow();
        let left = |id: u32| {
            deadlines
                .get(&id)
                .map(|d| d.duration_since(now).unwrap_or_default())
        };

        self.syncing.set(true);
        self.master.set_active(!cfg.paused);
        self.grid.set_sensitive(!cfg.paused);

        let subtitle = if cfg.paused {
            tr("paused")
        } else {
            cfg.reminders
                .iter()
                .filter(|r| r.enabled)
                .filter_map(|r| left(r.id).map(|d| (d, r)))
                .min_by_key(|(d, _)| *d)
                .map(|(d, r)| trf("next", &[("name", &r.display_name()), ("time", &clock(d))]))
                .unwrap_or_else(|| tr("nothing_scheduled"))
        };
        self.title.set_subtitle(&subtitle);

        for row in self.rows.borrow().iter() {
            let Some(r) = cfg.get(row.id) else { continue };
            if row.check.is_active() != r.enabled {
                row.check.set_active(r.enabled);
            }
            row.scale.set_sensitive(r.enabled);
            row.value.set_sensitive(r.enabled);
            if row.scale.value().round() as u32 != r.minutes {
                row.scale.set_value(f64::from(r.minutes));
            }
            // Time left, on hover.
            let tip = match left(r.id) {
                Some(d) if r.enabled && !cfg.paused => {
                    trf("next", &[("name", &r.display_name()), ("time", &clock(d))])
                }
                _ => String::new(),
            };
            row.scale
                .set_tooltip_text((!tip.is_empty()).then_some(tip.as_str()));
        }
        self.syncing.set(false);
    }

    /// Removes a reminder, with an Undo button in a toast.
    pub fn delete(self: &Rc<Self>, id: u32) {
        let removed = {
            let mut cfg = self.cfg.borrow_mut();
            cfg.reminders
                .iter()
                .position(|r| r.id == id)
                .map(|i| (i, cfg.reminders.remove(i)))
        };
        let Some((index, reminder)) = removed else {
            return;
        };
        self.rebuild();
        self.commit();

        let toast = adw::Toast::builder()
            .title(trf("deleted", &[("name", &reminder.display_name())]))
            .use_markup(false)
            .button_label(tr("undo"))
            .priority(adw::ToastPriority::High)
            .build();
        let weak = Rc::downgrade(self);
        toast.connect_button_clicked(move |_| {
            let Some(ui) = weak.upgrade() else { return };
            {
                let mut cfg = ui.cfg.borrow_mut();
                if cfg.get(reminder.id).is_none() {
                    let at = index.min(cfg.reminders.len());
                    cfg.reminders.insert(at, reminder.clone());
                }
            }
            ui.rebuild();
            ui.commit();
        });
        self.toasts.add_toast(toast);
    }
}
