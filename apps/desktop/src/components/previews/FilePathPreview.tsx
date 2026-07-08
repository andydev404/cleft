import { CopyButton } from "@/components/previews/CopyButton";

export function FilePathPreview({ value }: { value: string }) {
  const segments = value.split("/").filter(Boolean);
  const fileName = segments.pop();
  return (
    <div>
      <div className="flex flex-wrap items-center gap-[5px] rounded-[11px] bg-muted p-[14px] font-mono text-[12.5px]">
        {segments.map((seg, i) => (
          <span key={i}>
            <span className="text-muted-foreground">{seg}</span>
            <span className="text-text-tertiary">/</span>
          </span>
        ))}
        <span className="font-semibold text-primary">{fileName}</span>
      </div>
      <div className="mt-3">
        <CopyButton text={value} />
      </div>
    </div>
  );
}
