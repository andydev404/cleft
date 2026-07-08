import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useAutomationStore } from "@/store/automationStore";
import { useClipboardStore } from "@/store/clipboardStore";
import { useToastStore } from "@/store/toastStore";
import { useRunwayStore } from "@/store/runwayStore";
import { useUpdateStore } from "@/store/updateStore";
import { useSettingsStore } from "@/store/settingsStore";
import { playCaptureSound } from "@/lib/captureSound";
import type { ClipMetadata } from "@/types";

interface ClipContextUpdate {
  id: string;
  window_title: string | null;
  url: string | null;
}

// Shown once, the first time the rolling history actually wraps — not on
// every eviction after that.
const FIFO_NOTICE_KEY = "cleft:fifo-notice-shown";

// Runs once at the app root — Tauri events and the global Esc listener
// aren't tied to any one route.
export function useAppEvents(onPaletteShown: () => void) {
  useEffect(() => {
    useClipboardStore.getState().loadHistory();
    useAutomationStore.getState().load();
    useUpdateStore.getState().check();

    const unlistenAdded = listen<ClipMetadata>("clip-added", (event) => {
      useClipboardStore.getState().addClip(event.payload);
      if (useSettingsStore.getState().sound) playCaptureSound();
    });
    const unlistenContext = listen<ClipContextUpdate>("clip-context-updated", (event) => {
      useClipboardStore.getState().updateContext(event.payload.id, event.payload.window_title, event.payload.url);
    });
    const unlistenEvicted = listen<string[]>("clips-evicted", (event) => {
      useClipboardStore.getState().removeEvicted(event.payload);
      if (!localStorage.getItem(FIFO_NOTICE_KEY)) {
        localStorage.setItem(FIFO_NOTICE_KEY, "1");
        useToastStore.getState().show("Your oldest clip just dropped off — Cleft keeps your latest 500.");
      }
    });
    // The window is revealed by toggling visibility, not remounting — so
    // "reset to a clean state on open" needs its own listener.
    const unlistenShown = listen("palette-shown", () => {
      useClipboardStore.getState().clearSelection();
      onPaletteShown();
    });
    // Fired by the tray icon's "Permissions…" item — re-triggers the native
    // macOS Accessibility dialog directly, no custom screen in between.
    const unlistenReplayRunway = listen("replay-runway", () => {
      useRunwayStore.getState().requestPermission();
    });
    // Fired by the tray icon's "Check for Updates…" item — unlike the
    // silent startup check, this one always reports back (e.g. "You're up
    // to date") since the user asked directly.
    const unlistenCheckUpdates = listen("check-for-updates", () => {
      useUpdateStore.getState().check(false);
    });

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      // Escape backs out of bulk-select mode first, then closes the
      // palette on a second press — matches how Esc behaves in most list
      // UIs with a selection mode (e.g. Mail, Photos).
      const { bulkSelected, clearBulkSelection } = useClipboardStore.getState();
      if (bulkSelected.size > 0) {
        clearBulkSelection();
        return;
      }
      invoke("hide_palette");
    };
    window.addEventListener("keydown", onKeyDown);

    return () => {
      unlistenAdded.then((f) => f());
      unlistenContext.then((f) => f());
      unlistenEvicted.then((f) => f());
      unlistenShown.then((f) => f());
      unlistenReplayRunway.then((f) => f());
      unlistenCheckUpdates.then((f) => f());
      window.removeEventListener("keydown", onKeyDown);
    };
    // Runs once for the lifetime of the app; onPaletteShown (a router
    // navigate callback) is stable across renders.
  }, []);
}
