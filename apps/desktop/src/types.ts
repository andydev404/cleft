export type ContentType =
  | "PlainText"
  | "Code"
  | "URL"
  | "SQL"
  | "JSON"
  | "Markdown"
  | "Color"
  | "Email"
  | "FilePath"
  | "HTML";

export interface ClipMetadata {
  id: string;
  preview: string;
  content_type: ContentType;
  source_app: string;
  window_title: string | null;
  url: string | null;
  timestamp: number;
  is_favorite: boolean;
  workspace: string;
  collection: string | null;
  tags: string[];
  copy_count: number;
  expires_at: number | null;
}

export interface Workspace {
  name: string;
  is_current: boolean;
}

export interface CollectionSummary {
  name: string;
  count: number;
  samples: string[];
}

export type TriggerKind = "AppIs" | "UrlContains" | "ContentTypeIs" | "WindowTitleContains" | "ContentContains";
export type ActionKind = "AssignCollection" | "AddTag" | "Pin" | "Block" | "AssignWorkspace";

export interface AutomationRule {
  id: string;
  trigger_kind: TriggerKind;
  trigger_value: string;
  action_kind: ActionKind;
  action_value: string;
  enabled: boolean;
}
