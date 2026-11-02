//! Hatırlatma zamanlayıcılarının saf mantığı. `tick` saniyede bir çağrılır;
//! pencere açma/kapama gibi yan etkiler `Effects` olarak geri döner.

use serde::Serialize;

use crate::store::Settings;

/// "Mola bitti" ekranı bu kadar saniye sonra kendiliğinden kapanır.
const DONE_VISIBLE_SECS: u64 = 4;
/// 20-20-20 kuralı: 20 saniye uzağa bakılır.
pub const EYES_SECS: u64 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Water,
    Move,
    Eyes,
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
    pub eyes_elapsed: u64,
    /// Süren molanın toplam uzunluğu (saniye).
    pub break_total: u64,
    /// Odak oturumu sürerken ayrı hareket hatırlatması açılmaz; molayı odak zamanlayıcısı verir.
    pub focus_active: bool,
    pub water_snoozes: u32,
    pub move_snoozes: u32,
    pub active: Option<Active>,
    /// İstenen molanın süresi; bir sonraki tick'te başlar.
    break_requested: Option<u64>,
    /// Kullanıcı meşgulken açık hatırlatmanın penceresi gizlenir.
    hidden: bool,
}

impl Engine {
    /// `busy`: tam ekran uygulama ya da görüşme var; yeni hatırlatma açılmaz.
    pub fn tick(&mut self, s: &Settings, idle_secs: u64, running: bool, busy: bool) -> Effects {
        let mut fx = Effects::default();

        // Mola geri sayımı duraklatma ve mesai saatinden bağımsız işler.
        if let Some(active) = self.active {
            match active.phase {
                Phase::Break { remaining } => {
                    fx.changed = true;
                    if remaining > 1 {
                        self.set_phase(Phase::Break { remaining: remaining - 1 });
                    } else if active.kind == Kind::Eyes {
                        self.eyes_elapsed = 0;
                        self.close(&mut fx);
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

        if let Some(secs) = self.break_requested.take() {
            self.begin(Kind::Move, Phase::Break { remaining: secs }, secs, &mut fx);
            return fx;
        }

        // Meşgulken açık hatırlatma gizlenir (ekran paylaşımında görünmesin), bitince geri gelir.
        if busy && self.active.is_some() && !self.hidden {
            self.hidden = true;
            fx.hide = true;
        } else if !busy && self.hidden {
            self.hidden = false;
            fx.show = true;
            fx.changed = true;
        }

        // Klavye/fare uzun süre boşta kaldıysa kullanıcı kalkmış demektir. Toplantıda dinlerken
        // ya da film izlerken girdi olmaz ama kişi hâlâ oturuyordur; o yüzden meşgulken sayılmaz.
        let away = !busy && idle_secs >= mins(s.idle_reset_min);
        if away {
            if self.is_active(Kind::Move) {
                self.close(&mut fx);
                self.complete_break(&mut fx);
            }
            self.sitting = 0;
            self.move_snoozes = 0;
            self.eyes_elapsed = 0;
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
        if s.eyes_enabled && !away && !self.is_active(Kind::Eyes) {
            self.eyes_elapsed += 1;
        }

        if self.active.is_none() && !busy {
            if s.move_enabled && !self.focus_active && self.sitting >= mins(s.move_interval_min) {
                self.open(Kind::Move, &mut fx);
            } else if s.water_enabled && self.water_elapsed >= mins(s.water_interval_min) {
                self.open(Kind::Water, &mut fx);
            } else if s.eyes_enabled && self.eyes_elapsed >= mins(s.eyes_interval_min) {
                self.begin(Kind::Eyes, Phase::Break { remaining: EYES_SECS }, EYES_SECS, &mut fx);
            }
        }
        fx
    }

    /// Bir sonraki tick'te `secs` saniyelik mola başlatır (zaten hareket molasındaysa yok sayılır).
    pub fn request_break(&mut self, secs: u64) {
        let in_break = self.active.is_some_and(|a| {
            a.kind == Kind::Move && matches!(a.phase, Phase::Break { .. } | Phase::Done { .. })
        });
        if !in_break {
            self.break_requested = Some(secs.max(1));
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

    /// Su hatırlatmasını ya da göz molasını atlar.
    pub fn skip(&mut self) -> Result<Effects, String> {
        let mut fx = Effects::default();
        match self.active {
            Some(Active { kind: Kind::Water, phase: Phase::Prompt }) => {
                self.water_elapsed = 0;
                self.water_snoozes = 0;
            }
            Some(Active { kind: Kind::Eyes, .. }) => self.eyes_elapsed = 0,
            _ => return Err("nothing_to_skip".into()),
        }
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
            Kind::Eyes => return Err("nothing_to_snooze".into()),
        }
        let mut fx = Effects { stats: true, ..Default::default() };
        self.close(&mut fx);
        Ok(fx)
    }

    pub fn start_break(&mut self, s: &Settings) -> Result<Effects, String> {
        if self.prompt_kind() != Some(Kind::Move) {
            return Err("nothing_to_start".into());
        }
        self.break_total = break_secs(s);
        self.set_phase(Phase::Break { remaining: self.break_total });
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
            Kind::Eyes => return 0,
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
        self.begin(kind, Phase::Prompt, 0, fx);
    }

    fn begin(&mut self, kind: Kind, phase: Phase, break_total: u64, fx: &mut Effects) {
        self.active = Some(Active { kind, phase });
        if break_total > 0 {
            self.break_total = break_total;
        }
        self.hidden = false;
        fx.show = true;
        fx.changed = true;
    }

    fn close(&mut self, fx: &mut Effects) {
        self.active = None;
        self.hidden = false;
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
        self.eyes_elapsed = 0;
        fx.break_done = true;
    }
}

fn mins(m: u32) -> u64 {
    m as u64 * 60
}

pub fn break_secs(s: &Settings) -> u64 {
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
            eyes_enabled: false,
            ..Settings::default()
        }
    }

    fn run(e: &mut Engine, s: &Settings, secs: u64, idle: u64) {
        for _ in 0..secs {
            e.tick(s, idle, true, false);
        }
    }

    #[test]
    fn water_prompt_after_interval() {
        let s = Settings { move_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 119, 0);
        assert!(e.active.is_none());
        assert!(e.tick(&s, 0, true, false).show);
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
        let fx = e.tick(&s, 60, true, false);
        assert!(fx.break_done && fx.hide);
        assert_eq!(e.sitting, 0);
    }

    #[test]
    fn break_counts_down_then_closes() {
        let s = Settings { water_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 180, 0);
        e.start_break(&s).unwrap();
        let done = (0..60).any(|_| e.tick(&s, 0, true, false).break_done);
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
            e.tick(&s, 0, false, false);
        }
        assert!(e.active.is_none());
        assert_eq!(e.water_elapsed, 0);
    }

    #[test]
    fn busy_defers_new_reminders_until_free() {
        let s = Settings { move_enabled: false, ..settings() };
        let mut e = Engine::default();
        for _ in 0..300 {
            e.tick(&s, 0, true, true);
        }
        assert!(e.active.is_none());
        assert!(e.tick(&s, 0, true, false).show);
        assert_eq!(e.active.unwrap().kind, Kind::Water);
    }

    #[test]
    fn open_reminder_hides_while_busy_and_returns_after() {
        let s = Settings { move_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 120, 0);
        assert!(e.tick(&s, 0, true, true).hide);
        assert!(e.active.is_some());
        let fx = e.tick(&s, 0, true, false);
        assert!(fx.show && !fx.hide);
    }

    fn eyes_settings() -> Settings {
        Settings {
            water_enabled: false,
            move_enabled: false,
            eyes_enabled: true,
            eyes_interval_min: 1,
            ..settings()
        }
    }

    #[test]
    fn eyes_break_counts_down_and_closes_without_resetting_sitting() {
        let s = eyes_settings();
        let mut e = Engine::default();
        run(&mut e, &s, 59, 0);
        assert!(e.active.is_none());
        assert!(e.tick(&s, 0, true, false).show);
        assert_eq!(e.active.unwrap().kind, Kind::Eyes);
        run(&mut e, &s, EYES_SECS, 0);
        assert!(e.active.is_none());
        assert_eq!(e.eyes_elapsed, 0);
    }

    #[test]
    fn eyes_break_can_be_skipped() {
        let s = eyes_settings();
        let mut e = Engine::default();
        run(&mut e, &s, 60, 0);
        assert!(e.skip().is_ok());
        assert!(e.active.is_none());
    }

    #[test]
    fn requested_break_uses_given_length_and_resets_sitting() {
        let s = Settings { water_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 100, 0);
        e.request_break(10);
        e.tick(&s, 0, true, false);
        assert_eq!(e.break_total, 10);
        let done = (0..10).any(|_| e.tick(&s, 0, true, false).break_done);
        assert!(done);
        assert_eq!(e.sitting, 0);
    }

    #[test]
    fn focus_session_suppresses_move_prompt() {
        let s = Settings { water_enabled: false, ..settings() };
        let mut e = Engine { focus_active: true, ..Engine::default() };
        run(&mut e, &s, 400, 0);
        assert!(e.active.is_none());
    }

    #[test]
    fn idle_while_busy_does_not_count_as_standing_up() {
        let s = Settings { water_enabled: false, ..settings() };
        let mut e = Engine::default();
        run(&mut e, &s, 100, 0);
        for _ in 0..100 {
            e.tick(&s, 600, true, true);
        }
        assert_eq!(e.sitting, 200);
    }
}
