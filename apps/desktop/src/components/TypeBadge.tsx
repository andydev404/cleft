import { Badge } from "@/components/ui/badge";

// Per-type badge colors imported from the CopyFlow design's TYPE_COLOR map.
const TYPE_COLOR: Record<string, string> = {
  SQL: "#e0a23c",
  JSON: "#e0a23c",
  Code: "#7c9cff",
  URL: "#46b29d",
  Email: "#46b29d",
  Color: "#b06cff",
  Markdown: "#9aa0aa",
  FilePath: "#9aa0aa",
  PlainText: "#9aa0aa",
  HTML: "#9aa0aa",
};

export function TypeBadge({ type }: { type: string }) {
  const color = TYPE_COLOR[type] ?? "#9aa0aa";
  return (
    <Badge
      variant="outline"
      className="shrink-0 rounded-[5px] border-0 px-1.5 py-0.5 text-[9.5px] font-bold uppercase tracking-[.03em]"
      style={{ background: `${color}14`, color, boxShadow: `inset 0 0 0 1px ${color}2e` }}
    >
      {type}
    </Badge>
  );
}
