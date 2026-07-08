import { parseColor, rgbToHex, rgbToHsl } from "@/lib/color";
import { PlainTextPreview } from "@/components/previews/PlainTextPreview";

function ColorRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between rounded-[9px] bg-muted px-3 py-[10px]">
      <span className="text-[11px] font-semibold text-text-tertiary">{label}</span>
      <span className="font-mono text-[13px]">{value}</span>
    </div>
  );
}

export function ColorPreview({ value }: { value: string }) {
  const rgb = parseColor(value);
  if (!rgb) return <PlainTextPreview value={value} />;
  const hex = rgbToHex(rgb).toUpperCase();
  const hsl = rgbToHsl(rgb);
  return (
    <div>
      <div
        className="h-[140px] rounded-[12px] border"
        style={{ background: hex, borderColor: "var(--border-2)", boxShadow: "inset 0 0 0 1px rgba(255,255,255,.08)" }}
      />
      <div className="mt-4 flex flex-col gap-2">
        <ColorRow label="HEX" value={hex} />
        <ColorRow label="RGB" value={`rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`} />
        <ColorRow label="HSL" value={`hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`} />
      </div>
    </div>
  );
}
