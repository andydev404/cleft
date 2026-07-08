//! Window-context capture (focused window title, best-effort URL) plus the
//! macOS Accessibility permission that gates it there. One shared public
//! API — `is_trusted`, `request_trust`, `fetch_context`, `ClipContext` —
//! backed by ApplicationServices' AXUIElement API on macOS and plain Win32
//! calls on Windows, since neither the window-title APIs nor the
//! permission concept map 1:1 across platforms.

pub struct ClipContext {
    pub window_title: Option<String>,
    pub url: Option<String>,
}

#[cfg(target_os = "macos")]
pub use mac::*;

#[cfg(target_os = "windows")]
pub use win::*;

#[cfg(target_os = "macos")]
mod mac {
    use super::ClipContext;
    use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::{CFString, CFStringRef};
    use core_foundation::url::{CFURLRef, CFURL};
    use std::ffi::c_void;
    use std::os::raw::c_int;

    #[repr(C)]
    struct OpaqueAXUIElement(c_void);
    type AXUIElementRef = *const OpaqueAXUIElement;
    type AXError = i32;

    const AX_ERROR_SUCCESS: AXError = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(
            options: core_foundation::dictionary::CFDictionaryRef,
        ) -> bool;
        fn AXUIElementCreateApplication(pid: c_int) -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> AXError;
    }

    pub fn is_trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    /// Triggers the macOS system permission dialog if not already trusted.
    /// Returns the current (pre-dialog) trust state — the user's actual choice
    /// only takes effect after they respond in System Settings, which the
    /// Permission Runway detects by re-polling `is_trusted`.
    pub fn request_trust() -> bool {
        let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
        let dict = CFDictionary::from_CFType_pairs(&[(
            key.as_CFType(),
            CFBoolean::true_value().as_CFType(),
        )]);
        unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef()) }
    }

    unsafe fn copy_attribute(element: AXUIElementRef, attribute: &str) -> Option<CFTypeRef> {
        let attr = CFString::new(attribute);
        let mut value: CFTypeRef = std::ptr::null();
        let err = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value);
        (err == AX_ERROR_SUCCESS && !value.is_null()).then_some(value)
    }

    fn copy_string_attribute(element: AXUIElementRef, attribute: &str) -> Option<String> {
        unsafe {
            let value = copy_attribute(element, attribute)?;
            Some(CFString::wrap_under_create_rule(value as CFStringRef).to_string())
        }
    }

    fn copy_url_attribute(element: AXUIElementRef, attribute: &str) -> Option<String> {
        unsafe {
            let value = copy_attribute(element, attribute)?;
            Some(
                CFURL::wrap_under_create_rule(value as CFURLRef)
                    .get_string()
                    .to_string(),
            )
        }
    }

    fn copy_element_attribute(element: AXUIElementRef, attribute: &str) -> Option<AXUIElementRef> {
        unsafe { copy_attribute(element, attribute).map(|v| v as AXUIElementRef) }
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFArrayGetCount(array: core_foundation::array::CFArrayRef) -> isize;
        fn CFArrayGetValueAtIndex(
            array: core_foundation::array::CFArrayRef,
            idx: isize,
        ) -> *const c_void;
        fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    }

    // Retains each child before the backing array is released below, since
    // CFArrayGetValueAtIndex doesn't retain on our behalf and callers keep
    // using these pointers after this function returns. Callers must
    // CFRelease each one when done (see find_url_in_subtree).
    fn copy_children(element: AXUIElementRef) -> Vec<AXUIElementRef> {
        unsafe {
            let Some(value) = copy_attribute(element, "AXChildren") else {
                return Vec::new();
            };
            let array = value as core_foundation::array::CFArrayRef;
            let count = CFArrayGetCount(array);
            let children = (0..count)
                .map(|i| {
                    let child = CFArrayGetValueAtIndex(array, i) as AXUIElementRef;
                    CFRetain(child as CFTypeRef);
                    child
                })
                .collect();
            CFRelease(array as CFTypeRef);
            children
        }
    }

    // Chromium browsers (confirmed on Brave) don't put AXURL on the window or
    // the focused element — it lives one level down, on a wrapping AXGroup
    // around the actual web content. Safari/WebKit apps generally expose it
    // higher up, which the direct checks in fetch_context already catch cheap
    // and fast; this bounded walk is the fallback for browsers that nest it.
    // Depth is small and this only runs inside the caller's existing 50ms
    // timeout (see context.rs / clipboard.rs), so a pathological tree just
    // times out rather than ever blocking capture.
    fn find_url_in_subtree(element: AXUIElementRef, max_depth: u8) -> Option<String> {
        if let Some(url) = copy_url_attribute(element, "AXURL") {
            return Some(url);
        }
        if max_depth == 0 {
            return None;
        }
        for child in copy_children(element) {
            let found = find_url_in_subtree(child, max_depth - 1);
            unsafe { CFRelease(child as CFTypeRef) };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// Best-effort: browsers vary in exactly where they expose AXURL in the
    /// tree, and some apps don't expose it at all. When it's missing, callers
    /// fall back to window_title only — never an error, never a retry.
    pub fn fetch_context(pid: i32) -> ClipContext {
        unsafe {
            let app = AXUIElementCreateApplication(pid);
            if app.is_null() {
                return ClipContext {
                    window_title: None,
                    url: None,
                };
            }

            let window = copy_element_attribute(app, "AXFocusedWindow");
            let window_title = window.and_then(|w| copy_string_attribute(w, "AXTitle"));

            let focused_element = copy_element_attribute(app, "AXFocusedUIElement");
            const MAX_URL_SEARCH_DEPTH: u8 = 4;
            let url = window
                .and_then(|w| copy_url_attribute(w, "AXURL"))
                .or_else(|| focused_element.and_then(|el| copy_url_attribute(el, "AXURL")))
                .or_else(|| window.and_then(|w| find_url_in_subtree(w, MAX_URL_SEARCH_DEPTH)));

            if let Some(w) = window {
                CFRelease(w as CFTypeRef);
            }
            if let Some(el) = focused_element {
                CFRelease(el as CFTypeRef);
            }
            CFRelease(app as CFTypeRef);

            ClipContext { window_title, url }
        }
    }
} // mod mac

#[cfg(target_os = "windows")]
mod win {
    use super::ClipContext;

    // Window-title reading via Win32 needs no special permission on
    // Windows, unlike macOS Accessibility — these exist only so callers
    // (context.rs, the frontend's permission check) don't need to branch
    // on platform themselves.
    pub fn is_trusted() -> bool {
        true
    }

    pub fn request_trust() -> bool {
        true
    }

    // URL extraction would need UI Automation COM interop (walking the
    // tree for the browser's address-bar element) — real, but a
    // meaningfully bigger lift than the window-title call below, and this
    // already degrades the same way the macOS side does when AXURL isn't
    // found: window title only, never an error. Left as a follow-up
    // rather than guessed at without a way to verify it here.
    pub fn fetch_context(_pid: i32) -> ClipContext {
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

        let window_title = unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                None
            } else {
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut buf);
                (len > 0).then(|| String::from_utf16_lossy(&buf[..len as usize]))
            }
        };

        ClipContext {
            window_title,
            url: None,
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod fallback {
    use super::ClipContext;

    pub fn is_trusted() -> bool {
        false
    }

    pub fn request_trust() -> bool {
        false
    }

    pub fn fetch_context(_pid: i32) -> ClipContext {
        ClipContext {
            window_title: None,
            url: None,
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use fallback::*;
