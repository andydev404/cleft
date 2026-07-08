# Contributing to Cleft

Thanks for helping out! This is a small codebase — the fastest way in is the
[Architecture section of the README](README.md#architecture) and `CLAUDE.md`
(project conventions, also useful to humans).

## Setup

This is a bun-workspaces monorepo: `apps/desktop` (the Tauri app) and `apps/web`
(the Astro landing page).

```sh
bun install           # at the repo root
bun run desktop:dev   # the app
bun run web:dev       # the landing page
```

You need macOS, [Bun](https://bun.sh), [Rust](https://rustup.rs), and Xcode
Command Line Tools. Context capture requires granting Accessibility permission
to the dev build the first time it runs.

## Before you open a PR

CI runs all of these; save yourself a round-trip by running them locally:

- `cd apps/desktop && bun run build` — typechecks (`tsc`) and builds the frontend.
- `cd apps/desktop/src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
- `cd apps/web && bun run build` — if you touched the landing page.

Rust modules with real logic (`db`, `search`, `sensitive`, `markdown`, …) have
unit tests at the bottom of the file — add to them when you touch that logic.
- Keep changes focused; unrelated refactors make review slower.

## Ground rules

- **Privacy is the product.** No network calls in the capture path, no
  telemetry, and never persist anything `sensitive.rs` flags. Full clip
  content must not cross the IPC bridge in bulk (previews only in lists).
- **New backend functionality** goes in a `#[tauri::command]` registered in
  `src-tauri/src/lib.rs`; new plugin/window capabilities must be declared in
  `src-tauri/capabilities/default.json`.
- **UI primitives** come from shadcn/ui: `bunx shadcn@latest add <component>`
  rather than hand-writing them.
- **Keep the native feel** — system font stack, native macOS behaviors over
  web reimplementations.

## Reporting bugs

Open an issue with your macOS version, what you copied (shape, not contents),
and steps to reproduce. Never paste real secrets into an issue.
