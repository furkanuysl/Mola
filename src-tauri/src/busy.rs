//! Kullanıcının rahatsız edilmemesi gereken durumlar: tam ekran uygulama ya da görüşme.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Busy {
    Fullscreen,
    Call,
}

pub fn detect(fullscreen: bool, calls: bool) -> Option<Busy> {
    if calls && microphone_in_use() {
        return Some(Busy::Call);
    }
    if fullscreen && fullscreen_app() {
        return Some(Busy::Fullscreen);
    }
    None
}

/// Windows'un bildirimleri bastırmak için kullandığı durum (oyun, sunum, tam ekran video);
/// yakalanamazsa ön plandaki pencerenin tüm monitörü kaplayıp kaplamadığına bakılır.
#[cfg(windows)]
fn fullscreen_app() -> bool {
    use windows_sys::Win32::UI::Shell::{
        SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
    };

    let mut state = 0;
    let ok = unsafe { SHQueryUserNotificationState(&mut state) } == 0;
    if ok && matches!(state, QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE) {
        return true;
    }
    foreground_covers_monitor()
}

#[cfg(windows)]
fn foreground_covers_monitor() -> bool {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONULL};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetShellWindow, GetWindowLongW, GetWindowRect, GWL_STYLE, WS_CAPTION,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() || hwnd == GetShellWindow() {
            return false;
        }
        // Masaüstü de ekranı kaplar; tam ekran sayılmamalı.
        let mut class = [0u16; 32];
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
        let class = String::from_utf16_lossy(&class[..len.max(0) as usize]);
        if class == "WorkerW" || class == "Progman" {
            return false;
        }
        // Başlık çubuğu olan pencere (ör. gizlenen görev çubuğuyla büyütülmüş) tam ekran değildir.
        if GetWindowLongW(hwnd, GWL_STYLE) as u32 & WS_CAPTION == WS_CAPTION {
            return false;
        }

        let mut rect: RECT = zeroed();
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return false;
        }
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        if monitor.is_null() {
            return false;
        }
        let mut info: MONITORINFO = zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return false;
        }
        let m = info.rcMonitor;
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}

/// Windows, mikrofonu kullanan her uygulama için başlangıç/bitiş zamanını kayıt defterine yazar
/// (görev çubuğundaki "mikrofon kullanılıyor" simgesinin kaynağı). Bitiş 0 ise şu an kullanımdadır.
/// Ses verisine erişilmez, yalnızca bu zaman damgalarına bakılır.
#[cfg(windows)]
fn microphone_in_use() -> bool {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    const PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone";

    fn in_use(key: &RegKey) -> bool {
        let start: u64 = key.get_value("LastUsedTimeStart").unwrap_or(0);
        let stop: u64 = key.get_value("LastUsedTimeStop").unwrap_or(1);
        start > 0 && stop == 0
    }

    let Ok(root) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(PATH) else {
        return false;
    };
    root.enum_keys().flatten().any(|name| {
        let Ok(key) = root.open_subkey(&name) else { return false };
        // Mağaza dışı uygulamalar (Zoom, Discord, tarayıcılar…) "NonPackaged" altında listelenir.
        if name == "NonPackaged" {
            key.enum_keys()
                .flatten()
                .any(|app| key.open_subkey(&app).is_ok_and(|k| in_use(&k)))
        } else {
            in_use(&key)
        }
    })
}

#[cfg(not(windows))]
fn fullscreen_app() -> bool {
    false
}

#[cfg(not(windows))]
fn microphone_in_use() -> bool {
    false
}
