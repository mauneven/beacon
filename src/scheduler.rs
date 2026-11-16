//! The whole job of Beacon: each enabled reminder has a clock. When it
//! reaches `minutes`, a notification goes out and the clock starts over.
//!
//! Changing a reminder's minutes, turning it on, or resuming starts that
//! clock from zero. After the machine sleeps the clocks restart quietly
//! instead of firing a backlog of notifications.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::{Config, Reminder};

/// A deadline missed by more than this was slept through: restart quietly.
const LATE: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct Scheduler {
    deadlines: HashMap<u32, SystemTime>,
}

fn interval(r: &Reminder) -> Duration {
    Duration::from_secs(u64::from(r.minutes) * 60)
}

impl Scheduler {
    /// Starts a reminder's clock from zero, or stops it when it's off.
    pub fn restart(&mut self, cfg: &Config, r: &Reminder, now: SystemTime) {
        if !cfg.paused && r.enabled {
            self.deadlines.insert(r.id, now + interval(r));
        } else {
            self.deadlines.remove(&r.id);
        }
    }

    pub fn restart_all(&mut self, cfg: &Config, now: SystemTime) {
        self.deadlines.clear();
        for r in &cfg.reminders {
            self.restart(cfg, r, now);
        }
    }

    /// Brings the clocks in line with an edited config: a new reminder, new
    /// minutes, or a reminder turned on starts from zero; resuming restarts
    /// everything; untouched reminders keep counting.
    pub fn apply(&mut self, old: &Config, new: &Config, now: SystemTime) {
        if old.paused != new.paused {
            self.restart_all(new, now);
            return;
        }
        self.deadlines
            .retain(|id, _| new.reminders.iter().any(|r| r.id == *id));
        for r in &new.reminders {
            let changed = old
                .get(r.id)
                .is_none_or(|o| o.minutes != r.minutes || o.enabled != r.enabled);
            if changed || (!self.deadlines.contains_key(&r.id) && r.enabled) {
                self.restart(new, r, now);
            }
        }
    }

    /// Pushes a running reminder's next notification `by` from now.
    pub fn snooze(&mut self, id: u32, by: Duration, now: SystemTime) -> bool {
        match self.deadlines.get_mut(&id) {
            Some(deadline) => {
                *deadline = now + by;
                true
            }
            None => false,
        }
    }

    /// Returns the reminders due now and restarts their clocks. Deadlines
    /// missed by a lot (the machine slept) restart without firing.
    pub fn tick(&mut self, cfg: &Config, now: SystemTime) -> Vec<u32> {
        let mut due = Vec::new();
        if cfg.paused {
            return due;
        }
        for r in cfg.reminders.iter().filter(|r| r.enabled) {
            let deadline = self.deadlines.entry(r.id).or_insert(now + interval(r));
            if now >= *deadline {
                if now.duration_since(*deadline).unwrap_or_default() <= LATE {
                    due.push(r.id);
                }
                *deadline = now + interval(r);
            }
        }
        due
    }

    /// When the next clock runs out.
    pub fn next_deadline(&self) -> Option<SystemTime> {
        self.deadlines.values().min().copied()
    }

    /// Every clock as (id, Unix milliseconds), for the window.
    pub fn export(&self) -> Vec<(u32, i64)> {
        let mut v: Vec<_> = self
            .deadlines
            .iter()
            .map(|(id, d)| (*id, to_millis(*d)))
            .collect();
        v.sort_unstable();
        v
    }
}

pub fn to_millis(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn from_millis(ms: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(ms.max(0) as u64)
}

/// `12:04`, or `1:02:03` past an hour.
pub fn clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config {
            paused: false,
            snooze_minutes: 5,
            reminders: vec![
                Reminder::preset(1, "water", 20, true),
                Reminder::preset(2, "eyes", 10, false),
            ],
        }
    }

    fn start() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_000_000)
    }

    /// Fires the timer at each deadline, like the service does, and
    /// collects (minute, id).
    fn run(s: &mut Scheduler, cfg: &Config, until: SystemTime) -> Vec<(u64, u32)> {
        let mut fired = Vec::new();
        while let Some(at) = s.next_deadline() {
            if at > until {
                break;
            }
            let minute = at.duration_since(start()).unwrap().as_secs() / 60;
            fired.extend(s.tick(cfg, at).into_iter().map(|id| (minute, id)));
        }
        fired
    }

    #[test]
    fn fires_every_interval_and_restarts() {
        let cfg = cfg();
        let mut s = Scheduler::default();
        s.restart_all(&cfg, start());
        let fired = run(&mut s, &cfg, start() + Duration::from_secs(61 * 60));
        assert_eq!(fired, vec![(20, 1), (40, 1), (60, 1)]);
    }

    #[test]
    fn edits_restart_only_what_changed() {
        let old = cfg();
        let later = start() + Duration::from_secs(300);
        let mut s = Scheduler::default();
        s.restart_all(&old, start());
        let left = |s: &Scheduler, id| {
            s.export()
                .iter()
                .find(|(i, _)| *i == id)
                .map(|(_, ms)| from_millis(*ms).duration_since(later).unwrap().as_secs() / 60)
        };

        let mut new = old.clone();
        new.reminders[1].enabled = true;
        new.reminders.push(Reminder::preset(3, "legs", 40, true));
        s.apply(&old, &new, later);
        assert_eq!(left(&s, 1), Some(15));
        assert_eq!(left(&s, 2), Some(10));
        assert_eq!(left(&s, 3), Some(40));

        let mut newer = new.clone();
        newer.reminders[0].minutes = 30;
        newer.reminders.remove(2);
        s.apply(&new, &newer, later);
        assert_eq!(left(&s, 1), Some(30));
        assert_eq!(left(&s, 3), None);

        let mut paused = newer.clone();
        paused.paused = true;
        s.apply(&newer, &paused, later);
        assert!(s.export().is_empty());
        s.apply(&paused, &newer, later);
        assert_eq!(left(&s, 2), Some(10));
    }

    #[test]
    fn paused_is_silent() {
        let mut cfg = cfg();
        cfg.paused = true;
        let mut s = Scheduler::default();
        s.restart_all(&cfg, start());
        assert_eq!(s.next_deadline(), None);
        assert!(s.tick(&cfg, start() + Duration::from_secs(3600)).is_empty());
    }

    #[test]
    fn a_slept_through_deadline_restarts_quietly() {
        let cfg = cfg();
        let mut s = Scheduler::default();
        s.restart_all(&cfg, start());
        let wake = start() + Duration::from_secs(3 * 3600);
        assert!(s.tick(&cfg, wake).is_empty());
        assert_eq!(s.next_deadline(), Some(wake + Duration::from_secs(20 * 60)));
    }

    #[test]
    fn snooze_moves_only_running_clocks() {
        let cfg = cfg();
        let mut s = Scheduler::default();
        s.restart_all(&cfg, start());
        assert!(s.snooze(1, Duration::from_secs(300), start()));
        assert!(!s.snooze(2, Duration::from_secs(300), start()));
        assert_eq!(s.next_deadline(), Some(start() + Duration::from_secs(300)));
    }

    #[test]
    fn clock_format() {
        assert_eq!(clock(Duration::from_secs(65)), "01:05");
        assert_eq!(clock(Duration::from_secs(3723)), "1:02:03");
    }
}
