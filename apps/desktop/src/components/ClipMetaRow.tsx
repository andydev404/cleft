import { useState } from "react";
import { Timer, X } from "lucide-react";
import { timeLeft } from "@/lib/age";
import { useClipboardStore } from "@/store/clipboardStore";
import type { ClipMetadata } from "@/types";

const EXPIRY_CHOICES: { label: string; seconds: number | null }[] = [
  { label: "Never", seconds: null },
  { label: "1 hour", seconds: 3600 },
  { label: "24 hours", seconds: 24 * 3600 },
  { label: "7 days", seconds: 7 * 24 * 3600 },
];

// Manual collection/tag assignment; automation rules (clipboard.rs) are
// the other path that writes the same fields.
export function ClipMetaRow({ clip }: { clip: ClipMetadata }) {
  const { assignCollection, addTag, removeTag, setExpiry } = useClipboardStore();
  const [editingCollection, setEditingCollection] = useState(false);
  const [collectionDraft, setCollectionDraft] = useState(clip.collection ?? "");
  const [tagDraft, setTagDraft] = useState("");

  return (
    // Outer row never wraps — the expiry control stays pinned to the right
    // on the same line; only the collection/tags cluster wraps internally.
    <div className="flex shrink-0 items-start justify-between gap-3 border-b px-[18px] py-2 text-[11.5px]">
      <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
      <span className="font-semibold text-text-tertiary">Collection:</span>
      {editingCollection ? (
        <input
          autoFocus
          value={collectionDraft}
          onChange={(e) => setCollectionDraft(e.currentTarget.value)}
          onBlur={() => setEditingCollection(false)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              assignCollection(clip.id, collectionDraft.trim() || null);
              setEditingCollection(false);
            } else if (e.key === "Escape") {
              setCollectionDraft(clip.collection ?? "");
              setEditingCollection(false);
            }
          }}
          placeholder="Unassigned"
          className="min-w-0 flex-1 rounded-[6px] border bg-muted px-2 py-[3px] outline-none"
          style={{ borderColor: "var(--border-2)" }}
        />
      ) : (
        <button
          onClick={() => {
            setCollectionDraft(clip.collection ?? "");
            setEditingCollection(true);
          }}
          className="truncate rounded-[6px] px-2 py-[3px] hover:bg-muted"
          style={{ color: clip.collection ? "var(--primary)" : "var(--text-tertiary)" }}
        >
          {clip.collection ?? "Unassigned"}
        </button>
      )}

      <span className="ml-2 font-semibold text-text-tertiary">Tags:</span>
      {clip.tags.map((tag) => (
        <span key={tag} className="flex items-center gap-1 rounded-[6px] bg-muted px-2 py-[3px] font-mono">
          {tag}
          <button onClick={() => removeTag(clip.id, tag)} className="text-text-tertiary hover:text-destructive">
            <X className="h-2.5 w-2.5" />
          </button>
        </span>
      ))}
      <input
        value={tagDraft}
        onChange={(e) => setTagDraft(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && tagDraft.trim()) {
            addTag(clip.id, tagDraft.trim());
            setTagDraft("");
          }
        }}
        placeholder="+ add tag"
        className="w-[80px] rounded-[6px] bg-transparent px-1 py-[3px] text-text-tertiary outline-none placeholder:text-text-tertiary"
      />
      </div>

      {/* Self-destruct timer for temporary clips (one-time credentials,
          tokens) — swept by the Rust monitor once it passes. */}
      <span className="flex shrink-0 items-center gap-1 pt-[3px]">
        <Timer
          className="h-3 w-3"
          style={{ color: clip.expires_at ? "var(--primary)" : "var(--text-tertiary)" }}
        />
        {clip.expires_at && <span style={{ color: "var(--primary)" }}>{timeLeft(clip.expires_at)}</span>}
        <select
          value=""
          onChange={(e) => {
            const choice = EXPIRY_CHOICES[Number(e.currentTarget.value)];
            setExpiry(clip.id, choice.seconds === null ? null : Math.floor(Date.now() / 1000) + choice.seconds);
          }}
          title="Auto-delete this clip after…"
          className="w-[18px] cursor-pointer bg-transparent text-text-tertiary outline-none"
        >
          <option value="" disabled hidden />
          {EXPIRY_CHOICES.map((choice, i) => (
            <option key={choice.label} value={i}>
              {choice.label}
            </option>
          ))}
        </select>
      </span>
    </div>
  );
}
