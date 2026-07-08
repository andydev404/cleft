import { CopyButton } from "@/components/previews/CopyButton";

export function EmailPreview({ value }: { value: string }) {
  const domain = value.split("@")[1] ?? "";
  return (
    <div className="rounded-[12px] bg-muted p-[18px] text-center">
      <div className="font-mono text-[16px] font-semibold">{value}</div>
      {domain && (
        <div className="mt-[10px] inline-flex gap-1.5">
          <span
            className="rounded-[6px] px-[9px] py-[3px] text-[11px]"
            style={{ background: "color-mix(in srgb, var(--primary) 18%, transparent)", color: "var(--primary)" }}
          >
            @{domain}
          </span>
        </div>
      )}
      <div className="mt-3">
        <CopyButton text={value} />
      </div>
    </div>
  );
}
