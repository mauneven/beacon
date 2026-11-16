//! `beacon`: the window. It exists only while open; the reminders
//! themselves run in `beacon-daemon`.

mod client;
mod dialogs;
mod editor;
mod window;

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use beacon::APP_ID;
use gtk::{gio, glib};

fn main() -> glib::ExitCode {
    gio::resources_register_include!("beacon.gresource").expect("bundled resources");
    glib::set_application_name("Beacon");

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .resource_base_path("/io/github/mauneven/Beacon")
        .build();

    app.set_accels_for_action("app.quit", &["<Control>q"]);
    app.set_accels_for_action("app.preferences", &["<Control>comma"]);
    app.set_accels_for_action("app.shortcuts", &["<Control>question"]);
    app.set_accels_for_action("win.add", &["<Control>n"]);
    app.set_accels_for_action("win.toggle-pause", &["<Control>p"]);
    app.set_accels_for_action("window.close", &["<Control>w"]);

    // The window's state lives as long as the app; callbacks hold weak refs.
    let ui: RefCell<Option<Rc<window::Ui>>> = RefCell::default();
    app.connect_activate(move |app| {
        let existing = ui.borrow().clone();
        match existing {
            Some(ui) => ui.window.present(),
            None => {
                let new = window::Ui::build(app);
                new.window.present();
                ui.replace(Some(new));
            }
        }
    });
    app.run()
}
