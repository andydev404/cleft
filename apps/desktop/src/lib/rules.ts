import type { ActionKind, AutomationRule, TriggerKind } from "@/types";

export const TRIGGER_LABELS: Record<TriggerKind, string> = {
  AppIs: "Copied from app",
  UrlContains: "URL contains",
  ContentTypeIs: "Content type is",
  WindowTitleContains: "Window title contains",
  ContentContains: "Content contains",
};

export const ACTION_LABELS: Record<ActionKind, string> = {
  AssignCollection: "Assign to collection",
  AddTag: "Add tag",
  Pin: "Pin automatically",
  Block: "Block (never save)",
  AssignWorkspace: "Assign workspace",
};

// Actions with no meaningful value — the rule form hides the value input
// for these, and the value column is empty in the list.
export const VALUELESS_ACTIONS: ActionKind[] = ["Pin", "Block"];

export function formatTrigger(r: AutomationRule): string {
  switch (r.trigger_kind) {
    case "AppIs":
      return `app == "${r.trigger_value}"`;
    case "ContentTypeIs":
      return `type == ${r.trigger_value}`;
    case "UrlContains":
      return `url contains "${r.trigger_value}"`;
    case "WindowTitleContains":
      return `title contains "${r.trigger_value}"`;
    case "ContentContains":
      return `content contains "${r.trigger_value}"`;
  }
}

export function formatAction(r: AutomationRule): string {
  switch (r.action_kind) {
    case "AssignCollection":
      return `→ ${r.action_value}`;
    case "AssignWorkspace":
      return `workspace = ${r.action_value}`;
    case "AddTag":
      return `# ${r.action_value}`;
    case "Pin":
      return "pin";
    case "Block":
      return "block";
  }
}
