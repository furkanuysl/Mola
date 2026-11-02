//! Odak zamanlayıcısı: Pomodoro, 52/17, 90/20 ve Flowtime. Odak bitince verilen mola,
//! hatırlatma motorundaki hareket molası olarak yapılır; böylece oturma sayacı da sıfırlanır.

use serde::Serialize;

use crate::store::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Technique {
    Pomodoro,
    #[serde(rename = "52_17")]
    Fifty2,
    #[serde(rename = "90_20")]
    Ninety,
    Flowtime,
}

impl Technique {
    pub fn parse(id: &str) -> Option<Self> {
        Some(match id {
            "pomodoro" => Self::Pomodoro,
            "52_17" => Self::Fifty2,
            "90_20" => Self::Ninety,
            "flowtime" => Self::Flowtime,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Off,
    Focus,
    Break,
    /// Mola bitti, sonraki tur kullanıcının başlatmasını bekliyor.
    Ready,
}

#[derive(Debug)]
pub struct Focus {
    pub technique: Technique,
    pub phase: Phase,
    pub elapsed: u64,
    pub paused: bool,
    /// Bu oturumda tamamlanan odak turları.
    pub completed: u32,
}

impl Default for Focus {
    fn default() -> Self {
        Self { technique: Technique::Pomodoro, phase: Phase::Off, elapsed: 0, paused: false, completed: 0 }
    }
}

pub struct Tick {
    /// Bu saniye odak süresine sayıldı mı.
    pub counted: bool,
    /// Odak bitti; bu kadar saniyelik mola verilmeli.
    pub break_due: Option<u64>,
}

impl Focus {
    /// Odak oturumunu (ya da hazır bekleyen sonraki turu) başlatır.
    pub fn start(&mut self, technique: Technique) {
        if self.phase == Phase::Off || technique != self.technique {
            self.completed = 0;
        }
        self.technique = technique;
        self.phase = Phase::Focus;
        self.elapsed = 0;
        self.paused = false;
    }

    pub fn stop(&mut self) {
        *self = Self { technique: self.technique, ..Self::default() };
    }

    pub fn is_active(&self) -> bool {
        matches!(self.phase, Phase::Focus | Phase::Break)
    }

    pub fn target_secs(&self, s: &Settings) -> Option<u64> {
        match self.technique {
            Technique::Pomodoro => Some(s.pomodoro_work_min as u64 * 60),
            Technique::Fifty2 => Some(52 * 60),
            Technique::Ninety => Some(90 * 60),
            Technique::Flowtime => None,
        }
    }

    /// `busy`: görüşme/tam ekran; süre dolsa da mola, kullanıcı müsait olana kadar ertelenir.
    pub fn tick(&mut self, s: &Settings, busy: bool) -> Tick {
        if self.phase != Phase::Focus || self.paused {
            return Tick { counted: false, break_due: None };
        }
        self.elapsed += 1;
        let due = self.target_secs(s).is_some_and(|t| self.elapsed >= t) && !busy;
        Tick { counted: true, break_due: due.then(|| self.finish(s)) }
    }

    /// Odağı bitirip molaya geçer; mola süresini döndürür. Odakta değilse `None`.
    pub fn break_now(&mut self, s: &Settings) -> Option<u64> {
        (self.phase == Phase::Focus).then(|| self.finish(s))
    }

    fn finish(&mut self, s: &Settings) -> u64 {
        self.completed += 1;
        self.phase = Phase::Break;
        self.paused = false;
        let mins = match self.technique {
            Technique::Pomodoro if self.completed % s.pomodoro_rounds.max(1) == 0 => s.pomodoro_long_min as u64,
            Technique::Pomodoro => s.pomodoro_short_min as u64,
            Technique::Fifty2 => 17,
            Technique::Ninety => 20,
            Technique::Flowtime => flowtime_break_min(self.elapsed),
        };
        mins.max(1) * 60
    }

    /// Hatırlatma motorundaki mola bittiğinde çağrılır.
    pub fn break_finished(&mut self, s: &Settings) {
        if self.phase != Phase::Break {
            return;
        }
        self.elapsed = 0;
        self.phase = if s.focus_auto_continue { Phase::Focus } else { Phase::Ready };
    }
}

/// Flowtime: mola, ne kadar odaklanıldığına göre uzar.
fn flowtime_break_min(elapsed: u64) -> u64 {
    match elapsed / 60 {
        0..25 => 5,
        25..50 => 8,
        50..90 => 10,
        _ => 15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings { pomodoro_work_min: 1, pomodoro_short_min: 1, pomodoro_long_min: 3, pomodoro_rounds: 2, ..Settings::default() }
    }

    fn run(f: &mut Focus, s: &Settings, secs: u64, busy: bool) -> Option<u64> {
        (0..secs).find_map(|_| f.tick(s, busy).break_due)
    }

    #[test]
    fn pomodoro_alternates_short_and_long_breaks() {
        let s = settings();
        let mut f = Focus::default();
        f.start(Technique::Pomodoro);
        assert_eq!(run(&mut f, &s, 60, false), Some(60));
        f.break_finished(&s);
        assert_eq!(f.phase, Phase::Ready);
        f.start(Technique::Pomodoro);
        assert_eq!(run(&mut f, &s, 60, false), Some(180));
    }

    #[test]
    fn break_waits_while_busy() {
        let s = settings();
        let mut f = Focus::default();
        f.start(Technique::Pomodoro);
        assert_eq!(run(&mut f, &s, 120, true), None);
        assert!(f.tick(&s, false).break_due.is_some());
    }

    #[test]
    fn paused_focus_does_not_count() {
        let s = settings();
        let mut f = Focus::default();
        f.start(Technique::Pomodoro);
        f.paused = true;
        assert_eq!(run(&mut f, &s, 120, false), None);
        assert_eq!(f.elapsed, 0);
    }

    #[test]
    fn flowtime_break_scales_with_focus() {
        let s = settings();
        let mut f = Focus::default();
        f.start(Technique::Flowtime);
        assert_eq!(run(&mut f, &s, 60 * 60, false), None);
        assert_eq!(f.break_now(&s), Some(10 * 60));
    }

    #[test]
    fn auto_continue_starts_next_round() {
        let s = Settings { focus_auto_continue: true, ..settings() };
        let mut f = Focus::default();
        f.start(Technique::Pomodoro);
        run(&mut f, &s, 60, false);
        f.break_finished(&s);
        assert_eq!(f.phase, Phase::Focus);
        assert_eq!(f.completed, 1);
    }
}
