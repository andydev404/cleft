import { Check, Star, Tag, Trash2 } from "lucide-react";
import { TypeBadge } from "@/components/TypeBadge";
import { appIcon } from "@/lib/appIcon";
import type { ClipMetadata } from "@/types";

interface ClipRowProps {
  clip: ClipMetadata;
  isSelected: boolean;
  isChecked: boolean;
  bulkMode: boolean;
  rowRef: (el: HTMLDivElement | null) => void;
  onClick: (e: React.MouseEvent) => void;
  onToggleCheck: () => void;
  onSelect: () => void;
  onCopy: () => void;
  onDelete: () => void;
  onMove: (delta: number) => void;
}

export function ClipRow({
  clip,
  isSelected,
  isChecked,
  bulkMode,
  rowRef,
  onClick,
  onToggleCheck,
  onSelect,
  onCopy,
  onDelete,
  onMove,
}: ClipRowProps) {
  const icon = appIcon(clip.source_app);

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      onDelete();
    } else if (e.key === "Enter") {
      e.preventDefault();
      onCopy();
    } else if (e.key === " ") {
      e.preventDefault();
      onSelect();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      onMove(1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      onMove(-1);
    }
  }

  return (
    <div
      ref={rowRef}
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={handleKeyDown}
      className="group relative mb-0.5 flex w-full cursor-default items-start gap-2.5 rounded-[9px] p-2 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring/50"
      style={{ background: isSelected ? "var(--row-sel)" : "transparent", boxShadow: isSelected ? "inset 0 0 0 1px var(--border)" : "none" }}
    >
      <button
        onClick={(e) => {
          e.stopPropagation();
          onToggleCheck();
        }}
        title={isChecked ? "Deselect" : "Select"}
        className="relative flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-[8px]"
      >
        <span
          className={`absolute inset-0 flex items-center justify-center rounded-[8px] text-[13px] font-bold text-white transition-opacity ${
            isChecked || bulkMode ? "opacity-0" : "opacity-100 group-hover:opacity-0"
          }`}
          style={{ background: icon.bg }}
        >
          {icon.letter}
        </span>
        <span
          className={`absolute inset-0 flex items-center justify-center rounded-[8px] transition-opacity ${
            isChecked || bulkMode ? "opacity-100" : "opacity-0 group-hover:opacity-100"
          }`}
          style={{
            background: isChecked ? "var(--primary)" : "transparent",
            border: isChecked ? "none" : "2px dashed var(--border-2)",
          }}
        >
          {isChecked && <Check className="h-4 w-4 text-white" strokeWidth={3} />}
        </span>
      </button>
      <div className="min-w-0 flex-1">
        <div className="truncate text-[12.5px] font-medium leading-[1.3]">{clip.preview}</div>
        <div className="mt-[3px] flex items-center gap-[5px] overflow-hidden whitespace-nowrap text-[11px] text-text-tertiary">
          <span className="text-muted-foreground">{clip.source_app}</span>
          <span>·</span>
          <span className="truncate">{clip.window_title}</span>
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          title="Delete"
          className="flex h-6 w-6 shrink-0 items-center justify-center rounded-[6px] text-text-tertiary opacity-0 transition-opacity hover:bg-destructive/10 hover:text-destructive group-hover:opacity-100"
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
        <div className="flex flex-col items-end gap-[5px]">
          <div className="flex items-center gap-[5px]">
            {clip.is_favorite && <Star className="h-[9px] w-[9px] fill-primary text-primary" />}
            {clip.tags.length > 0 && (
              <span
                className="flex items-center gap-[3px] rounded-[5px] bg-muted px-1.5 py-0.5 text-[9.5px] font-medium text-text-tertiary"
                title={clip.tags.join(", ")}
              >
                <Tag className="h-[9px] w-[9px]" />
                {clip.tags.length > 1 ? clip.tags.length : clip.tags[0]}
              </span>
            )}
            <TypeBadge type={clip.content_type} />
          </div>
          <span className="text-[10.5px] tabular-nums text-text-tertiary">
            {new Date(clip.timestamp * 1000).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" })}
          </span>
        </div>
      </div>
    </div>
  );
}
