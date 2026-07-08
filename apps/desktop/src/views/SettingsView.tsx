import { ViewHeader } from "@/components/ViewHeader";
import { useSettingsStore } from "@/store/settingsStore";
import { modKey } from "@/lib/platform";

// Privacy & Security below describes the 4 layers actually implemented in
// src-tauri/src/sensitive.rs + blocklist.rs — this section is accurate, not
// mock. The 500-clip history limit is also real, matching clipboard.rs's
// rolling-FIFO design — no "free tier" framing since there's no paid tier
// to contrast it against yet. The Preferences toggles are real too: Launch
// at login mirrors the actual OS login-item registration
// (tauri-plugin-autostart), Follow system appearance gates
// useSystemTheme's live sync, and Capture feedback sound plays a
// synthesized blip (lib/captureSound.ts) on every real capture.
const DEFENSE_LAYERS = [
  {
    num: "1",
    title: "Process Blacklist",
    desc: "1Password, Bitwarden, KeePassXC, Keychain Access and more — compiled into the binary. Clipboard is never read when these are frontmost.",
  },
  {
    num: "2",
    title: "User Blocklist",
    desc: "Your own bundle IDs in blocklist.txt — auditable, version-controllable, shareable via dotfiles.",
  },
  {
    num: "3",
    title: "Content Patterns",
    desc: "AWS keys, JWTs, private keys, credit cards, BIP39 phrases, high-entropy secrets — caught by Rust regex + Shannon entropy.",
  },
  {
    num: "4",
    title: "Zero-Log Guarantee",
    desc: "Blocked content produces no log entry of any kind. A forensic attacker learns nothing about what was blocked, or when.",
  },
];

const SHORTCUTS = [
  { name: "Open clipboard", keys: [modKey, "⇧", "V"] },
  { name: "Close", keys: ["⎋"] },
];

function SectionLabel({ children }: { children: string }) {
  return <div className="mb-3 text-[11px] font-bold uppercase tracking-[.06em] text-text-tertiary">{children}</div>;
}

export function SettingsView() {
  const { launch, darkSync, sound, toggle } = useSettingsStore();

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ViewHeader title="Settings" subtitle="Cleft 1.0" />
      <div className="flex-1 overflow-y-auto px-[22px] pb-10 pt-5" style={{ maxWidth: 720 }}>
        <SectionLabel>Privacy &amp; Security</SectionLabel>
        <div className="mb-6 overflow-hidden rounded-[12px] border bg-card">
          {DEFENSE_LAYERS.map((d, i) => (
            <div
              key={d.num}
              className={`flex items-center gap-3.5 px-4 py-3.5 ${i < DEFENSE_LAYERS.length - 1 ? "border-b" : ""}`}
            >
              <div
                className="flex h-6 w-6 shrink-0 items-center justify-center rounded-[7px] text-[11px] font-bold"
                style={{ background: "color-mix(in srgb, var(--primary) 18%, transparent)", color: "var(--primary)" }}
              >
                {d.num}
              </div>
              <div className="min-w-0 flex-1">
                <div className="text-[13px] font-semibold">{d.title}</div>
                <div className="mt-0.5 text-[12px] leading-[1.45] text-muted-foreground">{d.desc}</div>
              </div>
              <div className="mt-[5px] h-2 w-2 shrink-0 rounded-full bg-success" />
            </div>
          ))}
        </div>

        <SectionLabel>Preferences</SectionLabel>
        <div className="mb-6 overflow-hidden rounded-[12px] border bg-card">
          <PrefToggle title="Launch at login" desc="Start Cleft automatically" on={launch} onToggle={() => toggle("launch")} />
          <PrefToggle
            title="Follow system appearance"
            desc="Match macOS light / dark mode"
            on={darkSync}
            onToggle={() => toggle("darkSync")}
          />
          <PrefToggle
            title="Capture feedback sound"
            desc="Subtle click when a clip is saved"
            on={sound}
            onToggle={() => toggle("sound")}
            last
          />
        </div>
        <div className="mb-6 overflow-hidden rounded-[12px] border bg-card">
          <PrefValue title="History limit" desc="Oldest unpinned clips drop first" value="500 clips" last />
        </div>

        <SectionLabel>Global Shortcuts</SectionLabel>
        <div className="overflow-hidden rounded-[12px] border bg-card">
          {SHORTCUTS.map((sc, i) => (
            <div key={sc.name} className={`flex items-center justify-between px-4 py-3.5 ${i < SHORTCUTS.length - 1 ? "border-b" : ""}`}>
              <span className="text-[13px]">{sc.name}</span>
              <span className="flex gap-1">
                {sc.keys.map((k, j) => (
                  <kbd key={j} className="min-w-[22px] rounded-[5px] border bg-muted px-[7px] py-[3px] text-center text-[11px] font-semibold">
                    {k}
                  </kbd>
                ))}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function PrefToggle({
  title,
  desc,
  on,
  onToggle,
  last,
}: {
  title: string;
  desc: string;
  on: boolean;
  onToggle: () => void;
  last?: boolean;
}) {
  return (
    <div className={`flex items-center gap-3.5 px-4 py-3.5 ${last ? "" : "border-b"}`}>
      <div className="min-w-0 flex-1">
        <div className="text-[13px] font-medium">{title}</div>
        <div className="mt-0.5 text-[11.5px] text-text-tertiary">{desc}</div>
      </div>
      <button
        onClick={onToggle}
        className="relative h-5 w-[34px] shrink-0 rounded-full border transition-colors"
        style={{ background: on ? "var(--primary)" : "var(--muted)", borderColor: on ? "var(--primary)" : "var(--border-2)" }}
      >
        <span className="absolute top-[1px] h-4 w-4 rounded-full bg-white shadow transition-all" style={{ left: on ? "15px" : "1px" }} />
      </button>
    </div>
  );
}

function PrefValue({ title, desc, value, last }: { title: string; desc: string; value: string; last?: boolean }) {
  return (
    <div className={`flex items-center gap-3.5 px-4 py-3.5 ${last ? "" : "border-b"}`}>
      <div className="min-w-0 flex-1">
        <div className="text-[13px] font-medium">{title}</div>
        <div className="mt-0.5 text-[11.5px] text-text-tertiary">{desc}</div>
      </div>
      <div className="rounded-[6px] border bg-muted px-[9px] py-1 font-mono text-[12px] text-muted-foreground">{value}</div>
    </div>
  );
}
