import { useEffect, useState } from "react";
import { Check, Trash2, X } from "lucide-react";
import { useConfirmStore } from "@/store/confirmStore";
import { useToastStore } from "@/store/toastStore";
import { useWorkspaceStore } from "@/store/workspaceStore";

const NEW_WORKSPACE = "__new__";
// Mirrors db::MAX_WORKSPACES (src-tauri/src/db.rs) — no shared-constant
// mechanism across the IPC boundary, so this just disables the option
// proactively; the backend is still the actual source of truth and would
// reject a create past this even if the two ever drifted.
const MAX_WORKSPACES = 5;

// A native <select> rather than a custom dropdown — no popover component
// exists in this codebase yet, and a native select renders with the real
// macOS dropdown chrome, which fits "feels native" better than a bespoke
// one anyway.
export function WorkspaceSwitcher() {
  const { workspaces, load, switchTo, create, remove } = useWorkspaceStore();
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    load();
  }, [load]);

  const current = workspaces.find((w) => w.is_current)?.name ?? "Personal";
  const atLimit = workspaces.length >= MAX_WORKSPACES;
  const onlyOneLeft = workspaces.length <= 1;

  function confirmDelete() {
    useConfirmStore.getState().show({
      title: `Delete "${current}"?`,
      description: `This permanently deletes the "${current}" workspace and every clip in it. This can't be undone.`,
      confirmLabel: "Delete Workspace",
      onConfirm: async () => {
        const err = await remove(current);
        if (err) useToastStore.getState().show(err);
      },
    });
  }

  function cancel() {
    setCreating(false);
    setName("");
    setError(null);
  }

  async function submit(trimmed: string) {
    if (workspaces.some((w) => w.name.toLowerCase() === trimmed.toLowerCase())) {
      setError(`A workspace named "${trimmed}" already exists`);
      return;
    }
    const err = await create(trimmed);
    if (err) {
      setError(err);
      return;
    }
    await switchTo(trimmed);
    cancel();
  }

  if (creating) {
    return (
      <div className="relative flex shrink-0 items-center">
        <form
          className="flex items-center gap-1"
          onSubmit={(e) => {
            e.preventDefault();
            const trimmed = name.trim();
            if (trimmed) submit(trimmed);
          }}
        >
          <input
            autoFocus
            value={name}
            onChange={(e) => {
              setName(e.currentTarget.value);
              setError(null);
            }}
            onKeyDown={(e) => {
              if (e.key === "Escape") cancel();
            }}
            placeholder="Workspace name"
            maxLength={40}
            className="w-[140px] rounded-[8px] border bg-muted px-2.5 py-[5px] text-[12px] font-semibold outline-none"
            style={{ borderColor: error ? "var(--destructive)" : "var(--border-2)" }}
          />
          <button
            type="submit"
            title="Create workspace"
            disabled={!name.trim()}
            className="flex h-[26px] w-[26px] shrink-0 items-center justify-center rounded-[7px] bg-primary text-primary-foreground disabled:opacity-40"
          >
            <Check className="h-3.5 w-3.5" strokeWidth={3} />
          </button>
          <button
            type="button"
            onClick={cancel}
            title="Cancel"
            className="flex h-[26px] w-[26px] shrink-0 items-center justify-center rounded-[7px] text-muted-foreground hover:bg-muted"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </form>
        {error && (
          <div
            className="absolute right-0 top-[calc(100%+6px)] z-10 max-w-[220px] rounded-[8px] border px-2.5 py-[6px] text-[11px] font-medium text-destructive shadow-sm"
            style={{ background: "var(--popover)", borderColor: "var(--border-2)" }}
          >
            {error}
          </div>
        )}
      </div>
    );
  }

  return (
    <div
      className="flex shrink-0 items-center gap-1.5 rounded-[8px] border bg-muted pl-2.5 pr-1.5 py-[5px] text-[12px] font-semibold text-muted-foreground"
      style={{ borderColor: "var(--border-2)" }}
    >
      <div className="h-[7px] w-[7px] shrink-0 rounded-full bg-primary" />
      <select
        value={current}
        onChange={(e) => {
          if (e.currentTarget.value === NEW_WORKSPACE) {
            setCreating(true);
          } else {
            switchTo(e.currentTarget.value);
          }
        }}
        className="bg-transparent outline-none"
      >
        {workspaces.map((w) => (
          <option key={w.name} value={w.name}>
            {w.name}
          </option>
        ))}
        <option value={NEW_WORKSPACE} disabled={atLimit}>
          {atLimit ? `Workspace limit reached (${MAX_WORKSPACES})` : "+ New Workspace…"}
        </option>
      </select>
      <button
        onClick={confirmDelete}
        disabled={onlyOneLeft}
        title={onlyOneLeft ? "Can't delete your only workspace" : `Delete "${current}"`}
        className="flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded-[6px] text-text-tertiary transition-colors hover:bg-destructive/10 hover:text-destructive disabled:pointer-events-none disabled:opacity-30"
      >
        <Trash2 className="h-3 w-3" />
      </button>
    </div>
  );
}
