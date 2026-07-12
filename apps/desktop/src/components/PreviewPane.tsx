import { useEffect, useRef, useState } from "react";
import { Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ClipMetaRow } from "@/components/ClipMetaRow";
import { ClipPreview } from "@/components/ClipPreview";
import { useClipboardStore } from "@/store/clipboardStore";
import { appIcon } from "@/lib/appIcon";
import { modKey } from "@/lib/platform";
import type { ClipMetadata } from "@/types";

interface PreviewPaneProps {
  selected: ClipMetadata | null;
  content: string | null;
  onToggleFavorite: () => void;
  onDelete: () => void;
  onCopy: () => void;
}

// Edit-before-paste: a scratch copy of the content. Pasting the edited text
// never touches the original clip in history.
function EditPane({ content }: { content: string }) {
  const { cancelEdit, pasteEdited } = useClipboardStore();
  const [draft, setDraft] = useState(content);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.focus();
    el.setSelectionRange(el.value.length, el.value.length);
  }, []);

  return (
    <div className="flex h-full flex-col gap-2.5">
      <textarea
        ref={textareaRef}
        value={draft}
        onChange={(e) => setDraft(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            pasteEdited(draft);
          } else if (e.key === "Escape") {
            e.preventDefault();
            e.stopPropagation();
            cancelEdit();
          }
        }}
        spellCheck={false}
        className="min-h-0 flex-1 resize-none rounded-[10px] border bg-muted p-3 font-mono text-[12.5px] leading-[1.6] outline-none focus:ring-2 focus:ring-ring/40"
        style={{ borderColor: "var(--border-2)" }}
      />
      <div className="flex shrink-0 items-center justify-between text-[11.5px] text-text-tertiary">
        <span>Editing a copy — the original stays in history.</span>
        <span>
          <kbd className="mr-1">⏎</kbd>paste · <kbd className="mx-1">⇧⏎</kbd>newline · <kbd className="mx-1">esc</kbd>cancel
        </span>
      </div>
    </div>
  );
}

export function PreviewPane({ selected, content, onToggleFavorite, onDelete, onCopy }: PreviewPaneProps) {
  const { editing, startEdit } = useClipboardStore();
  const isEditing = editing && selected !== null && content !== null;

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
          onClick={startEdit}
          title={editing ? "Exit edit mode (esc)" : `Edit before paste (${modKey}E)`}
          disabled={!content}
          className="flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-[8px] border transition-colors hover:text-foreground disabled:opacity-40"
          style={{
            borderColor: "var(--border-2)",
            color: editing ? "var(--primary)" : "var(--text-tertiary)",
            background: editing ? "color-mix(in srgb, var(--primary) 18%, transparent)" : "transparent",
          }}
        >
          <Pencil className="h-3.5 w-3.5" />
        </button>
        <button
          onClick={onToggleFavorite}
          title={`Pin (${modKey}D)`}
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
        {isEditing ? (
          <EditPane key={selected.id} content={content} />
        ) : selected ? (
          <ClipPreview contentType={selected.content_type} content={content} />
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            Select a clip to preview it.
          </div>
        )}
      </div>
      {/* EditPane owns paste/cancel (⏎ / esc) while editing. */}
      {!isEditing && (
        <div className="flex shrink-0 gap-2 border-t px-[18px] py-3">
          <Button className="flex-1 gap-[7px] rounded-[9px] py-2 text-[12.5px] font-semibold" onClick={onCopy} disabled={!content}>
            Copy <kbd className="rounded-[4px] bg-white/20 px-[5px] text-[11px]">⏎</kbd>
          </Button>
        </div>
      )}
    </div>
  );
}
