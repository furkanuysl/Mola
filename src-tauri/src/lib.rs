mod busy;
mod engine;
mod focus;
mod i18n;
mod idle;
mod store;

use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use chrono::{Local, TimeDelta};
use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, State, WebviewWindowBuilder, WindowEvent, Wry,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use engine::{break_secs, Effects, Engine, Kind, Phase};
use focus::{Focus, Technique};
use busy::Busy;
use i18n::Lang;
use store::{day_key, day_score, streak, DayStats, History, Settings, Store};

const APP_NAME: &str = "Mola";
const TRAY_ID: &str = "main";
/// Otomatik başlatmada verilen argüman; panel bu durumda gizli açılır.
const AUTOSTART_ARG: &str = "--minimized";

struct Inner {
    settings: Settings,
    history: History,
    engine: Engine,
    focus: Focus,
    paused_until: Option<Instant>,
    lang: Lang,
    busy: Option<Busy>,
    tray_text: String,
}

struct AppState {
    inner: Mutex<Inner>,
    store: Store,
    /// Dil değişince menü yeniden kurulduğu için değiştirilebilir tutulur.
    tray_status: Mutex<MenuItem<Wry>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReminderView {
    kind: Kind,
    phase: &'static str,
    break_remaining: u64,
    break_total: u64,
    snoozes_left: u32,
    snooze_min: u32,
    glass_ml: u32,
    water_today: u32,
    water_goal: u32,
    sitting_min: u64,
    break_routine: String,
    /// Süren mola, odak zamanlayıcısının verdiği moladır.
    focus_break: bool,
    /// Sonraki odak turu kullanıcıyı bekliyor / kendiliğinden başladı.
    focus_next: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FocusView {
    technique: Technique,
    phase: focus::Phase,
    elapsed: u64,
    target: Option<u64>,
    paused: bool,
    round: u32,
    rounds: Option<u32>,
    break_remaining: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    water_ml: u32,
    water_goal: u32,
    glasses: u32,
    breaks: u32,
    snoozes: u32,
    sitting_min: u64,
    next_water_min: Option<u64>,
    next_move_min: Option<u64>,
    paused_min: Option<u64>,
    in_work_hours: bool,
    busy: Option<Busy>,
    focus_min: u32,
    score: u32,
    streak: u32,
    focus: FocusView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryDay {
    date: String,
    score: u32,
    #[serde(flatten)]
    stats: DayStats,
}

impl Inner {
    fn today(&self) -> DayStats {
        let key = day_key(Local::now().date_naive());
        self.history.get(&key).copied().unwrap_or_default()
    }

    fn today_mut(&mut self) -> &mut DayStats {
        self.history.entry(day_key(Local::now().date_naive())).or_default()
    }

    fn add_glass(&mut self) -> Effects {
        let fx = self.engine.drink();
        let glass = self.settings.glass_ml;
        let day = self.today_mut();
        day.water_ml += glass;
        day.glasses += 1;
        fx
    }

    fn running(&self) -> bool {
        self.paused_until.is_none() && self.settings.in_work_hours(Local::now())
    }

    fn reminder_view(&self) -> Option<ReminderView> {
        let active = self.engine.active?;
        let s = &self.settings;
        let (phase, break_remaining) = match active.phase {
            Phase::Prompt => ("prompt", 0),
            Phase::Break { remaining } => ("break", remaining),
            Phase::Done { .. } => ("done", 0),
        };
        Some(ReminderView {
            kind: active.kind,
            phase,
            break_remaining,
            break_total: self.engine.break_total,
            snoozes_left: self.engine.snoozes_left(active.kind, s),
            snooze_min: s.snooze_min,
            glass_ml: s.glass_ml,
            water_today: self.today().water_ml,
            water_goal: s.water_goal_ml,
            sitting_min: self.engine.sitting / 60,
            break_routine: s.break_routine.clone(),
            focus_break: active.kind == Kind::Move && self.focus.phase == focus::Phase::Break,
            focus_next: match self.focus.phase {
                focus::Phase::Ready => "ready",
                focus::Phase::Focus => "started",
                _ => "",
            },
        })
    }

    fn focus_view(&self) -> FocusView {
        let f = &self.focus;
        let break_remaining = match self.engine.active {
            Some(engine::Active { kind: Kind::Move, phase: Phase::Break { remaining } })
                if f.phase == focus::Phase::Break =>
            {
                Some(remaining)
            }
            _ => None,
        };
        FocusView {
            technique: f.technique,
            phase: f.phase,
            elapsed: f.elapsed,
            target: f.target_secs(&self.settings),
            paused: f.paused,
            round: f.completed + u32::from(f.phase != focus::Phase::Break),
            rounds: (f.technique == Technique::Pomodoro).then_some(self.settings.pomodoro_rounds),
            break_remaining,
        }
    }

    fn status(&self) -> Status {
        let s = &self.settings;
        let today = self.today();
        let mins_left =
            |interval_min: u32, elapsed: u64| (interval_min as u64 * 60).saturating_sub(elapsed).div_ceil(60);
        Status {
            water_ml: today.water_ml,
            water_goal: s.water_goal_ml,
            glasses: today.glasses,
            breaks: today.breaks,
            snoozes: today.snoozes,
            sitting_min: self.engine.sitting / 60,
            next_water_min: s
                .water_enabled
                .then(|| mins_left(s.water_interval_min, self.engine.water_elapsed)),
            next_move_min: s
                .move_enabled
                .then(|| mins_left(s.move_interval_min, self.engine.sitting)),
            paused_min: self
                .paused_until
                .map(|t| t.saturating_duration_since(Instant::now()).as_secs().div_ceil(60)),
            in_work_hours: s.in_work_hours(Local::now()),
            busy: self.busy,
            focus_min: today.focus_secs / 60,
            score: day_score(&today, s),
            streak: streak(&self.history, s, Local::now().date_naive()),
            focus: self.focus_view(),
        }
    }

    fn tray_label(&self) -> String {
        let t = self.lang.texts();
        let today = self.today();
        let mut text = format!(
            "{}: {} / {} ml  ·  {}: {}",
            t.water, today.water_ml, self.settings.water_goal_ml, t.breaks, today.breaks
        );
        if self.paused_until.is_some() {
            text.push_str(&format!("  ({})", t.paused));
        }
        text
    }
}

/// Durumu değiştirir; ardından diske yazma, pencere, olay ve tepsi güncellemelerini yapar.
fn update<T>(app: &AppHandle, f: impl FnOnce(&mut Inner) -> (Effects, T)) -> T {
    let state = app.state::<AppState>();
    let (fx, out, view, status, history, tray_text) = {
        let mut inner = state.inner.lock().unwrap();
        let (fx, out) = f(&mut inner);
        if fx.break_done {
            inner.today_mut().breaks += 1;
        }
        let history = (fx.stats || fx.break_done).then(|| inner.history.clone());
        let label = inner.tray_label();
        let tray_text = (label != inner.tray_text).then(|| {
            inner.tray_text = label.clone();
            label
        });
        (fx, out, inner.reminder_view(), inner.status(), history, tray_text)
    };

    if let Some(history) = history {
        state.store.save_history(&history);
    }
    if fx.changed {
        let _ = app.emit("reminder", &view);
    }
    if fx.show {
        show_reminder(app);
    }
    if fx.hide {
        if let Some(window) = app.get_webview_window("reminder") {
            let _ = window.hide();
        }
    }
    let _ = app.emit("status", &status);
    if let Some(text) = tray_text {
        let _ = state.tray_status.lock().unwrap().set_text(&text);
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_tooltip(Some(format!("{APP_NAME}\n{text}")));
        }
    }
    out
}

/// Hatırlatma penceresini ekranın sağ alt köşesinde, görev çubuğunun üstünde gösterir.
fn show_reminder(app: &AppHandle) {
    let Some(window) = app.get_webview_window("reminder") else { return };
    if let (Ok(Some(monitor)), Ok(size)) = (window.primary_monitor(), window.outer_size()) {
        let area = monitor.work_area();
        let margin = (16.0 * monitor.scale_factor()) as i32;
        let x = area.position.x + area.size.width as i32 - size.width as i32 - margin;
        let y = area.position.y + area.size.height as i32 - size.height as i32 - margin;
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    let _ = window.show();
    let _ = window.set_always_on_top(true);
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn start_ticker(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let idle = idle::idle_seconds();
        update(&app, |inner| {
            if inner.paused_until.is_some_and(|t| Instant::now() >= t) {
                inner.paused_until = None;
            }
            let running = inner.running();
            let s = &inner.settings;
            inner.busy = busy::detect(s.quiet_in_fullscreen, s.quiet_in_calls);
            let busy = inner.busy.is_some();

            let focus = inner.focus.tick(&inner.settings, busy);
            if focus.counted {
                let day = inner.today_mut();
                day.focus_secs += 1;
            }
            if let Some(secs) = focus.break_due {
                inner.today_mut().focus_sessions += 1;
                inner.engine.request_break(secs);
            }
            inner.engine.focus_active = inner.focus.is_active();

            let mut fx = inner.engine.tick(&inner.settings, idle, running, busy);
            if fx.break_done {
                inner.focus.break_finished(&inner.settings);
                inner.engine.focus_active = inner.focus.is_active();
            }
            // Odak süresi dakikada bir ve tur bitince diske yazılır.
            let focus_secs = inner.today().focus_secs;
            if focus.break_due.is_some() || (focus.counted && focus_secs % 60 == 0) {
                fx.stats = true;
            }
            (fx, ())
        });
    });
}

#[tauri::command]
fn get_reminder(state: State<AppState>) -> Option<ReminderView> {
    state.inner.lock().unwrap().reminder_view()
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Status {
    state.inner.lock().unwrap().status()
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.inner.lock().unwrap().settings.clone()
}

#[tauri::command]
fn get_history(state: State<AppState>, days: u32) -> Vec<HistoryDay> {
    let inner = state.inner.lock().unwrap();
    let today = Local::now().date_naive();
    (0..days.min(60) as i64)
        .rev()
        .map(|i| {
            let date = day_key(today - TimeDelta::days(i));
            let stats = inner.history.get(&date).copied().unwrap_or_default();
            HistoryDay { date, score: day_score(&stats, &inner.settings), stats }
        })
        .collect()
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Settings {
    let settings = settings.sanitized();
    let autolaunch = app.autolaunch();
    let _ = if settings.autostart { autolaunch.enable() } else { autolaunch.disable() };
    let state = app.state::<AppState>();
    state.store.save_settings(&settings);

    let lang = Lang::resolve(&settings.language);
    let lang_changed = state.inner.lock().unwrap().lang != lang;
    if lang_changed {
        if let (Ok((menu, status)), Some(tray)) = (build_menu(&app, lang), app.tray_by_id(TRAY_ID)) {
            let _ = tray.set_menu(Some(menu));
            *state.tray_status.lock().unwrap() = status;
        }
        let _ = app.emit("language", lang);
    }

    update(&app, |inner| {
        inner.settings = settings.clone();
        inner.lang = lang;
        // Tepsi metni yeni dilde yeniden yazılsın.
        inner.tray_text.clear();
        (Effects { changed: true, ..Default::default() }, settings)
    })
}

#[tauri::command]
fn get_language(state: State<AppState>) -> Lang {
    state.inner.lock().unwrap().lang
}

#[tauri::command]
fn reminder_action(app: AppHandle, action: String) -> Result<(), String> {
    update(&app, |inner| {
        let result = match action.as_str() {
            "drink" => Ok(inner.add_glass()),
            "skip" => inner.engine.skip(),
            "snooze" => inner.engine.snooze(&inner.settings).map(|fx| {
                inner.today_mut().snoozes += 1;
                fx
            }),
            "start_break" => inner.engine.start_break(&inner.settings),
            "dismiss" => Ok(inner.engine.dismiss()),
            _ => Err("unknown_action".into()),
        };
        match result {
            Ok(fx) => (fx, Ok(())),
            Err(e) => (Effects::default(), Err(e)),
        }
    })
}

#[tauri::command]
fn add_water(app: AppHandle) {
    update(&app, |inner| (inner.add_glass(), ()));
}

#[tauri::command]
fn take_break(app: AppHandle) {
    update(&app, |inner| {
        let secs = break_secs(&inner.settings);
        inner.engine.request_break(secs);
        (Effects::default(), ())
    });
}

#[tauri::command]
fn pause(app: AppHandle, minutes: u64) {
    update(&app, |inner| {
        inner.paused_until = Some(Instant::now() + Duration::from_secs(minutes * 60));
        (Effects::default(), ())
    });
}

#[tauri::command]
fn resume(app: AppHandle) {
    update(&app, |inner| {
        inner.paused_until = None;
        (Effects::default(), ())
    });
}

#[tauri::command]
fn focus_start(app: AppHandle, technique: String) -> Result<(), String> {
    let technique = Technique::parse(&technique).ok_or("unknown_technique")?;
    let settings = update(&app, |inner| {
        inner.focus.start(technique);
        inner.engine.focus_active = true;
        inner.settings.focus_technique = technique_id(technique).into();
        (Effects { changed: true, ..Default::default() }, inner.settings.clone())
    });
    app.state::<AppState>().store.save_settings(&settings);
    Ok(())
}

fn technique_id(t: Technique) -> &'static str {
    match t {
        Technique::Pomodoro => "pomodoro",
        Technique::Fifty2 => "52_17",
        Technique::Ninety => "90_20",
        Technique::Flowtime => "flowtime",
    }
}

#[tauri::command]
fn focus_pause(app: AppHandle, paused: bool) {
    update(&app, |inner| {
        inner.focus.paused = paused;
        (Effects::default(), ())
    });
}

#[tauri::command]
fn focus_stop(app: AppHandle) {
    update(&app, |inner| {
        inner.focus.stop();
        inner.engine.focus_active = false;
        (Effects { stats: true, ..Default::default() }, ())
    });
}

/// Odağı erken bitirip molaya geçer (Flowtime'da molanın tek yolu budur).
#[tauri::command]
fn focus_break_now(app: AppHandle) {
    update(&app, |inner| {
        if let Some(secs) = inner.focus.break_now(&inner.settings) {
            inner.today_mut().focus_sessions += 1;
            inner.engine.request_break(secs);
        }
        (Effects { stats: true, ..Default::default() }, ())
    });
}

/// Tepsiden: odak yoksa ayarlı teknikle başlatır, varsa durdurur.
fn focus_toggle(app: &AppHandle) {
    let (active, technique) = {
        let state = app.state::<AppState>();
        let inner = state.inner.lock().unwrap();
        (inner.focus.is_active(), inner.settings.focus_technique.clone())
    };
    if active {
        focus_stop(app.clone());
    } else {
        let _ = focus_start(app.clone(), technique);
    }
}

/// Tepsi menüsünü verilen dilde kurar; durum satırını ayrıca döndürür ki metni güncellenebilsin.
fn build_menu(app: &AppHandle, lang: Lang) -> tauri::Result<(Menu<Wry>, MenuItem<Wry>)> {
    let t = lang.texts();
    let status = MenuItem::with_id(app, "status", APP_NAME, false, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &status,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "water", t.water_done, true, None::<&str>)?,
            &MenuItem::with_id(app, "break", t.take_break, true, None::<&str>)?,
            &MenuItem::with_id(app, "focus", t.focus_toggle, true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "pause", t.pause_hour, true, None::<&str>)?,
            &MenuItem::with_id(app, "resume", t.resume, true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "open", t.open, true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", t.quit, true, None::<&str>)?,
        ],
    )?;
    Ok((menu, status))
}

fn build_tray(app: &AppHandle, lang: Lang) -> tauri::Result<MenuItem<Wry>> {
    let (menu, status) = build_menu(app, lang)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "water" => add_water(app.clone()),
            "break" => take_break(app.clone()),
            "focus" => focus_toggle(app),
            "pause" => pause(app.clone(), 60),
            "resume" => resume(app.clone()),
            "open" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(status)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_main(app)))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .setup(|app| {
            let store = Store::new(app.path().app_data_dir()?);
            let saved = store.load_settings();
            let first_run = saved.is_none();
            let settings = saved.unwrap_or_default().sanitized();
            if first_run {
                store.save_settings(&settings);
            }
            let history = store.load_history();
            let lang = Lang::resolve(&settings.language);
            let tray_status = build_tray(app.handle(), lang)?;
            app.manage(AppState {
                inner: Mutex::new(Inner {
                    settings,
                    history,
                    engine: Engine::default(),
                    focus: Focus::default(),
                    paused_until: None,
                    lang,
                    busy: None,
                    tray_text: String::new(),
                }),
                store,
                tray_status: Mutex::new(tray_status),
            });
            // Pencereler durum kaydedildikten sonra açılır; yoksa sayfa ilk komutları
            // `manage` öncesinde çağırıp hata alır ve panel boş kalır.
            for config in app.config().app.windows.clone() {
                WebviewWindowBuilder::from_config(app.handle(), &config)?.build()?;
            }
            // Windows açılışında sessizce tepsiye; elle açılınca panel görünür.
            if !std::env::args().any(|a| a == AUTOSTART_ARG) {
                show_main(app.handle());
            }
            start_ticker(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Pencereler kapanmaz: panel gizlenir, hatırlatma ise butonlarla kapanır.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_reminder,
            get_status,
            get_settings,
            get_language,
            get_history,
            save_settings,
            reminder_action,
            add_water,
            take_break,
            pause,
            resume,
            focus_start,
            focus_pause,
            focus_stop,
            focus_break_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
