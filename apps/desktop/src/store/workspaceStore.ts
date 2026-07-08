import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore } from "@/store/clipboardStore";
import type { Workspace } from "@/types";

interface WorkspaceState {
  workspaces: Workspace[];
  load: () => Promise<void>;
  switchTo: (name: string) => Promise<void>;
  create: (name: string) => Promise<string | null>;
  remove: (name: string) => Promise<string | null>;
}

// Workspaces don't share history — switching reloads the clip list from
// whichever workspace is now current.
export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  workspaces: [],

  load: async () => {
    const workspaces = await invoke<Workspace[]>("list_workspaces");
    set({ workspaces });
  },

  switchTo: async (name) => {
    await invoke("switch_workspace", { name });
    set((s) => ({ workspaces: s.workspaces.map((w) => ({ ...w, is_current: w.name === name })) }));
    useClipboardStore.getState().clearSelection();
    await useClipboardStore.getState().loadHistory();
  },

  create: async (name) => {
    try {
      await invoke("create_workspace", { name });
      const workspaces = await invoke<Workspace[]>("list_workspaces");
      set({ workspaces });
      return null;
    } catch (e) {
      return String(e);
    }
  },

  // Deleting a workspace deletes its clips with it (they don't exist
  // anywhere else — see db::delete_workspace) — this always runs behind a
  // confirmation dialog, see WorkspaceSwitcher.tsx.
  remove: async (name) => {
    try {
      await invoke("delete_workspace", { name });
      const workspaces = await invoke<Workspace[]>("list_workspaces");
      set({ workspaces });
      useClipboardStore.getState().clearSelection();
      await useClipboardStore.getState().loadHistory();
      return null;
    } catch (e) {
      return String(e);
    }
  },
}));
