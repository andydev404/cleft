import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { PanelLeft, Search, X } from "lucide-react";
import { BulkActionsBar } from "@/components/BulkActionsBar";
import { ClipRow } from "@/components/ClipRow";
import { PreviewPane } from "@/components/PreviewPane";
import { WorkspaceSwitcher } from "@/components/WorkspaceSwitcher";
import { useClipboardStore } from "@/store/clipboardStore";
import { useConfirmStore } from "@/store/confirmStore";
import { isMac } from "@/lib/platform";
import type { ClipMetadata } from "@/types";

const SEARCH_DEBOUNCE_MS = 50;

export function ClipboardView() {
  const {
    history,
    query,
    searchResults,
    selected,
    selectedContent,
    railFilter,
    collectionFilter,
    setCollectionFilter,
    railOpen,
    toggleRail,
    setQuery,
    setSearchResults,
    selectClip,
    copyClip,
    toggleFavorite,
    bulkSelected,
    toggleBulkSelect,
    rangeBulkSelect,
    selectAll,
    clearBulkSelection,
    deleteClip,
    deleteBulkSelected,
  } = useClipboardStore();
  const searchInputRef = useRef<HTMLInputElement>(null);
  const rowRefs = useRef(new Map<string, HTMLDivElement>());

  useEffect(() => {
    searchInputRef.current?.focus();
    // The palette window is pre-rendered and just shown/hidden after the
    // first launch, not remounted — so this effect's initial run only
    // covers that very first appearance. Without also refocusing on every
    // subsequent "palette-shown" (fired by the global shortcut toggling
    // visibility), the webview falls back to whatever was focused before
    // hiding, or the first focusable element in the header (the sidebar
    // collapse button) if nothing was — landing focus somewhere that was
    // never intended to hold it.
    const unlisten = listen("palette-shown", () => {
      searchInputRef.current?.focus();
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed) {
      setSearchResults([]);
      return;
    }
    const debounce = setTimeout(() => {
      invoke<ClipMetadata[]>("search_clips", { query: trimmed }).then(setSearchResults);
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(debounce);
  }, [query, setSearchResults]);

  const base = query.trim() ? searchResults : history;
  const visible = collectionFilter
    ? base.filter((c) => c.collection === collectionFilter)
    : railFilter === "favorites"
      ? base.filter((c) => c.is_favorite)
      : base;
  const visibleIds = visible.map((c) => c.id);
  const allVisibleSelected = visible.length > 0 && visible.every((c) => bulkSelected.has(c.id));

  // Moves the highlight and returns focus to the search input (rows are
  // focusable, so a click would otherwise strand DOM focus on a row and
  // typing would stop working). Clamped at the ends, not wrapped.
  function moveSelection(delta: number) {
    if (visible.length === 0) return;
    const currentIndex = selected ? visible.findIndex((c) => c.id === selected.id) : -1;
    const nextIndex = Math.min(Math.max(currentIndex + delta, 0), visible.length - 1);
    const next = visible[nextIndex];
    selectClip(next);
    rowRefs.current.get(next.id)?.scrollIntoView({ block: "nearest" });
    searchInputRef.current?.focus();
  }

  function handleSearchKeyDown(e: React.KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      moveSelection(1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      moveSelection(-1);
    } else if (e.key === "Enter") {
      // No selection yet → copy the top result, Spotlight-style.
      const target = selected ?? visible[0];
      if (target) copyClip(target);
    } else if ((isMac ? e.metaKey : e.ctrlKey) && /^[1-9]$/.test(e.key)) {
      e.preventDefault();
      const target = visible[Number(e.key) - 1];
      if (target) copyClip(target);
    }
  }

  function handleRowClick(e: React.MouseEvent, clip: ClipMetadata) {
    if (e.metaKey || e.ctrlKey) {
      toggleBulkSelect(clip.id);
      return;
    }
    if (e.shiftKey) {
      rangeBulkSelect(clip.id, visibleIds);
      return;
    }
    selectClip(clip);
  }

  function confirmDeleteClip(clip: ClipMetadata) {
    const snippet = clip.preview.length > 60 ? `${clip.preview.slice(0, 60)}…` : clip.preview;
    useConfirmStore.getState().show({
      title: "Delete this clip?",
      description: `"${snippet}" will be permanently deleted. This can't be undone.`,
      onConfirm: () => deleteClip(clip.id),
    });
  }

  function confirmDeleteBulk() {
    const count = bulkSelected.size;
    useConfirmStore.getState().show({
      title: `Delete ${count} clip${count === 1 ? "" : "s"}?`,
      description: "This can't be undone.",
      confirmLabel: `Delete ${count} clip${count === 1 ? "" : "s"}`,
      onConfirm: deleteBulkSelected,
    });
  }

  const emptyMessage = query.trim() ? (
    <>
      No clips match <span className="font-semibold text-foreground">"{query}"</span>
    </>
  ) : collectionFilter ? (
    "No clips in this collection yet."
  ) : (
    "No favorites yet."
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex h-[54px] shrink-0 items-center gap-3 border-b px-[18px]">
        <button
          onClick={toggleRail}
          title={railOpen ? "Hide library" : "Show library"}
          className="flex h-7 w-7 shrink-0 items-center justify-center rounded-[7px] transition-colors"
          style={{ color: railOpen ? "var(--muted-foreground)" : "var(--text-tertiary)", background: railOpen ? "var(--muted)" : "transparent" }}
        >
          <PanelLeft className="h-4 w-4" strokeWidth={2} />
        </button>
        <Search className="h-[17px] w-[17px] shrink-0 text-text-tertiary" strokeWidth={2.2} />
        <input
          ref={searchInputRef}
          value={query}
          onChange={(e) => setQuery(e.currentTarget.value)}
          onKeyDown={handleSearchKeyDown}
          placeholder="Search everything you've copied…"
          className="min-w-0 flex-1 bg-transparent text-[15px] tracking-[-.01em] outline-none"
        />
        <WorkspaceSwitcher />
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="flex w-[320px] shrink-0 flex-col overflow-hidden border-r">
          {collectionFilter && (
            <div className="m-[7px] mb-0 flex shrink-0 items-center justify-between rounded-[9px] bg-accent px-2.5 py-[7px]">
              <span className="truncate text-[12px] font-semibold">{collectionFilter}</span>
              <button
                onClick={() => setCollectionFilter(null)}
                title="Clear filter"
                className="flex h-6 w-6 shrink-0 items-center justify-center rounded-[6px] text-muted-foreground hover:bg-muted hover:text-foreground"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          )}
          {bulkSelected.size > 0 && (
            <BulkActionsBar
              count={bulkSelected.size}
              allSelected={allVisibleSelected}
              onSelectAll={() => selectAll(visibleIds)}
              onClear={clearBulkSelection}
              onDelete={confirmDeleteBulk}
            />
          )}
          <div className="min-h-0 flex-1 overflow-y-auto p-[7px]">
            {visible.length === 0 ? (
              <div className="p-10 text-center text-[13px] text-text-tertiary">{emptyMessage}</div>
            ) : (
              visible.map((clip) => (
                <ClipRow
                  key={clip.id}
                  clip={clip}
                  isSelected={selected?.id === clip.id}
                  isChecked={bulkSelected.has(clip.id)}
                  bulkMode={bulkSelected.size > 0}
                  rowRef={(el) => {
                    if (el) rowRefs.current.set(clip.id, el);
                    else rowRefs.current.delete(clip.id);
                  }}
                  onClick={(e) => handleRowClick(e, clip)}
                  onToggleCheck={() => toggleBulkSelect(clip.id)}
                  onSelect={() => selectClip(clip)}
                  onCopy={() => copyClip(clip)}
                  onDelete={() => confirmDeleteClip(clip)}
                  onMove={moveSelection}
                />
              ))
            )}
          </div>
        </div>

        <PreviewPane
          selected={selected}
          content={selectedContent}
          onToggleFavorite={() => selected && toggleFavorite(selected.id)}
          onDelete={() => selected && confirmDeleteClip(selected)}
          onCopy={() => selected && copyClip(selected)}
        />
      </div>
    </div>
  );
}
