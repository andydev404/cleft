import { PlainTextPreview } from "@/components/previews/PlainTextPreview";

// Native <details>/<summary> gives collapsible nodes for free — no
// expand/collapse state to manage in React.
function JsonNode({ value }: { value: unknown }) {
  if (value === null || typeof value !== "object") {
    return <span className="text-[13px]">{JSON.stringify(value)}</span>;
  }
  const entries = Array.isArray(value) ? value.map((v, i) => [i, v] as const) : Object.entries(value);
  return (
    <ul className="space-y-0.5 border-l pl-3">
      {entries.map(([key, v]) => (
        <li key={key}>
          {v !== null && typeof v === "object" ? (
            <details open>
              <summary className="cursor-pointer text-[13px] text-muted-foreground">{key}</summary>
              <JsonNode value={v} />
            </details>
          ) : (
            <div className="text-[13px]">
              <span className="text-muted-foreground">{key}: </span>
              {JSON.stringify(v)}
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

export function JsonPreview({ value }: { value: string }) {
  try {
    return <JsonNode value={JSON.parse(value)} />;
  } catch {
    return <PlainTextPreview value={value} />;
  }
}
