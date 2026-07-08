export function PlainTextPreview({ value }: { value: string }) {
  return (
    <div>
      <div className="whitespace-pre-wrap text-[13px] leading-[1.62]">{value}</div>
      <div className="mt-[14px] text-[11.5px] text-text-tertiary">{value.length} characters</div>
    </div>
  );
}
