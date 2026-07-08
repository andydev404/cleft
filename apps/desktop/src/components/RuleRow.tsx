import { Pencil, Trash2 } from "lucide-react";
import { formatAction, formatTrigger } from "@/lib/rules";
import type { AutomationRule } from "@/types";

interface RuleRowProps {
  rule: AutomationRule;
  onToggle: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function RuleRow({ rule, onToggle, onEdit, onDelete }: RuleRowProps) {
  return (
    <div
      className="group flex items-start gap-[13px] rounded-[12px] border bg-card px-4 py-3.5"
      style={{ opacity: rule.enabled ? 1 : 0.55 }}
    >
      <button
        onClick={onToggle}
        className="relative mt-0.5 h-5 w-[34px] shrink-0 rounded-full border transition-colors"
        style={{
          background: rule.enabled ? "var(--primary)" : "var(--muted)",
          borderColor: rule.enabled ? "var(--primary)" : "var(--border-2)",
        }}
      >
        <span
          className="absolute top-[1px] h-4 w-4 rounded-full bg-white shadow transition-all"
          style={{ left: rule.enabled ? "15px" : "1px" }}
        />
      </button>
      <div className="min-w-0 flex-1 text-[12.5px]">
        <div className="flex flex-wrap items-center gap-[6px]">
          <span className="font-semibold text-text-tertiary">IF</span>
          <span className="rounded-[6px] border bg-muted px-2 py-[3px] font-mono text-[11.5px]">{formatTrigger(rule)}</span>
        </div>
        <div className="mt-[7px] flex flex-wrap items-center gap-[6px]">
          <span className="font-semibold text-primary">THEN</span>
          <span
            className="rounded-[6px] px-2 py-[3px] text-[11.5px] font-medium"
            style={{ background: "color-mix(in srgb, var(--primary) 18%, transparent)", color: "var(--primary)" }}
          >
            {formatAction(rule)}
          </span>
        </div>
      </div>
      <button
        onClick={onEdit}
        title="Edit rule"
        className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-[6px] text-text-tertiary opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover:opacity-100"
      >
        <Pencil className="h-3.5 w-3.5" />
      </button>
      <button
        onClick={onDelete}
        title="Delete rule"
        className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-[6px] text-text-tertiary opacity-0 transition-opacity hover:bg-destructive/10 hover:text-destructive group-hover:opacity-100"
      >
        <Trash2 className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
