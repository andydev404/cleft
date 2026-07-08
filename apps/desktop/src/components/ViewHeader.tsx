// Exact 46px content header from the design's Main Window ("mainTitle" /
// "mainSubtitle" row).
export function ViewHeader({ title, subtitle }: { title: string; subtitle?: string }) {
  return (
    <div className="flex h-[46px] shrink-0 items-center justify-between border-b px-[22px]">
      <div className="text-[14px] font-semibold tracking-[-.01em]">{title}</div>
      {subtitle && <div className="text-[12px] text-text-tertiary">{subtitle}</div>}
    </div>
  );
}
