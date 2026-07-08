import { useEffect, useRef } from "react";
import gsap from "gsap";

interface Clip {
  app: string;
  letter: string;
  color: string;
  preview: string;
  badge: string;
  badgeColor: string;
}

const CLIPS: Clip[] = [
  { app: "Figma", letter: "F", color: "#9b54ff", preview: "#FF6B35", badge: "COLOR", badgeColor: "#ff8a4c" },
  { app: "Terminal", letter: "T", color: "#1d8a5b", preview: "git rebase -i origin/main", badge: "CODE", badgeColor: "#46b29d" },
  { app: "Safari", letter: "S", color: "#3a8ee6", preview: "stripe.com/docs/api/charges", badge: "URL", badgeColor: "#7c9cff" },
  { app: "TablePlus", letter: "T", color: "#e0a23c", preview: "SELECT * FROM orders WHERE…", badge: "SQL", badgeColor: "#e0a23c" },
  { app: "Mail", letter: "M", color: "#2f6fed", preview: "ana@acme.com", badge: "EMAIL", badgeColor: "#c0426b" },
  { app: "VS Code", letter: "V", color: "#5b63d3", preview: '{ "retries": 3, "timeout": 500 }', badge: "JSON", badgeColor: "#b06cff" },
  { app: "Notes", letter: "N", color: "#e0533f", preview: "Ship the beta on Thursday", badge: "TEXT", badgeColor: "#9aa4b2" },
];

function Row({ clip }: { clip: Clip }) {
  return (
    <div className="pd-row">
      <span className="pd-icon" style={{ background: clip.color }}>
        {clip.letter}
      </span>
      <span className="pd-preview">{clip.preview}</span>
      <span className="pd-badge" style={{ color: clip.badgeColor, borderColor: `color-mix(in srgb, ${clip.badgeColor} 40%, transparent)` }}>
        {clip.badge}
      </span>
    </div>
  );
}

// The hero's live demo: the Cleft palette continuously "catching" copies.
// Rows arrive on top every couple of seconds; the system clipboard slot
// above shows only the latest copy overwriting the previous one — the
// one-slot problem and its fix, side by side.
export function PaletteDemo() {
  const listRef = useRef<HTMLDivElement>(null);
  const slotRef = useRef<HTMLSpanElement>(null);
  const indexRef = useRef(4);

  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const list = listRef.current;
    if (!list) return;

    const interval = setInterval(() => {
      const clip = CLIPS[indexRef.current % CLIPS.length];
      indexRef.current += 1;

      if (slotRef.current) {
        slotRef.current.textContent = clip.preview;
        gsap.fromTo(slotRef.current, { opacity: 0, y: 4 }, { opacity: 1, y: 0, duration: 0.35 });
      }

      // Rows are appended imperatively (outside React's render) so GSAP can
      // own their entrance without fighting reconciliation.
      const el = document.createElement("div");
      el.className = "pd-row";
      el.innerHTML = `
        <span class="pd-icon" style="background:${clip.color}">${clip.letter}</span>
        <span class="pd-preview"></span>
        <span class="pd-badge" style="color:${clip.badgeColor};border-color:color-mix(in srgb, ${clip.badgeColor} 40%, transparent)">${clip.badge}</span>`;
      (el.querySelector(".pd-preview") as HTMLElement).textContent = clip.preview;
      list.prepend(el);
      gsap.fromTo(el, { opacity: 0, y: -14, scale: 0.98 }, { opacity: 1, y: 0, scale: 1, duration: 0.45, ease: "power2.out" });

      while (list.children.length > 5) list.removeChild(list.lastElementChild!);
    }, 2200);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="pd" aria-hidden="true">
      <div className="pd-slot">
        <span className="pd-slot-label">Your clipboard</span>
        <span className="pd-slot-value" ref={slotRef}>
          {CLIPS[3].preview}
        </span>
        <span className="pd-slot-note">holds 1 item</span>
      </div>
      <div className="pd-window">
        <div className="pd-search">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4">
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.5-3.5" />
          </svg>
          <span>Search everything you've copied…</span>
          <span className="pd-kbd">⌘⇧V</span>
        </div>
        <div className="pd-list" ref={listRef}>
          {CLIPS.slice(0, 4).map((clip) => (
            <Row key={clip.preview} clip={clip} />
          ))}
        </div>
        <div className="pd-footer">
          <span>Cleft</span>
          <span>
            keeps all of them <span className="pd-dot">●</span> on your Mac
          </span>
        </div>
      </div>
    </div>
  );
}
