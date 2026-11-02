//! Ayarlar ve günlük istatistikler; uygulama veri klasöründe JSON olarak saklanır.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub water_enabled: bool,
    pub water_interval_min: u32,
    pub water_goal_ml: u32,
    pub glass_ml: u32,
    pub move_enabled: bool,
    pub move_interval_min: u32,
    pub break_min: u32,
    /// Bu kadar dakika klavye/fare kullanılmazsa kullanıcı kalkmış sayılır.
    pub idle_reset_min: u32,
    pub snooze_min: u32,
    pub max_snoozes: u32,
    pub work_hours_enabled: bool,
    pub work_start: String,
    pub work_end: String,
    pub weekdays_only: bool,
    pub autostart: bool,
    /// Tam ekran uygulama (oyun, sunum, video) varken hatırlatmaları beklet.
    pub quiet_in_fullscreen: bool,
    /// Mikrofon kullanımdayken (görüntülü/sesli görüşme) hatırlatmaları beklet.
    pub quiet_in_calls: bool,
    /// "auto", "tr" veya "en".
    pub language: String,
    /// Mola sırasında gösterilen egzersiz rutini (src/exerciseData.ts içindeki kimlikler).
    pub break_routine: String,
    /// 20-20-20 göz molası.
    pub eyes_enabled: bool,
    pub eyes_interval_min: u32,
    /// "pomodoro", "52_17", "90_20" ya da "flowtime".
    pub focus_technique: String,
    pub pomodoro_work_min: u32,
    pub pomodoro_short_min: u32,
    pub pomodoro_long_min: u32,
    /// Kaç turda bir uzun mola verilir.
    pub pomodoro_rounds: u32,
    /// Mola bitince sonraki odak turu kendiliğinden başlasın.
    pub focus_auto_continue: bool,
    /// Günlük skor hedefleri.
    pub breaks_goal: u32,
    pub focus_goal_min: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            water_enabled: true,
            water_interval_min: 45,
            water_goal_ml: 2500,
            glass_ml: 250,
            move_enabled: true,
            move_interval_min: 50,
            break_min: 5,
            idle_reset_min: 5,
            snooze_min: 10,
            max_snoozes: 2,
            work_hours_enabled: false,
            work_start: "09:00".into(),
            work_end: "18:00".into(),
            weekdays_only: true,
            autostart: false,
            quiet_in_fullscreen: true,
            quiet_in_calls: true,
            language: "auto".into(),
            break_routine: "mixed".into(),
            eyes_enabled: true,
            eyes_interval_min: 20,
            focus_technique: "pomodoro".into(),
            pomodoro_work_min: 25,
            pomodoro_short_min: 5,
            pomodoro_long_min: 15,
            pomodoro_rounds: 4,
            focus_auto_continue: false,
            breaks_goal: 6,
            focus_goal_min: 120,
        }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Self {
        self.water_interval_min = self.water_interval_min.clamp(1, 480);
        self.water_goal_ml = self.water_goal_ml.clamp(100, 10_000);
        self.glass_ml = self.glass_ml.clamp(10, 2000);
        self.move_interval_min = self.move_interval_min.clamp(1, 480);
        self.break_min = self.break_min.clamp(1, 60);
        self.idle_reset_min = self.idle_reset_min.clamp(1, 120);
        self.snooze_min = self.snooze_min.clamp(1, 120);
        self.max_snoozes = self.max_snoozes.min(10);
        if !matches!(self.language.as_str(), "auto" | "tr" | "en") {
            self.language = "auto".into();
        }
        if !matches!(self.break_routine.as_str(), "mixed" | "posture" | "neck" | "back" | "quick") {
            self.break_routine = "mixed".into();
        }
        self.eyes_interval_min = self.eyes_interval_min.clamp(5, 120);
        if crate::focus::Technique::parse(&self.focus_technique).is_none() {
            self.focus_technique = "pomodoro".into();
        }
        self.pomodoro_work_min = self.pomodoro_work_min.clamp(1, 180);
        self.pomodoro_short_min = self.pomodoro_short_min.clamp(1, 60);
        self.pomodoro_long_min = self.pomodoro_long_min.clamp(1, 90);
        self.pomodoro_rounds = self.pomodoro_rounds.clamp(1, 12);
        self.breaks_goal = self.breaks_goal.clamp(1, 30);
        self.focus_goal_min = self.focus_goal_min.clamp(10, 720);
        self
    }

    pub fn in_work_hours(&self, now: DateTime<Local>) -> bool {
        if !self.work_hours_enabled {
            return true;
        }
        if self.weekdays_only && now.weekday().number_from_monday() > 5 {
            return false;
        }
        let (Ok(start), Ok(end)) = (
            NaiveTime::parse_from_str(&self.work_start, "%H:%M"),
            NaiveTime::parse_from_str(&self.work_end, "%H:%M"),
        ) else {
            return true;
        };
        let t = now.time();
        if start <= end {
            start <= t && t < end
        } else {
            t >= start || t < end
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DayStats {
    pub water_ml: u32,
    pub glasses: u32,
    pub breaks: u32,
    pub snoozes: u32,
    pub focus_secs: u32,
    pub focus_sessions: u32,
}

/// Bu skorun üstündeki günler "iyi gün" sayılır ve seriyi sürdürür.
pub const GOOD_DAY_SCORE: u32 = 70;

/// Günün 0–100 arası skoru: açık olan hedeflerin (su, mola, kullanıldıysa odak) ortalaması.
pub fn day_score(d: &DayStats, s: &Settings) -> u32 {
    let ratio = |value: u32, goal: u32| (value as f64 / goal.max(1) as f64).min(1.0);
    let mut parts = Vec::new();
    if s.water_enabled {
        parts.push(ratio(d.water_ml, s.water_goal_ml));
    }
    if s.move_enabled {
        parts.push(ratio(d.breaks, s.breaks_goal));
    }
    if d.focus_secs > 0 {
        parts.push(ratio(d.focus_secs / 60, s.focus_goal_min));
    }
    if parts.is_empty() {
        return 0;
    }
    (parts.iter().sum::<f64>() / parts.len() as f64 * 100.0).round() as u32
}

/// Bugünden geriye kesintisiz "iyi gün" sayısı. Bugün henüz bitmediği için iyi değilse seriyi
/// bozmaz; yalnızca hafta içi çalışılıyorsa hafta sonları atlanır.
pub fn streak(history: &History, s: &Settings, today: NaiveDate) -> u32 {
    let good = |date: NaiveDate| {
        history
            .get(&day_key(date))
            .is_some_and(|d| day_score(d, s) >= GOOD_DAY_SCORE)
    };
    let mut count = 0;
    let mut date = today;
    for i in 0..366 {
        let weekend = date.weekday().number_from_monday() > 5;
        if good(date) {
            count += 1;
        } else if !(i == 0 || (s.weekdays_only && weekend)) {
            break;
        }
        let Some(prev) = date.pred_opt() else { break };
        date = prev;
    }
    count
}

/// Gün anahtarı ("YYYY-MM-DD") → o günün istatistikleri.
pub type History = BTreeMap<String, DayStats>;

pub fn day_key(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    pub fn load_settings(&self) -> Option<Settings> {
        read_json(&self.dir.join("settings.json"))
    }

    pub fn save_settings(&self, settings: &Settings) {
        write_json(&self.dir.join("settings.json"), settings);
    }

    pub fn load_history(&self) -> History {
        read_json(&self.dir.join("history.json")).unwrap_or_default()
    }

    pub fn save_history(&self, history: &History) {
        write_json(&self.dir.join("history.json"), history);
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Yarım yazılmış dosya kalmasın diye önce geçici dosyaya yazıp yerine taşır.
fn write_json<T: Serialize>(path: &Path, value: &T) {
    let Ok(json) = serde_json::to_string_pretty(value) else { return };
    let tmp = path.with_extension("json.tmp");
    if fs::write(&tmp, json).is_ok() {
        let _ = fs::rename(&tmp, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> DayStats {
        DayStats { water_ml: 2500, breaks: 6, ..DayStats::default() }
    }

    #[test]
    fn score_averages_enabled_goals() {
        let s = Settings::default();
        assert_eq!(day_score(&good(), &s), 100);
        let half = DayStats { water_ml: 1250, breaks: 3, ..DayStats::default() };
        assert_eq!(day_score(&half, &s), 50);
        let no_water = Settings { water_enabled: false, ..Settings::default() };
        assert_eq!(day_score(&half, &no_water), 50);
    }

    #[test]
    fn streak_skips_unfinished_today_and_weekends() {
        let s = Settings::default();
        // 2026-11-02 Pazartesi; 31 Ekim–1 Kasım hafta sonu.
        let today = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        let mut h = History::new();
        for d in ["2026-10-28", "2026-10-29", "2026-10-30"] {
            h.insert(d.into(), good());
        }
        assert_eq!(streak(&h, &s, today), 3);
        h.insert("2026-11-02".into(), good());
        assert_eq!(streak(&h, &s, today), 4);
        h.remove("2026-10-29");
        assert_eq!(streak(&h, &s, today), 2);
    }
}
