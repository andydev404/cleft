import { useEffect, useState } from "react";
import { ViewHeader } from "@/components/ViewHeader";
import { RuleForm } from "@/components/RuleForm";
import { RuleRow } from "@/components/RuleRow";
import { useAutomationStore } from "@/store/automationStore";
import { useConfirmStore } from "@/store/confirmStore";
import { useToastStore } from "@/store/toastStore";

export function AutomationView() {
  const { rules, load, setEnabled, remove, create, update } = useAutomationStore();
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const activeCount = rules.filter((r) => r.enabled).length;

  useEffect(() => {
    load();
  }, [load]);

  function confirmDelete(id: string) {
    useConfirmStore.getState().show({
      title: "Delete this rule?",
      description: "It'll stop running immediately. This can't be undone.",
      confirmLabel: "Delete",
      onConfirm: () => remove(id),
    });
  }

  async function toggleRule(id: string, enabled: boolean) {
    const err = await setEnabled(id, enabled);
    if (err) useToastStore.getState().show(err);
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ViewHeader title="Automation Rules" subtitle={`${activeCount} of 10 active`} />
      <div className="flex-1 overflow-y-auto px-[22px] pb-8 pt-5">
        <div className="mb-3.5 flex items-center justify-between">
          <div className="text-[12.5px] text-muted-foreground">
            Rules run in the Rust core at capture time — before a clip ever reaches the UI.
          </div>
          {!creating && (
            <button
              onClick={() => {
                setEditingId(null);
                setCreating(true);
              }}
              className="cursor-pointer rounded-[8px] bg-primary px-[13px] py-[7px] text-[12px] font-semibold text-primary-foreground transition-[filter] hover:brightness-[1.08]"
            >
              + New Rule
            </button>
          )}
        </div>

        {creating && (
          <RuleForm
            submitLabel="Create Rule"
            onSubmit={(v) => create(v.trigger_kind, v.trigger_value, v.action_kind, v.action_value)}
            onCancel={() => setCreating(false)}
          />
        )}

        <div className="flex flex-col gap-2.5">
          {rules.map((rule) =>
            editingId === rule.id ? (
              <RuleForm
                key={rule.id}
                initial={rule}
                submitLabel="Save Changes"
                onSubmit={(v) => update(rule.id, v.trigger_kind, v.trigger_value, v.action_kind, v.action_value)}
                onCancel={() => setEditingId(null)}
              />
            ) : (
              <RuleRow
                key={rule.id}
                rule={rule}
                onToggle={() => toggleRule(rule.id, !rule.enabled)}
                onEdit={() => {
                  setCreating(false);
                  setEditingId(rule.id);
                }}
                onDelete={() => confirmDelete(rule.id)}
              />
            )
          )}
        </div>
      </div>
    </div>
  );
}
