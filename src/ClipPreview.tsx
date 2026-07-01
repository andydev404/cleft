import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

type ContentType =
  | "PlainText"
  | "Code"
  | "URL"
  | "SQL"
  | "JSON"
  | "Markdown"
  | "Color"
  | "Email"
  | "FilePath"
  | "HTML";

function CopyButton({ text }: { text: string }) {
  return (
    <Button size="sm" variant="outline" onClick={() => invoke("copy_to_clipboard", { text })}>
      Copy
    </Button>
  );
}

function parseColor(input: string): { r: number; g: number; b: number } | null {
  const trimmed = input.trim();

  const hex = trimmed.match(/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/)?.[1];
  if (hex) {
    const full = hex.length === 3 ? hex.split("").map((c) => c + c).join("") : hex;
    const num = parseInt(full, 16);
    return { r: (num >> 16) & 255, g: (num >> 8) & 255, b: num & 255 };
  }

  const rgb = trimmed.match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/i);
  if (rgb) return { r: +rgb[1], g: +rgb[2], b: +rgb[3] };

  const hsl = trimmed.match(/^hsla?\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%/i);
  if (hsl) return hslToRgb(+hsl[1], +hsl[2], +hsl[3]);

  return null;
}

function hslToRgb(h: number, s: number, l: number) {
  s /= 100;
  l /= 100;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  const [r, g, b] =
    h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return { r: Math.round((r + m) * 255), g: Math.round((g + m) * 255), b: Math.round((b + m) * 255) };
}

function rgbToHex(r: number, g: number, b: number) {
  return "#" + [r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("");
}

function rgbToHsl(r: number, g: number, b: number) {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h *= 60;
  }
  return { h: Math.round(h), s: Math.round(s * 100), l: Math.round(l * 100) };
}

function ColorPreview({ value }: { value: string }) {
  const rgb = parseColor(value);
  if (!rgb) return <PlainTextPreview value={value} />;
  const hex = rgbToHex(rgb.r, rgb.g, rgb.b);
  const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b);
  return (
    <div className="space-y-3">
      <div className="h-24 w-full rounded border" style={{ backgroundColor: hex }} />
      <div className="space-y-1 text-sm">
        <div>hex: {hex}</div>
        <div>
          rgb: rgb({rgb.r}, {rgb.g}, {rgb.b})
        </div>
        <div>
          hsl: hsl({hsl.h}, {hsl.s}%, {hsl.l}%)
        </div>
      </div>
    </div>
  );
}

// ponytail: native <details>/<summary> gives collapsible nodes for free —
// no expand/collapse state to manage in React.
function JsonNode({ value }: { value: unknown }) {
  if (value === null || typeof value !== "object") {
    return <span className="text-sm">{JSON.stringify(value)}</span>;
  }
  const entries = Array.isArray(value) ? value.map((v, i) => [i, v] as const) : Object.entries(value);
  return (
    <ul className="space-y-0.5 border-l pl-3">
      {entries.map(([key, v]) => (
        <li key={key}>
          {v !== null && typeof v === "object" ? (
            <details open>
              <summary className="cursor-pointer text-sm text-muted-foreground">{key}</summary>
              <JsonNode value={v} />
            </details>
          ) : (
            <div className="text-sm">
              <span className="text-muted-foreground">{key}: </span>
              {JSON.stringify(v)}
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

function JsonPreview({ value }: { value: string }) {
  try {
    return <JsonNode value={JSON.parse(value)} />;
  } catch {
    return <PlainTextPreview value={value} />;
  }
}

function MarkdownPreview({ value }: { value: string }) {
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("render_markdown", { content: value }).then(setHtml);
  }, [value]);

  if (html === null) return null;
  // Safe to inject: render_markdown sanitizes with ammonia in Rust before
  // this ever reaches the frontend — see src-tauri/src/markdown.rs.
  return <div className="text-sm [&_h1]:text-lg [&_h1]:font-semibold [&_h2]:font-semibold [&_ul]:list-disc [&_ul]:pl-4 [&_a]:underline [&_code]:rounded [&_code]:bg-muted [&_code]:px-1" dangerouslySetInnerHTML={{ __html: html }} />;
}

// ponytail: monospace block, no tree-sitter syntax highlighting — that's a
// WASM-grammar integration on its own. Add it if plain text stops cutting it.
function CodePreview({ value, label }: { value: string; label: string }) {
  const lines = value.split("\n").length;
  return (
    <div className="space-y-1">
      <div className="text-xs text-muted-foreground">
        {label} · {lines} line{lines === 1 ? "" : "s"}
      </div>
      <pre className="max-h-80 overflow-auto rounded border bg-muted p-2 text-xs">{value}</pre>
    </div>
  );
}

function EmailPreview({ value }: { value: string }) {
  const domain = value.split("@")[1] ?? "";
  return (
    <div className="flex items-center justify-between gap-2">
      <div className="text-sm">
        {value}
        {domain && (
          <span className="ml-2 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">{domain}</span>
        )}
      </div>
      <CopyButton text={value} />
    </div>
  );
}

function FilePathPreview({ value }: { value: string }) {
  const segments = value.split("/").filter(Boolean);
  return (
    <div className="flex items-center justify-between gap-2">
      <div className="truncate text-sm text-muted-foreground">
        {segments.map((seg, i) => (
          <span key={i}>
            {i > 0 && <span className="mx-1">/</span>}
            {seg}
          </span>
        ))}
      </div>
      <CopyButton text={value} />
    </div>
  );
}

// ponytail: no favicon/title fetch — that needs a network call at capture
// time, which cuts against "no network calls by default." Domain + copy is
// the useful part without that tradeoff.
function UrlPreview({ value }: { value: string }) {
  let hostname = value;
  try {
    hostname = new URL(value).hostname;
  } catch {
    // not a fully-qualified URL; fall back to showing it as-is
  }
  return (
    <div className="flex items-center justify-between gap-2">
      <div className="truncate text-sm">
        <span className="text-muted-foreground">{hostname}</span>
        <div className="truncate text-xs text-muted-foreground">{value}</div>
      </div>
      <CopyButton text={value} />
    </div>
  );
}

function PlainTextPreview({ value }: { value: string }) {
  return (
    <div className="space-y-1">
      <textarea
        readOnly
        value={value}
        className="h-64 w-full resize-none rounded border bg-background p-2 text-sm outline-none"
      />
      <div className="text-xs text-muted-foreground">{value.length} characters</div>
    </div>
  );
}

export function ClipPreview({ contentType, content }: { contentType: ContentType; content: string | null }) {
  if (content === null) {
    return <div className="text-sm text-muted-foreground">Loading…</div>;
  }

  switch (contentType) {
    case "Color":
      return <ColorPreview value={content} />;
    case "JSON":
      return <JsonPreview value={content} />;
    case "Markdown":
      return <MarkdownPreview value={content} />;
    case "Code":
    case "SQL":
      return <CodePreview value={content} label={contentType} />;
    case "Email":
      return <EmailPreview value={content} />;
    case "FilePath":
      return <FilePathPreview value={content} />;
    case "URL":
      return <UrlPreview value={content} />;
    default:
      return <PlainTextPreview value={content} />;
  }
}
