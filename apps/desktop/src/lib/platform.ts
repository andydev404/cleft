// navigator.platform is soft-deprecated but still universally supported
// and is the simplest check available without pulling in a Tauri OS-info
// plugin just for this. Used anywhere the UI shows a keyboard shortcut,
// since ⌘ means nothing on Windows and Ctrl is the real modifier there.
export const isMac = navigator.platform.toLowerCase().includes("mac");

export const modKey = isMac ? "⌘" : "Ctrl";
