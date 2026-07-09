import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

const PERMISSION_ASKED_KEY = "cleft:permission-asked";

interface RunwayState {
  trusted: boolean | null;
  init: () => Promise<void>;
  requestPermission: () => void;
}

let pollHandle: ReturnType<typeof setInterval> | null = null;

// No custom pre-explanation screen — macOS's own Accessibility dialog is
// the explanation. Just checks status once and asks automatically on first
// launch; there's no manual re-ask UI anymore (the banner/tray item for it
// were removed as dead weight — Settings still shows the real, accurate
// permission-independent info).
export const useRunwayStore = create<RunwayState>((set, get) => ({
  trusted: null,

  init: async () => {
    const isTrusted = await invoke<boolean>("check_accessibility_trusted");
    set({ trusted: isTrusted });
    if (isTrusted) {
      // Already trusted on first launch — always the case on Windows,
      // where no Accessibility permission exists. Mark onboarded now or
      // lib.rs would force-show the palette on every startup. Idempotent.
      invoke("mark_onboarded");
      return;
    }
    if (localStorage.getItem(PERMISSION_ASKED_KEY) !== "true") {
      localStorage.setItem(PERMISSION_ASKED_KEY, "true");
      // Mark onboarded regardless of what the user does with the system
      // dialog — this only gates "reveal the window unprompted on first
      // launch" (see lib.rs), not permission state itself. Otherwise
      // denying would force the window open on every subsequent launch.
      invoke("mark_onboarded");
      get().requestPermission();
    }
  },

  requestPermission: () => {
    invoke("request_accessibility_trust");

    // The grant only takes effect once the user responds in System
    // Settings, not synchronously — poll until it does (or give up).
    let attempts = 0;
    if (pollHandle) clearInterval(pollHandle);
    pollHandle = setInterval(async () => {
      attempts += 1;
      const isTrusted = await invoke<boolean>("check_accessibility_trusted");
      if (isTrusted || attempts >= 30) {
        if (pollHandle) clearInterval(pollHandle);
        set({ trusted: isTrusted });
        if (isTrusted) invoke("mark_onboarded");
      }
    }, 1000);
  },
}));
