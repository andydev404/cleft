#[derive(Clone)]
pub struct FrontmostApp {
    pub bundle_id: String,
    pub pid: i32,
}

// NSWorkspace.frontmostApplication requires no macOS permission at all —
// unlike AppleScript's "tell application" (never used, see sensitive.rs),
// this is a plain OS query.
#[cfg(target_os = "macos")]
pub fn current() -> Option<FrontmostApp> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    let bundle_id = app.bundleIdentifier()?.to_string();
    let pid = app.processIdentifier();
    Some(FrontmostApp { bundle_id, pid })
}

// No bundle-id concept on Windows, so the executable's file name (e.g.
// "1password.exe") stands in for it everywhere bundle_id is compared —
// the blocklist/automation "AppIs" matching, and sensitive.rs's hardcoded
// blacklist, both just need a stable, lowercased identifier per app.
#[cfg(target_os = "windows")]
pub fn current() -> Option<FrontmostApp> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let exe_name = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .ok()
        .map(|_| {
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            path.rsplit(['\\', '/'])
                .next()
                .unwrap_or(&path)
                .to_lowercase()
        });
        let _ = CloseHandle(handle);

        let bundle_id = exe_name.filter(|n| !n.is_empty())?;
        Some(FrontmostApp {
            bundle_id,
            pid: pid as i32,
        })
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn current() -> Option<FrontmostApp> {
    None
}
