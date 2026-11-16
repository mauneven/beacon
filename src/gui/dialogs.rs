//! Preferences, keyboard shortcuts and About.

use std::rc::Rc;

use adw::prelude::*;
use beacon::config::{MAX_SNOOZE, MIN_SNOOZE};
use beacon::i18n::tr;
use beacon::{APP_ID, REPOSITORY, VERSION, autostart};

use crate::window::Ui;

pub fn preferences(ui: &Rc<Ui>) {
    let dialog = adw::PreferencesDialog::new();

    let login = adw::SwitchRow::builder()
        .title(tr("start_at_login"))
        .subtitle(tr("start_at_login_subtitle"))
        .active(autostart::is_enabled())
        .build();
    let d = dialog.clone();
    login.connect_active_notify(move |row| {
        if let Err(err) = autostart::set(row.is_active()) {
            d.add_toast(
                adw::Toast::builder()
                    .title(err.to_string())
                    .use_markup(false)
                    .build(),
            );
        }
    });
    let general = adw::PreferencesGroup::builder()
        .title(tr("general"))
        .build();
    general.add(&login);

    let snooze = adw::SpinRow::with_range(f64::from(MIN_SNOOZE), f64::from(MAX_SNOOZE), 1.0);
    snooze.set_title(&tr("snooze_length"));
    snooze.set_subtitle(&tr("snooze_length_subtitle"));
    snooze.set_value(f64::from(ui.cfg.borrow().snooze_minutes));
    let u = ui.clone();
    snooze.connect_value_notify(move |row| {
        let minutes = row.value().round() as u32;
        if u.cfg.borrow().snooze_minutes != minutes {
            u.cfg.borrow_mut().snooze_minutes = minutes;
            u.commit();
        }
    });
    let notifications = adw::PreferencesGroup::builder()
        .title(tr("notifications"))
        .build();
    notifications.add(&snooze);

    let page = adw::PreferencesPage::new();
    page.add(&general);
    page.add(&notifications);
    dialog.add(&page);
    dialog.present(Some(&ui.window));
}

pub fn shortcuts(ui: &Rc<Ui>) {
    let section = adw::ShortcutsSection::new(None);
    for (title, accel) in [
        (tr("add_reminder"), "<Control>n"),
        (tr("pause"), "<Control>p"),
        (tr("preferences"), "<Control>comma"),
        (tr("shortcuts"), "<Control>question"),
        (tr("close_window"), "<Control>w"),
        (tr("quit_description"), "<Control>q"),
    ] {
        section.add(adw::ShortcutsItem::new(&title, accel));
    }
    let dialog = adw::ShortcutsDialog::new();
    dialog.add(section);
    dialog.present(Some(&ui.window));
}

pub fn about(ui: &Rc<Ui>) {
    let dialog = adw::AboutDialog::builder()
        .application_name("Beacon")
        .application_icon(APP_ID)
        .developer_name("mauneven")
        .developers(["mauneven https://github.com/mauneven"])
        .version(VERSION)
        .comments(tr("comments"))
        .website(REPOSITORY)
        .issue_url(format!("{REPOSITORY}/issues"))
        .license_type(gtk::License::Gpl30)
        .copyright("© 2026 mauneven")
        .build();
    dialog.present(Some(&ui.window));
}
