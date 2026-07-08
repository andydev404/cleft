import { useState } from "react";
import { ACTION_LABELS, TRIGGER_LABELS, VALUELESS_ACTIONS } from "@/lib/rules";
import { useToastStore } from "@/store/toastStore";
import type { ActionKind, TriggerKind } from "@/types";

export interface RuleFormValues {
  trigger_kind: TriggerKind;
  trigger_value: string;
  action_kind: ActionKind;
  action_value: string;
}

// Shared by both "+ New Rule" and the per-row "Edit" button — same fields,
// just a different initial state and submit target.
export function RuleForm({
  initial,
  submitLabel,
  onSubmit,
  onCancel,
}: {
  initial?: RuleFormValues;
  submitLabel: string;
  onSubmit: (v: RuleFormValues) => Promise<string | null>;
  onCancel: () => void;
}) {
  const [triggerKind, setTriggerKind] = useState<TriggerKind>(initial?.trigger_kind ?? "AppIs");
  const [triggerValue, setTriggerValue] = useState(initial?.trigger_value ?? "");
  const [actionKind, setActionKind] = useState<ActionKind>(initial?.action_kind ?? "AddTag");
  const [actionValue, setActionValue] = useState(initial?.action_value ?? "");

  const needsActionValue = !VALUELESS_ACTIONS.includes(actionKind);

  async function submit() {
    if (!triggerValue.trim()) {
      useToastStore.getState().show("Enter a value for the trigger first");
      return;
    }
    const err = await onSubmit({
      trigger_kind: triggerKind,
      trigger_value: triggerValue.trim(),
      action_kind: actionKind,
      action_value: needsActionValue ? actionValue.trim() : "",
    });
    if (err) {
      useToastStore.getState().show(err);
      return;
    }
    onCancel();
  }

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
      className="mb-2.5 flex flex-col gap-2.5 rounded-[12px] border bg-card px-4 py-3.5 text-[12.5px]"
    >
      <div className="flex flex-wrap items-center gap-[6px]">
        <span className="font-semibold text-text-tertiary">IF</span>
        <select
          value={triggerKind}
          onChange={(e) => setTriggerKind(e.currentTarget.value as TriggerKind)}
          className="rounded-[6px] border bg-muted px-2 py-[3px] text-[11.5px]"
          style={{ borderColor: "var(--border-2)" }}
        >
          {(Object.keys(TRIGGER_LABELS) as TriggerKind[]).map((k) => (
            <option key={k} value={k}>
              {TRIGGER_LABELS[k]}
            </option>
          ))}
        </select>
        <input
          autoFocus
          value={triggerValue}
          onChange={(e) => setTriggerValue(e.currentTarget.value)}
          placeholder={triggerKind === "ContentTypeIs" ? "e.g. Color, SQL, JSON" : "value"}
          className="min-w-0 flex-1 rounded-[6px] border bg-muted px-2 py-[3px] font-mono text-[11.5px] outline-none"
          style={{ borderColor: "var(--border-2)" }}
        />
      </div>
      <div className="flex flex-wrap items-center gap-[6px]">
        <span className="font-semibold text-primary">THEN</span>
        <select
          value={actionKind}
          onChange={(e) => setActionKind(e.currentTarget.value as ActionKind)}
          className="rounded-[6px] border bg-muted px-2 py-[3px] text-[11.5px]"
          style={{ borderColor: "var(--border-2)" }}
        >
          {(Object.keys(ACTION_LABELS) as ActionKind[]).map((k) => (
            <option key={k} value={k}>
              {ACTION_LABELS[k]}
            </option>
          ))}
        </select>
        {needsActionValue && (
          <input
            value={actionValue}
            onChange={(e) => setActionValue(e.currentTarget.value)}
            placeholder="value"
            className="min-w-0 flex-1 rounded-[6px] border bg-muted px-2 py-[3px] font-mono text-[11.5px] outline-none"
            style={{ borderColor: "var(--border-2)" }}
          />
        )}
      </div>
      <div className="mt-1 flex justify-end gap-2">
        <button type="button" onClick={onCancel} className="rounded-[6px] px-2.5 py-[5px] text-[11.5px] font-semibold text-muted-foreground hover:bg-muted">
          Cancel
        </button>
        <button
          type="submit"
          className="rounded-[6px] bg-primary px-2.5 py-[5px] text-[11.5px] font-semibold text-primary-foreground hover:brightness-[1.08]"
        >
          {submitLabel}
        </button>
      </div>
    </form>
  );
}
