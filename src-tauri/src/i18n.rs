//! Rust tarafında gösterilen metinler (tepsi menüsü ve ipucu).

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Tr,
    En,
}

impl Lang {
    /// "auto" ise sistem diline bakar: Türkçe değilse İngilizce.
    pub fn resolve(setting: &str) -> Self {
        match setting {
            "tr" => Lang::Tr,
            "en" => Lang::En,
            _ if sys_locale::get_locale().is_some_and(|l| l.to_lowercase().starts_with("tr")) => Lang::Tr,
            _ => Lang::En,
        }
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::Tr => &TR,
            Lang::En => &EN,
        }
    }
}

pub struct Texts {
    pub water_done: &'static str,
    pub take_break: &'static str,
    pub pause_hour: &'static str,
    pub resume: &'static str,
    pub open: &'static str,
    pub quit: &'static str,
    pub water: &'static str,
    pub breaks: &'static str,
    pub paused: &'static str,
}

static TR: Texts = Texts {
    water_done: "Su içtim",
    take_break: "Şimdi mola ver",
    pause_hour: "1 saat duraklat",
    resume: "Devam et",
    open: "Paneli aç",
    quit: "Çıkış",
    water: "Su",
    breaks: "Mola",
    paused: "duraklatıldı",
};

static EN: Texts = Texts {
    water_done: "I drank water",
    take_break: "Take a break now",
    pause_hour: "Pause for 1 hour",
    resume: "Resume",
    open: "Open dashboard",
    quit: "Quit",
    water: "Water",
    breaks: "Breaks",
    paused: "paused",
};
