import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { PermissionRunway } from "@/PermissionRunway";
import { ClipPreview } from "@/ClipPreview";

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

interface ClipMetadata {
  id: string;
  preview: string;
  content_type: ContentType;
  source_app: string;
  window_title: string | null;
  url: string | null;
  timestamp: number;
}

interface ClipContextUpdate {
  id: string;
  window_title: string | null;
  url: string | null;
}

const RUNWAY_DISMISSED_KEY = "cleft:runway-dismissed";

function App() {
  const [history, setHistory] = useState<ClipMetadata[]>([]);
  const [query, setQuery] = useState("");
  const [searchResults, setSearchResults] = useState<ClipMetadata[]>([]);
  const [trusted, setTrusted] = useState<boolean | null>(null);
  const [runwayDismissed, setRunwayDismissed] = useState(
    () => localStorage.getItem(RUNWAY_DISMISSED_KEY) === "true",
  );
  const [selected, setSelected] = useState<ClipMetadata | null>(null);
  const [selectedContent, setSelectedContent] = useState<string | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  function selectClip(clip: ClipMetadata) {
    setSelected(clip);
    setSelectedContent(null);
    invoke<string | null>("get_clip_content", { id: clip.id }).then(setSelectedContent);
  }

  useEffect(() => {
    invoke<ClipMetadata[]>("get_history").then(setHistory);
    invoke<boolean>("check_accessibility_trusted").then(setTrusted);

    const unlistenAdded = listen<ClipMetadata>("clip-added", (event) => {
      setHistory((prev) => [event.payload, ...prev]);
    });
    const unlistenContext = listen<ClipContextUpdate>(
      "clip-context-updated",
      (event) => {
        setHistory((prev) =>
          prev.map((clip) =>
            clip.id === event.payload.id
              ? { ...clip, window_title: event.payload.window_title, url: event.payload.url }
              : clip,
          ),
        );
      },
    );
    // The window is revealed by toggling visibility, not remounting — so
    // "auto-focused on open" needs its own listener, not just autoFocus.
    const unlistenShown = listen("palette-shown", () => {
      setQuery("");
      setSelected(null);
      setSelectedContent(null);
      searchInputRef.current?.focus();
    });

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") invoke("hide_palette");
    };
    window.addEventListener("keydown", onKeyDown);

    return () => {
      unlistenAdded.then((f) => f());
      unlistenContext.then((f) => f());
      unlistenShown.then((f) => f());
      window.removeEventListener("keydown", onKeyDown);
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed) {
      setSearchResults([]);
      return;
    }
    const debounce = setTimeout(() => {
      invoke<ClipMetadata[]>("search_clips", { query: trimmed }).then(setSearchResults);
    }, 50);
    return () => clearTimeout(debounce);
  }, [query]);

  function requestPermission() {
    invoke("request_accessibility_trust");

    // The grant only takes effect once the user responds in System
    // Settings, not synchronously — poll until it does (or give up).
    let attempts = 0;
    pollRef.current = setInterval(async () => {
      attempts += 1;
      const isTrusted = await invoke<boolean>("check_accessibility_trusted");
      if (isTrusted || attempts >= 30) {
        if (pollRef.current) clearInterval(pollRef.current);
        setTrusted(isTrusted);
        if (isTrusted) invoke("mark_onboarded");
      }
    }, 1000);
  }

  function continueWithout() {
    localStorage.setItem(RUNWAY_DISMISSED_KEY, "true");
    setRunwayDismissed(true);
    invoke("mark_onboarded");
  }

  if (trusted === false && !runwayDismissed) {
    return (
      <PermissionRunway onGrant={requestPermission} onContinueWithout={continueWithout} />
    );
  }

  const query_trimmed = query.trim();
  const visible = query_trimmed ? searchResults : history;

  return (
    <main className="flex h-screen w-screen flex-col p-4">
      <h1 className="mb-2 text-lg font-semibold">Clipboard History</h1>

      <input
        ref={searchInputRef}
        autoFocus
        value={query}
        onChange={(e) => setQuery(e.currentTarget.value)}
        placeholder='Search... (try "type:sql" or "app:tableplus")'
        className="mb-3 w-full rounded border bg-background px-2 py-1.5 text-sm outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
      />

      {trusted === false && (
        <div className="mb-3 flex items-center justify-between gap-3 rounded border bg-muted px-3 py-2 text-sm">
          <span>
            Context capture is off. Cleft still saves your clipboard history,
            but can't remember where you copied things from.
          </span>
          <Button size="sm" onClick={requestPermission}>
            Enable
          </Button>
        </div>
      )}

      <div className="flex min-h-0 flex-1 gap-3">
        <ul className="w-2/5 min-w-0 space-y-1 overflow-y-auto">
          {visible.map((clip) => (
            <li key={clip.id}>
              <button
                onClick={() => selectClip(clip)}
                className={`w-full rounded border px-2 py-1 text-left text-sm ${
                  selected?.id === clip.id ? "border-ring bg-muted" : ""
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 text-xs font-medium text-muted-foreground">
                    {clip.content_type}
                  </span>
                  <span className="truncate">{clip.preview}</span>
                </div>
                {(clip.source_app || clip.window_title || clip.url) && (
                  <div className="mt-0.5 truncate text-xs text-muted-foreground">
                    {[clip.source_app, clip.window_title, clip.url]
                      .filter(Boolean)
                      .join(" — ")}
                  </div>
                )}
              </button>
            </li>
          ))}
        </ul>

        <div className="min-w-0 flex-1 overflow-y-auto rounded border p-3">
          {selected ? (
            <ClipPreview contentType={selected.content_type} content={selectedContent} />
          ) : (
            <div className="text-sm text-muted-foreground">Select a clip to preview it.</div>
          )}
        </div>
      </div>
    </main>
  );
}

export default App;
