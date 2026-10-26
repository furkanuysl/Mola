//! Kullanıcının son klavye/fare girdisinden bu yana geçen süre.

#[cfg(windows)]
pub fn idle_seconds() -> u64 {
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        if GetLastInputInfo(&mut info) == 0 {
            return 0;
        }
        (GetTickCount().wrapping_sub(info.dwTime) / 1000) as u64
    }
}

#[cfg(not(windows))]
pub fn idle_seconds() -> u64 {
    0
}
