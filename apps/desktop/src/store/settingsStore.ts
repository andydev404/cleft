import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "@/store/toastStore";

const STORAGE_KEY = "cleft:settings";

interface StoredSettings {
  darkSync: boolean;
  sound: boolean;
}

function loadStored(): StoredSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { darkSync: true, sound: false };
    const parsed = JSON.parse(raw);
    return { darkSync: parsed.darkSync ?? true, sound: parsed.sound ?? false };
  } catch {
    return { darkSync: true, sound: false };
  }
}

function saveStored(s: StoredSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
}

interface SettingsState {
  launch: boolean;
  darkSync: boolean;
  sound: boolean;
  init: () => Promise<void>;
  toggle: (key: "launch" | "darkSync" | "sound") => void;
}

// launch-at-login mirrors the OS's actual login-item registration (queried
// via is_autostart_enabled on init) — it's the one setting here that isn't
// just a local preference, so it isn't part of the localStorage blob below.
// darkSync/sound are real, applied preferences now: darkSync gates
// useSystemTheme's live sync (off = force light), sound gates the capture
// blip in useAppEvents.
export const useSettingsStore = create<SettingsState>((set, get) => ({
  launch: false,
  ...loadStored(),

  init: async () => {
    const enabled = await invoke<boolean>("is_autostart_enabled").catch(() => false);
    set({ launch: enabled });
  },

  toggle: (key) => {
    if (key === "launch") {
      const next = !get().launch;
      set({ launch: next });
      invoke("set_autostart", { enabled: next }).catch(() => {
        set({ launch: !next });
        useToastStore.getState().show("Couldn't change login item");
      });
      return;
    }
    const next = !get()[key];
    set({ [key]: next } as Partial<SettingsState>);
    saveStored({ darkSync: get().darkSync, sound: get().sound });
  },
}));
