//! The dialog that adds or edits a reminder.

use std::rc::Rc;

use adw::prelude::*;
use beacon::config::{MAX_MINUTES, MIN_MINUTES, Reminder};
use beacon::i18n::tr;

use crate::window::Ui;

pub fn open(ui: &Rc<Ui>, id: Option<u32>) {
    let existing = id.and_then(|id| ui.cfg.borrow().get(id).cloned());
    let is_new = existing.is_none();

    let dialog = adw::Dialog::builder()
        .title(tr(if is_new {
            "new_reminder"
        } else {
            "edit_reminder"
        }))
        .content_width(420)
        .build();

    let cancel = gtk::Button::with_label(&tr("cancel"));
    let save = gtk::Button::builder()
        .label(tr(if is_new { "add" } else { "save" }))
        .css_classes(["suggested-action"])
        .build();
    let header = adw::HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();
    header.pack_start(&cancel);
    header.pack_end(&save);

    let name = adw::EntryRow::builder()
        .title(tr("name"))
        .text(
            existing
                .as_ref()
                .map(Reminder::display_name)
                .unwrap_or_default(),
        )
        .build();
    let minutes = adw::SpinRow::with_range(f64::from(MIN_MINUTES), f64::from(MAX_MINUTES), 1.0);
    minutes.set_title(&tr("interval"));
    minutes.set_subtitle(&tr("interval_subtitle"));
    minutes.set_value(f64::from(existing.as_ref().map_or(30, |r| r.minutes)));

    let group = adw::PreferencesGroup::new();
    group.add(&name);
    group.add(&minutes);
    let page = adw::PreferencesPage::new();
    page.add(&group);

    if let Some(reminder) = &existing {
        let delete = adw::ButtonRow::builder()
            .title(tr("delete_reminder"))
            .build();
        // add_css_class, not the builder: that would drop the row's own classes.
        delete.add_css_class("destructive-action");
        let group = adw::PreferencesGroup::new();
        group.add(&delete);
        page.add(&group);
        let (ui, dialog, id) = (ui.clone(), dialog.clone(), reminder.id);
        delete.connect_activated(move |_| {
            dialog.close();
            ui.delete(id);
        });
    }

    let view = adw::ToolbarView::new();
    view.add_top_bar(&header);
    view.set_content(Some(&page));
    dialog.set_child(Some(&view));
    dialog.set_default_widget(Some(&save));

    let valid = |text: &str| !text.trim().is_empty();
    save.set_sensitive(valid(&name.text()));
    let s = save.clone();
    name.connect_changed(move |entry| s.set_sensitive(valid(&entry.text())));

    let d = dialog.clone();
    cancel.connect_clicked(move |_| {
        d.close();
    });

    let apply = {
        let (ui, dialog, name, minutes) =
            (ui.clone(), dialog.clone(), name.clone(), minutes.clone());
        move || {
            let text = name.text();
            if !valid(&text) {
                return;
            }
            // Commit a value typed into the spin button but not yet applied.
            minutes.update();
            let mins = minutes.value().round() as u32;
            {
                let mut cfg = ui.cfg.borrow_mut();
                match id.and_then(|id| cfg.get_mut(id)) {
                    Some(r) => {
                        r.rename(&text);
                        r.minutes = mins;
                    }
                    None => {
                        let id = cfg.next_id();
                        cfg.reminders.push(Reminder::custom(id, text.trim(), mins));
                    }
                }
            }
            dialog.close();
            ui.rebuild();
            ui.commit();
        }
    };
    let apply = Rc::new(apply);
    let a = apply.clone();
    save.connect_clicked(move |_| a());
    name.connect_entry_activated(move |_| apply());

    dialog.present(Some(&ui.window));
    name.grab_focus();
}
