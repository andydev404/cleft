import { useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { ViewHeader } from "@/components/ViewHeader";
import { appIcon } from "@/lib/appIcon";
import { useClipboardStore } from "@/store/clipboardStore";
import type { CollectionSummary } from "@/types";

export function CollectionsView() {
  const [collections, setCollections] = useState<CollectionSummary[]>([]);
  const setCollectionFilter = useClipboardStore((s) => s.setCollectionFilter);
  const navigate = useNavigate();

  useEffect(() => {
    invoke<CollectionSummary[]>("list_collections").then(setCollections);
  }, []);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ViewHeader title="Collections" subtitle={`${collections.length} collection${collections.length === 1 ? "" : "s"}`} />
      {collections.length === 0 ? (
        <div className="flex flex-1 items-center justify-center px-8 text-center text-[13px] text-text-tertiary">
          No collections yet. Assign a clip to one from its preview, or set up an automation rule to do it for you.
        </div>
      ) : (
        <div className="grid flex-1 auto-rows-min grid-cols-2 gap-3.5 overflow-y-auto px-[22px] pb-8 pt-5">
          {collections.map((c) => {
            const icon = appIcon(c.name);
            return (
              <button
                key={c.name}
                onClick={() => {
                  setCollectionFilter(c.name);
                  navigate({ to: "/" });
                }}
                className="cursor-pointer rounded-[12px] border bg-card p-4 text-left transition-colors hover:border-[var(--border-2)]"
              >
                <div className="flex items-center gap-2.5">
                  <div
                    className="flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-[8px] text-[14px] font-bold text-white"
                    style={{ background: icon.bg }}
                  >
                    {icon.letter}
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-[13.5px] font-semibold">{c.name}</div>
                    <div className="text-[11.5px] text-text-tertiary">
                      {c.count} clip{c.count === 1 ? "" : "s"}
                    </div>
                  </div>
                </div>
                <div className="mt-3 flex flex-col gap-1.5">
                  {c.samples.map((s, i) => (
                    <div key={i} className="truncate rounded-[6px] bg-muted px-[9px] py-[6px] font-mono text-[11.5px] text-muted-foreground">
                      {s}
                    </div>
                  ))}
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
