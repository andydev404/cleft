# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

This project uses **bun** (see `bun.lock`), not npm/yarn.

- `bun install` — install dependencies
- `bun run dev` — start the Vite dev server only (frontend at `http://localhost:1420`)
- `bun run tauri dev` — run the full desktop app (spawns the Vite dev server via `beforeDevCommand` and opens the native window)
- `bun run build` — typecheck (`tsc`) then production-build the frontend to `dist/`
- `bun run tauri build` — build the distributable desktop app/installer
- `bunx shadcn@latest add <component>` — add a new shadcn/ui component into `src/components/ui/`

There is no lint or test tooling configured in this repo yet.

## Architecture

This is a **Tauri v2** desktop app: a Rust host process (`src-tauri/`) wraps a React/TypeScript frontend (`src/`) rendered in a system webview. The two sides communicate over Tauri's IPC bridge, not HTTP.

- **Frontend → Rust**: call `invoke("command_name", { args })` from `@tauri-apps/api/core` (see `src/App.tsx`'s `greet()` call).
- **Rust commands**: defined with `#[tauri::command]` in `src-tauri/src/lib.rs` and registered in `tauri::generate_handler![...]` inside `run()`. Add new backend functionality here.
- **Permissions**: any Tauri plugin or capability a window/command needs must be declared in `src-tauri/capabilities/default.json` (currently `core:default`, `opener:default`). Tauri's security model denies anything not explicitly granted here.
- **App config**: `src-tauri/tauri.conf.json` wires the frontend build to the Rust shell — `devUrl` must match Vite's fixed dev port (1420, set in `vite.config.ts`), and `frontendDist` points at `dist/` for production builds.

### Frontend

- Vite + React 19 + TypeScript, entry at `src/main.tsx` → `src/App.tsx`.
- Path alias `@/*` maps to `src/*` (configured in both `tsconfig.json` and `vite.config.ts`) — use `@/...` imports for anything under `src/`.
- **Styling**: Tailwind CSS v4, configured CSS-first (no `tailwind.config.js`). All theme tokens (colors, radii, fonts) live in the `@theme inline` block at the top of `src/index.css`.
- Font stack is intentionally native: `--font-sans` uses `-apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif` (no bundled webfont) so the webview feels native on macOS. Keep it that way rather than reintroducing a webfont import.
- **shadcn/ui**: components live in `src/components/ui/`, config in `components.json` (style: `radix-nova`, base color: `neutral`, icon library: `lucide`). Use the shadcn CLI (via `bunx` or the configured `shadcn` MCP server) to add components rather than hand-writing them, so generated files stay consistent with the registry.
