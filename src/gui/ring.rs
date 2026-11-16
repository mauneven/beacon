//! A small countdown ring in the system accent color: full when the clock
//! starts, empty when the reminder is due.

use std::cell::Cell;
use std::f64::consts::PI;
use std::rc::Rc;

use adw::prelude::*;

#[derive(Clone)]
pub struct Ring {
    area: gtk::DrawingArea,
    state: Rc<Cell<(f64, bool)>>,
}

impl Ring {
    pub fn new() -> Self {
        let area = gtk::DrawingArea::builder()
            .content_width(26)
            .content_height(26)
            .valign(gtk::Align::Center)
            .build();
        let state = Rc::new(Cell::new((0.0_f64, false)));
        let s = state.clone();
        area.set_draw_func(move |area, cr, w, h| {
            let (fraction, active) = s.get();
            let line = 3.0;
            let (cx, cy) = (f64::from(w) / 2.0, f64::from(h) / 2.0);
            let r = f64::from(w.min(h)) / 2.0 - line / 2.0 - 0.5;
            let fg = area.color();
            cr.set_line_width(line);
            cr.set_source_rgba(
                fg.red().into(),
                fg.green().into(),
                fg.blue().into(),
                f64::from(fg.alpha()) * 0.15,
            );
            cr.arc(cx, cy, r, 0.0, 2.0 * PI);
            let _ = cr.stroke();
            if active && fraction > 0.0 {
                let style = adw::StyleManager::default();
                let accent = style.accent_color().to_standalone_rgba(style.is_dark());
                cr.set_source_rgba(
                    accent.red().into(),
                    accent.green().into(),
                    accent.blue().into(),
                    accent.alpha().into(),
                );
                cr.set_line_cap(gtk::cairo::LineCap::Round);
                let start = -PI / 2.0;
                cr.arc(cx, cy, r, start, start + 2.0 * PI * fraction.min(1.0));
                let _ = cr.stroke();
            }
        });
        Ring { area, state }
    }

    pub fn widget(&self) -> &gtk::DrawingArea {
        &self.area
    }

    pub fn set(&self, fraction: f64, active: bool) {
        let new = ((fraction * 200.0).round() / 200.0, active);
        if self.state.replace(new) != new {
            self.area.queue_draw();
        }
    }
}
