import { useEffect } from "react";

// Nothing was previously toggling the `.dark` class that index.css's dark
// palette depends on — the app always rendered light-mode colors
// regardless of actual system appearance. This keeps it in sync, including
// live switches while the app is running — unless the user's turned off
// "Follow system appearance" in Settings, in which case it just forces
// light (there's no separate manual light/dark picker in this app, so
// "don't follow system" and "light" are the same thing here).
export function useSystemTheme(follow: boolean) {
  useEffect(() => {
    if (!follow) {
      document.documentElement.classList.remove("dark");
      return;
    }
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = (isDark: boolean) => document.documentElement.classList.toggle("dark", isDark);

    apply(query.matches);
    const onChange = (e: MediaQueryListEvent) => apply(e.matches);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, [follow]);
}
