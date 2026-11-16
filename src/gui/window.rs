//! The main window: a list of reminders with live countdowns.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use adw::prelude::*;
use beacon::config::{Config, Loaded, Reminder};
use beacon::i18n::{tr, trf};
use beacon::scheduler::clock;
use beacon::{APP_ID, autostart};
use gtk::{gio, glib};

use crate::client::{Client, Deadlines};
use crate::ring::Ring;
use crate::{dialogs, editor};

struct Row {
    id: u32,
    row: adw::ActionRow,
    switch: gtk::Switch,
    ring: Ring,
}

pub struct Ui {
    pub app: adw::Application,
    pub window: adw::ApplicationWindow,
    toasts: adw::ToastOverlay,
    title: adw::WindowTitle,
    banner: adw::Banner,
    pause: gtk::ToggleButton,
    stack: gtk::Stack,
    list: gtk::ListBox,
    rows: RefCell<Vec<Row>>,
    pub cfg: RefCell<Config>,
    deadlines: RefCell<Deadlines>,
    client: RefCell<Option<Client>>,
    /// Set while widgets are updated from the model, so their handlers
    /// don't write the change back.
    syncing: Cell<bool>,
}

impl Ui {
    pub fn build(app: &adw::Application) -> Rc<Ui> {
        let cfg = match Config::load() {
            Loaded::FirstRun(cfg) => {
                // Start with the session by default, like the user expects
                // from a reminder app; Preferences turns it off.
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
        let pause = gtk::ToggleButton::builder()
            .icon_name("media-playback-pause-symbolic")
            .tooltip_text(tr("pause"))
            .active(cfg.paused)
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
        header.pack_end(&pause);
        header.set_title_widget(Some(&title));

        let banner = adw::Banner::builder()
            .title(tr("paused_banner"))
            .button_label(tr("resume"))
            .revealed(cfg.paused)
            .build();

        // Reminder list
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .valign(gtk::Align::Start)
            .build();
        let note = gtk::Label::builder()
            .label(tr("background_note"))
            .wrap(true)
            .justify(gtk::Justification::Center)
            .css_classes(["dim-label", "caption"])
            .build();
        let column = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(12)
            .margin_end(12)
            .build();
        column.append(&list);
        column.append(&note);
        let clamp = adw::Clamp::builder()
            .maximum_size(560)
            .child(&column)
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&clamp)
            .build();

        let empty = adw::StatusPage::builder()
            .icon_name(format!("{APP_ID}-symbolic"))
            .title(tr("empty_title"))
            .description(tr("empty_description"))
            .child(
                &gtk::Button::builder()
                    .label(tr("add_reminder"))
                    .halign(gtk::Align::Center)
                    .css_classes(["pill", "suggested-action"])
                    .action_name("win.add")
                    .build(),
            )
            .build();

        let stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::Crossfade)
            .build();
        stack.add_named(&scroller, Some("list"));
        stack.add_named(&empty, Some("empty"));

        let view = adw::ToolbarView::new();
        view.add_top_bar(&header);
        view.add_top_bar(&banner);
        view.set_content(Some(&stack));
        let toasts = adw::ToastOverlay::new();
        toasts.set_child(Some(&view));

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Beacon")
            .default_width(460)
            .default_height(640)
            .width_request(360)
            .height_request(294)
            .content(&toasts)
            .build();

        let ui = Rc::new(Ui {
            app: app.clone(),
            window,
            toasts,
            title,
            banner,
            pause,
            stack,
            list,
            rows: RefCell::new(Vec::new()),
            cfg: RefCell::new(cfg),
            deadlines: RefCell::new(Deadlines::new()),
            client: RefCell::new(None),
            syncing: Cell::new(false),
        });

        ui.connect_signals();
        ui.add_actions();
        ui.rebuild();
        ui.connect_service();
        ui.start_ticking();
        ui
    }

    fn connect_signals(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.pause.connect_toggled(move |button| {
            let Some(ui) = weak.upgrade() else { return };
            if ui.syncing.get() {
                return;
            }
            ui.cfg.borrow_mut().paused = button.is_active();
            ui.commit();
        });
        let pause = self.pause.clone();
        self.banner
            .connect_button_clicked(move |_| pause.set_active(false));
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
        let pause = self.pause.clone();
        toggle.connect_activate(move |_, _| pause.set_active(!pause.is_active()));
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

    /// Updates the countdowns once a second, skipped while GNOME reports
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
        if let Err(err) = self.cfg.borrow().save() {
            eprintln!("beacon: could not save the config: {err}");
            self.toast(&err.to_string());
        }
        if let Some(client) = self.client.borrow().as_ref() {
            client.reload();
        }
        self.refresh();
    }

    pub fn toast(&self, text: &str) {
        self.toasts
            .add_toast(adw::Toast::builder().title(text).use_markup(false).build());
    }

    /// Recreates the rows after reminders were added, removed or renamed.
    pub fn rebuild(self: &Rc<Self>) {
        self.list.remove_all();
        let rows: Vec<Row> = self
            .cfg
            .borrow()
            .reminders
            .iter()
            .map(|r| self.make_row(r))
            .collect();
        for row in &rows {
            self.list.append(&row.row);
        }
        let empty = rows.is_empty();
        self.rows.replace(rows);
        self.stack
            .set_visible_child_name(if empty { "empty" } else { "list" });
        self.refresh();
    }

    fn make_row(self: &Rc<Self>, r: &Reminder) -> Row {
        let row = adw::ActionRow::builder()
            .title(r.display_name())
            .use_markup(false)
            .activatable(true)
            .build();
        row.add_css_class("countdown");
        let ring = Ring::new();
        row.add_prefix(ring.widget());
        let switch = gtk::Switch::builder()
            .active(r.enabled)
            .valign(gtk::Align::Center)
            .tooltip_text(tr("toggle_reminder"))
            .build();
        row.add_suffix(&switch);

        let id = r.id;
        let weak = Rc::downgrade(self);
        row.connect_activated(move |_| {
            if let Some(ui) = weak.upgrade() {
                editor::open(&ui, Some(id));
            }
        });
        let weak = Rc::downgrade(self);
        switch.connect_active_notify(move |switch| {
            let Some(ui) = weak.upgrade() else { return };
            if ui.syncing.get() {
                return;
            }
            if let Some(r) = ui.cfg.borrow_mut().get_mut(id) {
                r.enabled = switch.is_active();
            }
            ui.commit();
        });
        Row {
            id,
            row,
            switch,
            ring,
        }
    }

    /// Brings every label, ring and switch in line with the model.
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
        self.pause.set_active(cfg.paused);
        self.banner.set_revealed(cfg.paused);

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
            let every = trf("every", &[("n", &r.minutes.to_string())]);
            let running = !cfg.paused && r.enabled;
            let state = if cfg.paused {
                tr("paused")
            } else if !r.enabled {
                tr("off")
            } else {
                left(r.id).map(clock).unwrap_or_else(|| "–".into())
            };
            row.row.set_subtitle(&format!("{every} · {state}"));
            if row.switch.is_active() != r.enabled {
                row.switch.set_active(r.enabled);
            }
            let total = f64::from(r.minutes) * 60.0;
            let fraction = left(r.id).map_or(0.0, |d| d.as_secs_f64() / total);
            row.ring.set(fraction, running);
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
