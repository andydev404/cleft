export function CodePreview({ value, label }: { value: string; label: string }) {
  const lines = value.split("\n").length;
  return (
    <div>
      <div className="mb-[10px] flex items-center justify-between">
        <span className="font-mono text-[11px] font-semibold uppercase tracking-[.04em] text-muted-foreground">
          {label}
        </span>
        <span className="text-[11px] text-text-tertiary">
          {lines} line{lines === 1 ? "" : "s"}
        </span>
      </div>
      <pre className="whitespace-pre-wrap break-words font-mono text-[12.5px] leading-[1.65]">{value}</pre>
    </div>
  );
}
