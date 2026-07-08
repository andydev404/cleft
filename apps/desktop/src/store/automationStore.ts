import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ActionKind, AutomationRule, TriggerKind } from "@/types";

interface AutomationState {
  rules: AutomationRule[];
  load: () => Promise<void>;
  setEnabled: (id: string, enabled: boolean) => Promise<string | null>;
  remove: (id: string) => Promise<void>;
  create: (trigger_kind: TriggerKind, trigger_value: string, action_kind: ActionKind, action_value: string) => Promise<string | null>;
  update: (id: string, trigger_kind: TriggerKind, trigger_value: string, action_kind: ActionKind, action_value: string) => Promise<string | null>;
}

// Real rules, evaluated at capture time in clipboard.rs (see automation.rs)
// — replaces what used to be a purely local, cosmetic toggle.
export const useAutomationStore = create<AutomationState>((set, get) => ({
  rules: [],

  load: async () => {
    const rules = await invoke<AutomationRule[]>("list_rules");
    set({ rules });
  },

  setEnabled: async (id, enabled) => {
    try {
      await invoke("set_rule_enabled", { id, enabled });
      set({ rules: get().rules.map((r) => (r.id === id ? { ...r, enabled } : r)) });
      return null;
    } catch (e) {
      return String(e);
    }
  },

  remove: async (id) => {
    await invoke("delete_rule", { id });
    set({ rules: get().rules.filter((r) => r.id !== id) });
  },

  create: async (trigger_kind, trigger_value, action_kind, action_value) => {
    try {
      const rule = await invoke<AutomationRule>("create_rule", { triggerKind: trigger_kind, triggerValue: trigger_value, actionKind: action_kind, actionValue: action_value });
      set({ rules: [...get().rules, rule] });
      return null;
    } catch (e) {
      return String(e);
    }
  },

  update: async (id, trigger_kind, trigger_value, action_kind, action_value) => {
    try {
      await invoke("update_rule", { id, triggerKind: trigger_kind, triggerValue: trigger_value, actionKind: action_kind, actionValue: action_value });
      set({
        rules: get().rules.map((r) =>
          r.id === id ? { ...r, trigger_kind, trigger_value, action_kind, action_value } : r
        ),
      });
      return null;
    } catch (e) {
      return String(e);
    }
  },
}));
