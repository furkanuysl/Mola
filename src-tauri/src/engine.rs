//! Hatırlatma zamanlayıcılarının saf mantığı. `tick` saniyede bir çağrılır;
//! pencere açma/kapama gibi yan etkiler `Effects` olarak geri döner.

use serde::Serialize;

use crate::store::Settings;

/// "Mola bitti" ekranı bu kadar saniye sonra kendiliğinden kapanır.
const DONE_VISIBLE_SECS: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Water,
    Move,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Prompt,
    Break { remaining: u64 },
    Done { shown_for: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Active {
    pub kind: Kind,
    pub phase: Phase,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub show: bool,
    pub hide: bool,
    pub changed: bool,
    pub break_done: bool,
    pub stats: bool,
}

#[derive(Debug, Default)]
pub struct Engine {
    pub water_elapsed: u64,
    pub sitting: u64,
    pub water_snoozes: u32,
    pub move_snoozes: u32,
    pub active: Option<Active>,
    break_requested: bool,
}

impl Engine {
    pub fn tick(&mut self, s: &Settings, idle_secs: u64, running: bool) -> Effects {
        let mut fx = Effects::default();

        // Mola geri sayımı duraklatma ve mesai saatinden bağımsız işler.
        if let Some(active) = self.active {
            match active.phase {
                Phase::Break { remaining } => {
                    fx.changed = true;
                    if remaining > 1 {
                        self.set_phase(Phase::Break { remaining: remaining - 1 });
                    } else {
                        self.complete_break(&mut fx);
                        self.set_phase(Phase::Done { shown_for: 0 });
                    }
                    return fx;
                }
                Phase::Done { shown_for } => {
                    if shown_for + 1 >= DONE_VISIBLE_SECS {
                        self.close(&mut fx);
                    } else {
                        self.set_phase(Phase::Done { shown_for: shown_for + 1 });
                    }
                    return fx;
                }
                Phase::Prompt => {}
            }
        }

        if self.break_requested {
            self.break_requested = false;
            self.active = Some(Active {
                kind: Kind::Move,
                phase: Phase::Break { remaining: break_secs(s) },
            });
            fx.show = true;
            fx.changed = true;
            return fx;
        }

        // Klavye/fare uzun süre boşta kaldıysa kullanıcı kalkmış demektir.
        let away = idle_secs >= mins(s.idle_reset_min);
        if away {
            if self.is_active(Kind::Move) {
                self.close(&mut fx);
                self.complete_break(&mut fx);
            }
            self.sitting = 0;
            self.move_snoozes = 0;
        }

        if !running {
            return fx;
        }

        if s.water_enabled && !self.is_active(Kind::Water) {
            self.water_elapsed += 1;
        }
        if s.move_enabled && !away && !self.is_active(Kind::Move) {
            self.sitting += 1;
        }

        if self.active.is_none() {
            if s.move_enabled && self.sitting >= mins(s.move_interval_min) {
                self.open(Kind::Move, &mut fx);
            } else if s.water_enabled && self.water_elapsed >= mins(s.water_interval_min) {
                self.open(Kind::Water, &mut fx);
            }
        }
        fx
    }

    /// Bir sonraki tick'te mola geri sayımını başlatır (zaten moladaysa yok sayılır).
    pub fn request_break(&mut self) {
        let in_break = self
            .active
            .is_some_and(|a| matches!(a.phase, Phase::Break { .. } | Phase::Done { .. }));
        if !in_break {
            self.break_requested = true;
        }
    }

    pub fn drink(&mut self) -> Effects {
        let mut fx = Effects { stats: true, ..Default::default() };
        self.water_elapsed = 0;
        self.water_snoozes = 0;
        if self.is_active(Kind::Water) {
            self.close(&mut fx);
        }
        fx
    }

    pub fn skip_water(&mut self) -> Result<Effects, String> {
        if self.prompt_kind() != Some(Kind::Water) {
            return Err("nothing_to_skip".into());
        }
        let mut fx = Effects::default();
        self.water_elapsed = 0;
        self.water_snoozes = 0;
        self.close(&mut fx);
        Ok(fx)
    }

    pub fn snooze(&mut self, s: &Settings) -> Result<Effects, String> {
        let kind = self.prompt_kind().ok_or("nothing_to_snooze")?;
        if self.snoozes_left(kind, s) == 0 {
            return Err("no_snoozes_left".into());
        }
        // Sayaç, erteleme süresi sonra tekrar dolacak şekilde geri sarılır.
        let snooze = mins(s.snooze_min);
        match kind {
            Kind::Water => {
                self.water_snoozes += 1;
                self.water_elapsed = mins(s.water_interval_min).saturating_sub(snooze);
            }
            Kind::Move => {
                self.move_snoozes += 1;
                self.sitting = mins(s.move_interval_min).saturating_sub(snooze);
            }
        }
        let mut fx = Effects { stats: true, ..Default::default() };
        self.close(&mut fx);
        Ok(fx)
    }

    pub fn start_break(&mut self, s: &Settings) -> Result<Effects, String> {
        if self.prompt_kind() != Some(Kind::Move) {
            return Err("nothing_to_start".into());
        }
        self.set_phase(Phase::Break { remaining: break_secs(s) });
        Ok(Effects { changed: true, ..Default::default() })
    }

    pub fn dismiss(&mut self) -> Effects {
        let mut fx = Effects::default();
        if self.active.is_some_and(|a| matches!(a.phase, Phase::Done { .. })) {
            self.close(&mut fx);
        }
        fx
    }

    pub fn snoozes_left(&self, kind: Kind, s: &Settings) -> u32 {
        let used = match kind {
            Kind::Water => self.water_snoozes,
            Kind::Move => self.move_snoozes,
        };
        s.max_snoozes.saturating_sub(used)
    }

    fn prompt_kind(&self) -> Option<Kind> {
        match self.active {
            Some(Active { kind, phase: Phase::Prompt }) => Some(kind),
            _ => None,
        }
    }

    fn is_active(&self, kind: Kind) -> bool {
        self.active.is_some_and(|a| a.kind == kind)
    }

    fn open(&mut self, kind: Kind, fx: &mut Effects) {
        self.active = Some(Active { kind, phase: Phase::Prompt });
        fx.show = true;
        fx.changed = true;
    }

    fn close(&mut self, fx: &mut Effects) {
        self.active = None;
        fx.hide = true;
        fx.changed = true;
    }

    fn set_phase(&mut self, phase: Phase) {
        if let Some(active) = &mut self.active {
            active.phase = phase;
        }
    }

    fn complete_break(&mut self, fx: &mut Effects) {
        self.sitting = 0;
        self.move_snoozes = 0;
        fx.break_done = true;
    }
}

fn mins(m: u32) -> u64 {
    m as u64 * 60
}

fn break_secs(s: &Settings) -> u64 {
    mins(s.break_min).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings {
            water_interval_min: 2,
            move_interval_min: 3,
            break_min: 1,
            idle_reset_min: 1,
            snooze_min: 1,
            max_snoozes: 2,
            ..Settings::default()
        }
    }

    fn run(e: &mut Engine, s: &Settings, secs: u64, idle: u64) {
        for _ in 0..secs {
            e.tick(s, idle, true);
        }
    }

    #[test]
    fn water_prompt_after_interval() {
        let s = Settings { move_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 119, 0);
        assert!(e.active.is_none());
        assert!(e.tick(&s, 0, true).show);
        assert_eq!(e.active.unwrap().kind, Kind::Water);
    }

    #[test]
    fn snooze_limit_is_enforced() {
        let s = Settings { move_enabled: false, ..settings() };
        let mut e = Engine::default();
        for _ in 0..2 {
            run(&mut e, &s, 120, 0);
            assert!(e.snooze(&s).is_ok());
        }
        run(&mut e, &s, 120, 0);
        assert_eq!(e.snoozes_left(Kind::Water, &s), 0);
        assert!(e.snooze(&s).is_err());
        e.drink();
        assert_eq!(e.snoozes_left(Kind::Water, &s), 2);
    }

    #[test]
    fn snooze_reopens_after_snooze_minutes() {
        let s = Settings { move_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 120, 0);
        e.snooze(&s).unwrap();
        run(&mut e, &s, 59, 0);
        assert!(e.active.is_none());
        run(&mut e, &s, 1, 0);
        assert!(e.active.is_some());
    }

    #[test]
    fn getting_up_resets_sitting_and_resolves_move_prompt() {
        let s = Settings { water_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 180, 0);
        assert_eq!(e.active.unwrap().kind, Kind::Move);
        let fx = e.tick(&s, 60, true);
        assert!(fx.break_done && fx.hide);
        assert_eq!(e.sitting, 0);
    }

    #[test]
    fn break_counts_down_then_closes() {
        let s = Settings { water_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 180, 0);
        e.start_break(&s).unwrap();
        let done = (0..60).any(|_| e.tick(&s, 0, true).break_done);
        assert!(done);
        assert!(matches!(e.active.unwrap().phase, Phase::Done { .. }));
        run(&mut e, &s, DONE_VISIBLE_SECS, 0);
        assert!(e.active.is_none());
    }

    #[test]
    fn nothing_counts_while_not_running() {
        let s = settings();
        let mut e = Engine::default();
        for _ in 0..500 {
            e.tick(&s, 0, false);
        }
        assert!(e.active.is_none());
        assert_eq!(e.water_elapsed, 0);
    }
}
