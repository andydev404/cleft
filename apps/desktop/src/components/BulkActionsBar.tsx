import { ListChecks, Trash2, X } from "lucide-react";

interface BulkActionsBarProps {
  count: number;
  allSelected: boolean;
  onSelectAll: () => void;
  onClear: () => void;
  onDelete: () => void;
}

export function BulkActionsBar({ count, allSelected, onSelectAll, onClear, onDelete }: BulkActionsBarProps) {
  return (
    <div className="m-[7px] mb-0 flex shrink-0 items-center justify-between rounded-[9px] bg-accent px-2.5 py-[7px]">
      <span className="text-[12px] font-semibold">{count} selected</span>
      <div className="flex items-center gap-1">
        {!allSelected && (
          <button
            onClick={onSelectAll}
            className="flex items-center gap-1 rounded-[6px] px-2 py-1 text-[11.5px] font-semibold text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            <ListChecks className="h-3.5 w-3.5" />
            Select All
          </button>
        )}
        <button
          onClick={onClear}
          title="Cancel"
          className="flex h-6 w-6 items-center justify-center rounded-[6px] text-muted-foreground hover:bg-muted hover:text-foreground"
        >
          <X className="h-3.5 w-3.5" />
        </button>
        <button
          onClick={onDelete}
          className="flex items-center gap-1 rounded-[6px] px-2 py-1 text-[11.5px] font-semibold text-destructive hover:bg-destructive/10"
        >
          <Trash2 className="h-3 w-3" />
          Delete
        </button>
      </div>
    </div>
  );
}
