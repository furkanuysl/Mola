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
    /// "auto", "tr" veya "en".
    pub language: String,
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
            language: "auto".into(),
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
