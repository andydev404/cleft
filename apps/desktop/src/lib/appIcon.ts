// Deterministic letter + color per bundle ID, since captures can come from
// any installed app — we can't hardcode a lookup table the way the design's
// demo data does (APPS = {TablePlus: ..., 'VS Code': ...}) and expect it to
// cover a real user's machine.
const PALETTE = [
  "#2f6fed",
  "#e0533f",
  "#9b54ff",
  "#1d8a5b",
  "#c0426b",
  "#3a8ee6",
  "#5b63d3",
  "#e0a23c",
  "#46b29d",
  "#7c9cff",
];

function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}

export function appDisplayName(bundleId: string): string {
  if (!bundleId) return "Unknown";
  const last = bundleId.split(".").filter(Boolean).pop() ?? bundleId;
  return last.charAt(0).toUpperCase() + last.slice(1);
}

export function appIcon(bundleId: string): { letter: string; bg: string } {
  const name = appDisplayName(bundleId);
  return {
    letter: name.charAt(0).toUpperCase() || "?",
    bg: bundleId ? PALETTE[hashString(bundleId) % PALETTE.length] : "#9aa0aa",
  };
}

export function relativeTime(unixSeconds: number): string {
  const diffMs = Date.now() - unixSeconds * 1000;
  const mins = Math.floor(diffMs / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}
