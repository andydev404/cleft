import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ClipMetaRow } from "@/components/ClipMetaRow";
import { ClipPreview } from "@/components/ClipPreview";
import { appIcon } from "@/lib/appIcon";
import type { ClipMetadata } from "@/types";

interface PreviewPaneProps {
  selected: ClipMetadata | null;
  content: string | null;
  onToggleFavorite: () => void;
  onDelete: () => void;
  onCopy: () => void;
}

export function PreviewPane({ selected, content, onToggleFavorite, onDelete, onCopy }: PreviewPaneProps) {
  return (
    <div className="flex min-w-0 flex-1 flex-col">
      {/* The header shell always renders (only inner content swaps) —
          remounting the whole subtree caused a WebKit compositing glitch. */}
      <div className="flex shrink-0 items-center gap-2.5 border-b px-[18px] pb-3 pt-3.5" style={{ visibility: selected ? "visible" : "hidden" }}>
        <div
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-[8px] text-[13px] font-bold text-white"
          style={{ background: selected ? appIcon(selected.source_app).bg : "var(--muted)" }}
        >
          {selected ? appIcon(selected.source_app).letter : ""}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-[13px] font-semibold">
            {selected ? selected.window_title || selected.preview : ""}
          </div>
          <div className="mt-px text-[11.5px] text-text-tertiary">
            {selected &&
              `${selected.source_app} · ${new Date(selected.timestamp * 1000).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" })}`}
          </div>
        </div>
        <button
          onClick={onToggleFavorite}
          title="Pin (⌘D)"
          className="flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-[8px] border text-[14px] transition-colors"
          style={{
            borderColor: "var(--border-2)",
            color: selected?.is_favorite ? "var(--primary)" : "var(--text-tertiary)",
            background: selected?.is_favorite ? "color-mix(in srgb, var(--primary) 18%, transparent)" : "transparent",
          }}
        >
          ★
        </button>
        <button
          onClick={onDelete}
          title="Delete (⌫)"
          className="flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-[8px] border text-text-tertiary transition-colors hover:border-destructive/40 hover:text-destructive"
          style={{ borderColor: "var(--border-2)" }}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
      </div>
      {selected && <ClipMetaRow clip={selected} />}
      <div className="flex-1 overflow-y-auto p-[18px]">
        {selected ? (
          <ClipPreview contentType={selected.content_type} content={content} />
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            Select a clip to preview it.
          </div>
        )}
      </div>
      <div className="flex shrink-0 gap-2 border-t px-[18px] py-3">
        <Button className="flex-1 gap-[7px] rounded-[9px] py-2 text-[12.5px] font-semibold" onClick={onCopy} disabled={!content}>
          Copy <kbd className="rounded-[4px] bg-white/20 px-[5px] text-[11px]">⏎</kbd>
        </Button>
      </div>
    </div>
  );
}
