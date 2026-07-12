import { Clipboard } from "lucide-react";
import { useRouterState } from "@tanstack/react-router";
import { modKey } from "@/lib/platform";
import { useClipboardStore } from "@/store/clipboardStore";

export function FooterBar() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const { history, searchResults, query, selected, copyClip, startEdit, editing } = useClipboardStore();

  const resultCount = pathname === "/" && query.trim() ? searchResults.length : history.length;
  const resultLabel = pathname === "/" ? `${resultCount} ${resultCount === 1 ? "result" : "results"}` : `${history.length} clips`;

  return (
    <div className="flex h-[42px] shrink-0 items-center justify-between border-t bg-sidebar pl-3.5 pr-3 text-[11.5px] text-text-tertiary">
      <div className="flex items-center gap-2.5">
        <Clipboard className="h-[15px] w-[15px] text-primary" strokeWidth={2} />
        <span className="font-semibold tracking-[-.01em] text-muted-foreground">Cleft</span>
        <span className="font-mono text-[10.5px] text-text-tertiary">
          · {resultLabel} · FTS5
        </span>
      </div>
      <div className="flex items-center gap-1">
        <div className="flex items-center gap-1.5 rounded-[7px] px-2.5 py-[5px] font-medium text-muted-foreground">
          Navigate
          <kbd className="rounded-[5px] border bg-muted px-[5px] py-px font-mono text-[10.5px] text-muted-foreground">
            ↑↓
          </kbd>
        </div>
        <div className="h-4 w-px bg-border" />
        <button
          onClick={() => selected && startEdit()}
          disabled={!selected}
          title={editing ? "Exit edit mode (esc)" : "Edit a copy before pasting — the original stays in history"}
          className="flex items-center gap-1.5 rounded-[7px] px-2.5 py-[5px] font-medium transition-colors hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
          style={{ color: editing ? "var(--primary)" : "var(--muted-foreground)" }}
        >
          Edit
          <kbd className="rounded-[5px] border bg-muted px-[5px] py-px font-mono text-[10.5px] text-muted-foreground">
            {modKey}E
          </kbd>
        </button>
        <div className="h-4 w-px bg-border" />
        <button
          onClick={() => selected && copyClip(selected)}
          disabled={!selected}
          className="flex items-center gap-1.5 rounded-[7px] px-2.5 py-[5px] font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
        >
          Copy
          <kbd className="rounded-[5px] border bg-muted px-[5px] py-px font-mono text-[10.5px] text-muted-foreground">
            ⏎
          </kbd>
        </button>
      </div>
    </div>
  );
}
