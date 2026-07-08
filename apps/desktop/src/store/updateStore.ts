import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "@/store/toastStore";
import { useConfirmStore } from "@/store/confirmStore";

interface UpdateInfo {
  version: string;
  notes: string;
}

interface UpdateStoreState {
  checking: boolean;
  // silent: true for the automatic startup check (say nothing if there's
  // nothing new); false for the tray's "Check for Updates…" (always give
  // feedback, since the user asked directly).
  check: (silent?: boolean) => Promise<void>;
  install: () => Promise<void>;
}

export const useUpdateStore = create<UpdateStoreState>((set, get) => ({
  checking: false,

  check: async (silent = true) => {
    if (get().checking) return;
    set({ checking: true });
    try {
      const info = await invoke<UpdateInfo | null>("check_for_update");
      if (info) {
        useConfirmStore.getState().show({
          title: `Update available — v${info.version}`,
          description: info.notes || "Restart Cleft to install the update.",
          confirmLabel: "Restart & Update",
          variant: "default",
          onConfirm: () => get().install(),
        });
      } else if (!silent) {
        useToastStore.getState().show("You're up to date");
      }
    } catch {
      if (!silent) useToastStore.getState().show("Couldn't check for updates");
    } finally {
      set({ checking: false });
    }
  },

  install: async () => {
    useToastStore.getState().show("Installing update…");
    try {
      await invoke("install_update");
    } catch {
      useToastStore.getState().show("Update failed");
    }
  },
}));
