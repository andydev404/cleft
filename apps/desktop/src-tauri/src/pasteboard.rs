// NSPasteboard.changeCount, not content-string diffing, is what decides
// whether something new was copied. It's a cheap counter read regardless
// of which app is frontmost, so it can be checked *before* any blocking
// decision — which matters: if we only tracked "last saved content", a
// clip that was correctly blocked while a blocked app was frontmost would
// look "new" again the instant the app loses focus, and get captured
// retroactively. Bumping this counter first closes that gap.
#[cfg(target_os = "macos")]
pub fn change_count() -> isize {
    use objc2_app_kit::NSPasteboard;

    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.changeCount()
}

#[cfg(not(target_os = "macos"))]
pub fn change_count() -> isize {
    0
}
