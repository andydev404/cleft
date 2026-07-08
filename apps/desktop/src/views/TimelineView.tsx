import { ViewHeader } from "@/components/ViewHeader";
import { useClipboardStore } from "@/store/clipboardStore";
import { TypeBadge } from "@/components/TypeBadge";
import { appIcon, relativeTime } from "@/lib/appIcon";

// Blocked-app rows are deliberately absent — blocked content is never
// logged anywhere, so there's no data to show for it.
export function TimelineView() {
  const history = useClipboardStore((s) => s.history);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ViewHeader title="Clipboard Timeline" subtitle={`Today · ${history.length} clips`} />
      <div className="flex-1 overflow-y-auto px-[22px] pb-8 pt-[18px]">
        {history.map((clip, i) => {
          const icon = appIcon(clip.source_app);
          return (
            <div key={clip.id} className="flex gap-4">
              <div className="w-[62px] shrink-0 pt-[13px] text-right text-[12px] tabular-nums text-text-tertiary">
                {relativeTime(clip.timestamp)}
              </div>
              <div className="flex shrink-0 flex-col items-center">
                <div className="mt-[14px] h-[11px] w-[11px] rounded-full ring-4 ring-background" style={{ background: icon.bg }} />
                {i < history.length - 1 && <div className="w-[2px] flex-1 bg-border" />}
              </div>
              <div className="min-w-0 flex-1 pb-2.5">
                <div className="flex items-center gap-3.5 rounded-[11px] border bg-card px-3.5 py-2.5">
                  <div
                    className="flex h-[26px] w-[26px] shrink-0 items-center justify-center rounded-[7px] text-[12px] font-bold text-white"
                    style={{ background: icon.bg }}
                  >
                    {icon.letter}
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-[12.5px] font-medium">{clip.preview}</div>
                    <div className="mt-[3px] truncate text-[11.5px] text-text-tertiary">
                      {clip.window_title || clip.source_app}
                    </div>
                  </div>
                  <TypeBadge type={clip.content_type} />
                </div>
              </div>
            </div>
          );
        })}
        {history.length === 0 && (
          <div className="py-10 text-center text-sm text-muted-foreground">Nothing captured yet.</div>
        )}
      </div>
    </div>
  );
}
