# Cleft

A fast, privacy-first clipboard manager for macOS and Windows. Press <kbd>⌘⇧V</kbd> (<kbd>Ctrl+Alt+V</kbd> on Windows) anywhere to open a Spotlight-style palette with everything you've copied — searchable, organized, and stored entirely on your machine.

**[Website](https://andydev404.github.io/cleft/)** · **[Download](https://github.com/andydev404/cleft/releases/latest)**

## Features

- **Global palette** — <kbd>⌘⇧V</kbd> opens instantly over any app; <kbd>Esc</kbd> or focus loss dismisses it.
- **Full-text search** — SQLite FTS5 over your whole history, with keyboard-first navigation (<kbd>↑↓</kbd>, <kbd>⏎</kbd> to copy, <kbd>⌘1–9</kbd> for quick picks).
- **Context capture** — remembers which app, window, and URL each clip came from (via the macOS Accessibility API, with your permission).
- **Smart classification** — clips are auto-detected as code, SQL, JSON, Markdown, URLs, colors, emails, or file paths, each with a tailored preview.
- **Smart duplicate detection** — re-copying something bumps the existing clip to the top (tags, pin, and counts intact) instead of cluttering history with twins.
- **Edit before paste** — press <kbd>E</kbd> / <kbd>⌘E</kbd> to tweak a copy of a clip inline (fix a typo, swap prod for staging) and paste the edited version; the original stays in history.
- **Clip expiry** — mark a clip as temporary (1h / 24h / 7d) and it deletes itself. Built for one-time codes and temp tokens.
- **Copy counts & aging** — clips show how often you've reused them, and a fading edge bar makes old vs. fresh scannable without reading timestamps.
- **Workspaces, collections & tags** — keep personal and work clips separate; group and label anything.
- **Automation rules** — "if the clip comes from app X / matches Y, then tag / collect / pin / block it."
- **Sensitive-data guard** — password-manager output, keys, and secret-shaped content are detected and never stored; a per-app blocklist is supported.
- **Local-only** — no accounts, no telemetry, no network calls at capture time. The database is encrypted (SQLCipher, key in your OS keychain) and lives in your app-data directory.

## Install

Download the latest release for your platform from [Releases](https://github.com/andydev404/cleft/releases), or build from source (below).

## Development

This is a bun-workspaces monorepo: the desktop app lives in `apps/desktop`, the landing page in `apps/web`.

Prerequisites: [Bun](https://bun.sh), [Rust](https://rustup.rs), and your platform's build tools (Xcode Command Line Tools on macOS, [Tauri's Windows prerequisites](https://tauri.app/start/prerequisites/) on Windows).

```sh
bun install              # once, at the repo root
bun run desktop:dev      # desktop app (Vite dev server + native window)
bun run desktop:build    # distributable .app/.dmg
bun run web:dev          # landing page dev server (Astro)
bun run web:build        # landing page production build
```

## Architecture

Cleft is a [Tauri v2](https://tauri.app) app: a Rust host process wraps a React/TypeScript frontend rendered in a system webview, communicating over Tauri's IPC bridge.

```
apps/desktop/            The desktop app (macOS & Windows)
  src/                   React frontend
    components/          App components (+ components/ui/ for shadcn/ui primitives)
    views/               One component per route (clipboard, timeline, collections, …)
    store/               Zustand stores
    hooks/               Shared React hooks
    lib/                 Small pure helpers
  src-tauri/src/         Rust host
    lib.rs               App wiring: plugins, global shortcut, setup
    commands.rs          All #[tauri::command] IPC handlers
    palette.rs           Palette window show/hide/toggle behavior
    tray.rs              Menu bar tray icon & menu
    clipboard.rs         Pasteboard polling & capture pipeline
    db.rs                SQLite schema, history, workspaces, eviction
    search.rs            FTS5 search
    classifier.rs        Content-type detection
    sensitive.rs         Secret detection (never-store guarantees)
    automation.rs        Automation rules engine
apps/web/                Landing page (Astro + React + GSAP)
```

Full clip content never crosses the IPC bridge in bulk — list endpoints carry previews only; content is fetched per-clip on selection.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Bug reports and PRs welcome.

## License

[MIT](LICENSE)
