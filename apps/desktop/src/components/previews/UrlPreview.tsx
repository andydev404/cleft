import { appIcon } from "@/lib/appIcon";
import { CopyButton } from "@/components/previews/CopyButton";

// No favicon/title fetch — that would need a network call, which cuts
// against "no network calls by default."
export function UrlPreview({ value }: { value: string }) {
  let hostname = value;
  try {
    hostname = new URL(value).hostname;
  } catch {
    // not a fully-qualified URL; fall back to showing it as-is
  }
  const icon = appIcon(hostname);
  return (
    <div>
      <div className="flex items-center gap-3 rounded-[11px] bg-muted p-[14px]">
        <div
          className="flex h-[38px] w-[38px] shrink-0 items-center justify-center rounded-[9px] text-[16px] font-bold text-white"
          style={{ background: icon.bg }}
        >
          {icon.letter}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-[13.5px] font-semibold leading-[1.35]">{hostname}</div>
          <div className="mt-[3px] truncate font-mono text-[12px] text-primary">{value}</div>
        </div>
      </div>
      <div className="mt-3 flex items-center justify-between text-[12px] text-text-tertiary">
        <span>Full URL captured — title and favicon aren't fetched (no network calls at capture time).</span>
      </div>
      <div className="mt-3">
        <CopyButton text={value} />
      </div>
    </div>
  );
}
