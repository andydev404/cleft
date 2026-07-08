# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

This is a **bun-workspaces monorepo** (bun, not npm/yarn): `apps/desktop` is the Tauri app, `apps/web` is the Astro landing page. Run `bun install` once at the repo root.

Root scripts:

- `bun run desktop:dev` / `bun run desktop:build` — the desktop app (`tauri dev` / `tauri build` in `apps/desktop`)
- `bun run web:dev` / `bun run web:build` — the landing page (Astro in `apps/web`)

Inside `apps/desktop`:

- `bun run dev` — Vite dev server only (frontend at `http://localhost:1420`)
- `bun run build` — typecheck (`tsc`) then production-build the frontend to `dist/`
- `bunx shadcn@latest add <component>` — add a new shadcn/ui component into `src/components/ui/`

CI (`.github/workflows/ci.yml`) gates on the desktop frontend build (tsc), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (in `apps/desktop/src-tauri`), and the Astro build. Rust unit tests live at the bottom of each module (`db.rs`, `search.rs`, `sensitive.rs`, `markdown.rs`, …). There is no JS lint or test tooling configured yet.

## Architecture

### Desktop app (`apps/desktop`)

A **Tauri v2** desktop app: a Rust host process (`src-tauri/`) wraps a React/TypeScript frontend (`src/`) rendered in a system webview. The two sides communicate over Tauri's IPC bridge, not HTTP.

- **Frontend → Rust**: call `invoke("command_name", { args })` from `@tauri-apps/api/core`.
- **Rust commands**: defined with `#[tauri::command]` in `src-tauri/src/commands.rs` and registered in `tauri::generate_handler![...]` in `lib.rs`'s `run()`. Add new backend functionality here. Domain logic lives in the sibling modules (`db.rs`, `clipboard.rs`, `search.rs`, `classifier.rs`, `sensitive.rs`, `automation.rs`, …); commands stay thin. Window behavior is in `palette.rs`, the tray in `tray.rs`.
- **IPC contract**: full clip content never crosses the bridge in bulk — list endpoints (`get_history`, `search_clips`) carry previews only; content is fetched per-clip via `get_clip_content` on selection. Keep it that way.
- **Permissions**: any Tauri plugin or capability a window/command needs must be declared in `src-tauri/capabilities/default.json` (currently `core:default`, `opener:default`). Tauri's security model denies anything not explicitly granted here.
- **App config**: `src-tauri/tauri.conf.json` wires the frontend build to the Rust shell — `devUrl` must match Vite's fixed dev port (1420, set in `vite.config.ts`), and `frontendDist` points at `dist/` for production builds.

### Frontend

- Vite + React 19 + TypeScript, entry at `src/main.tsx` → `src/router.tsx` (TanStack Router; one route per view).
- Layout: `src/components/` (app components, plus `components/ui/` for shadcn/ui primitives), `src/views/` (route components), `src/store/` (Zustand stores), `src/hooks/`, `src/lib/` (pure helpers), `src/types.ts` (IPC payload types mirroring the Rust structs).
- Path alias `@/*` maps to `src/*` (configured in both `tsconfig.json` and `vite.config.ts`) — use `@/...` imports for anything under `src/`.
- **Styling**: Tailwind CSS v4, configured CSS-first (no `tailwind.config.js`). All theme tokens (colors, radii, fonts) live in the `@theme inline` block at the top of `src/index.css`.
- Font stack is intentionally native: `--font-sans` uses `-apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif` (no bundled webfont) so the webview feels native on macOS. Keep it that way rather than reintroducing a webfont import.
- **shadcn/ui**: components live in `src/components/ui/`, config in `components.json` (style: `radix-nova`, base color: `neutral`, icon library: `lucide`). Use the shadcn CLI (via `bunx` or the configured `shadcn` MCP server) to add components rather than hand-writing them, so generated files stay consistent with the registry.

### Landing page (`apps/web`)

- Astro with React islands (`@astrojs/react`), Partytown for third-party scripts, GSAP for animation, `@astrojs/sitemap` for SEO.
- One page (`src/pages/index.astro`) + `src/layouts/Layout.astro` (all SEO meta, JSON-LD). Global styles in `src/styles/global.css`; design tokens are CSS custom properties on `:root`.
- The `site` URL in `astro.config.mjs` drives canonical URLs and the sitemap — update it when the production domain changes.
- All motion respects `prefers-reduced-motion`.
