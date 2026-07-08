# Cleft — Product Requirements Document

**Version:** 4.0  
**Status:** Final MVP  
**Target Platform:** macOS (Windows in V2)  
**MVP Timeline:** 3–4 months  
**Beachhead Audience:** Software Engineers  
**Stack:** Tauri v2 + React + TypeScript (Rust core)

---

## Table of Contents

1. [Vision](#1-vision)
2. [Mission](#2-mission)
3. [Problem Statement](#3-problem-statement)
4. [Target Users](#4-target-users)
5. [Positioning & Competitive Strategy](#5-positioning--competitive-strategy)
6. [Success Metrics](#6-success-metrics)
7. [Design Principles](#7-design-principles)
8. [Core Features — V1](#8-core-features--v1)
9. [Features Explicitly Cut from V1](#9-features-explicitly-cut-from-v1)
10. [Advanced Features — V2](#10-advanced-features--v2)
11. [User Journey](#11-user-journey)
12. [UX Principles](#12-ux-principles)
13. [Non-Functional Requirements](#13-non-functional-requirements)
14. [Monetization](#14-monetization)
15. [Technical Architecture](#15-technical-architecture)
16. [Security Architecture](#16-security-architecture)
17. [Go-To-Market Strategy](#17-go-to-market-strategy)
18. [Competitive Advantage](#18-competitive-advantage)

---

## 1. Vision

Cleft is the working memory layer for your computer.

Not a clipboard manager. Not a history list. A system that captures everything you copy — including the context of _why_ you copied it — and makes it instantly retrievable, forever.

The product should be one of the first things developers install on a new machine, and the last tool they'd ever uninstall.

---

## 2. Mission

Give every developer instant recall of everything they've ever copied, without changing how they work.

---

## 3. Problem Statement

Existing clipboard managers store _what_ you copied. None of them store _why_.

A developer copies a SQL query, a Stack Overflow snippet, an API key, a command, and a Slack message — all within 20 minutes. Two hours later, they need that query. The clipboard shows a list of raw text with no context, no source, no memory of when or where.

The result: time wasted retyping, re-searching, re-copying. Knowledge that was already in your hands, lost.

**The clipboard should be a searchable memory — not a temporary buffer.**

Secondary problems with current tools:

- Endless unsorted chronological lists
- Weak or slow search
- No understanding of what was copied
- Privacy concerns (cloud-first sync, no sensitive content protection)
- Bloated features that slow the tool down
- Poor integration with developer workflows

---

## 4. Target Users

### Beachhead: Software Engineers

**Why engineers first:**  
Developers install tools compulsively. They share what they love on Twitter, Hacker News, and Reddit. They have the highest clipboard pain (code, commands, SQL, API keys, stack traces) and the lowest resistance to paying for productivity tools. One viral post from a developer can drive thousands of installs.

**Pain points:**

- Losing commands and SQL queries minutes after copying them
- Copying multi-part things (name, email, ID, URL) into separate forms — one at a time
- Accidentally copying over something they needed
- No way to search clipboard history by what they were working on
- Sensitive data (API keys, tokens) accidentally saved in insecure tools

**Needs:**

- Code formatting and syntax highlighting in previews
- Context about where and when they copied something
- Sequential paste for filling multi-field forms
- Automatic blocking of secrets and credentials
- Sub-50ms search across everything they've ever copied

---

### Secondary Users (V1 supported, not prioritized)

**Product Managers** — notes, URLs, meeting summaries, requirements  
**Designers** — colors, asset links, Figma URLs  
**Content Creators** — research links, prompts, image references  
**Customer Support** — templated responses, variable snippets

---

## 5. Positioning & Competitive Strategy

### The Honest Competitive Landscape

| Tool              | Strengths                  | Weakness vs. Cleft                                      |
| ----------------- | -------------------------- | ------------------------------------------------------- |
| Raycast Clipboard | Free, fast, huge user base | No context, no source tracking, basic search            |
| Paste (macOS)     | Beautiful UI               | Cloud-first, no sensitive content protection, expensive |
| Maccy             | Lightweight, open source   | Minimal features, no search intelligence                |
| 1Password         | Trusted security           | Not a clipboard manager; limited history                |
| Alfred            | Power users love it        | Clipboard is secondary, not a core feature              |

### Why Cleft Wins

**The differentiator is context capture** — a feature no competitor has built well.

Every other tool shows you a list of raw text. Cleft shows you:

- What you copied
- Which app you were in
- What file or URL was active
- The time and date
- Which project or workspace it belongs to

This transforms search from _"find text that matches these words"_ to _"find that thing I copied while working on the auth bug last Tuesday"_ — and it works.

### Positioning Statement

> "Your clipboard, but it remembers everything — including why you copied it."

### Raycast Strategy

Don't fight Raycast. Ship a native Raycast plugin that surfaces Cleft's context-aware search inside Raycast's command palette. Users who love Raycast keep using it. They discover Cleft's depth. They gradually migrate.

---

## 6. Success Metrics

### North Star Metric

**Clipboard Recovery Rate** — the percentage of paste actions that come from Cleft history rather than re-copying.

> Target: **> 30%** within 30 days of install

---

### Supporting Metrics

| Metric                              | Target            | Notes                                       |
| ----------------------------------- | ----------------- | ------------------------------------------- |
| Day 7 Retention                     | > 70%             | Tool must survive the first week            |
| Day 30 Retention                    | > 55%             | Habitual use established                    |
| Day 90 Retention                    | > 45%             | Long-term stickiness                        |
| Daily Active Users                  | > 60% of installs |                                             |
| Average opens per day               | > 40              |                                             |
| Search success rate                 | > 95%             | User finds what they searched for           |
| Search latency                      | < 50ms            | Non-negotiable                              |
| NPS                                 | > 65              | Benchmark: best-in-class productivity tools |
| Free → Pro conversion               | > 12%             | Within 60 days of install                   |
| Accessibility permission grant rate | > 80%             | Measures Permission Runway effectiveness    |
| Automation permission grant rate    | > 75%             | Per-browser; tracked separately             |

---

## 7. Design Principles

### Instant

Every interaction under 100ms. Search results under 50ms. Clipboard capture under 10ms. Performance is a feature, not a goal.

### Invisible

Cleft must never interrupt a workflow. It lives in the background until summoned. No notifications. No popups. No badges. The one exception: the Permission Runway on first launch, which is intentional and necessary.

### Native

On macOS, Cleft must feel like it was built by Apple. Native typography, native animations, native keyboard patterns. Tauri's webview renders the UI, but the experience must never feel like a web app.

### Private by Default

Everything stored locally. No data leaves the machine without explicit user action. No telemetry by default. Sensitive content is blocked before it ever hits storage. Blocked content leaves zero trace — not even a log entry.

### Honest

Cleft is transparent about what it reads, why, and where it stores it. Permission screens show exactly which API is accessed and what it does. The core engine is open source and directly linkable from within the app. macOS system permission dialogs are explained _before_ they appear, not after.

### Keyboard First

Every action must be reachable without a mouse. Power users should never need to click.

### Beautiful

Not just functional — genuinely delightful. Linear-level polish. Every pixel deliberate.

---

## 8. Core Features — V1

### 8.1 Clipboard Capture

**Implementation:** A dedicated Rust background thread polls `NSPasteboard` change count every 250ms. Clipboard content is read instantly upon change detection. Context capture (window title, URL) runs on a **separate async thread pool** with a strict 50ms timeout — it never blocks clipboard saving. The React frontend never touches the clipboard directly and receives only lightweight metadata events, never raw payloads.

Automatically captures:

- Plain text
- Rich text and HTML
- Code (with language detection)
- URLs
- Images
- Files and file paths
- Colors (hex, RGB, HSL)

**Context captured alongside every clip:**

| Context Field      | API Used                                 | Notes                             |
| ------------------ | ---------------------------------------- | --------------------------------- |
| Source application | `NSWorkspace` (no permission required)   | Always captured                   |
| Window title       | Accessibility API (`kAXTitleAttribute`)  | Requires Accessibility permission |
| Active browser URL | Accessibility tree only — no AppleScript | See Section 8.2 for details       |
| Active file path   | Accessibility API (editor-specific)      | Requires Accessibility permission |
| Timestamp          | System clock                             | Always captured                   |
| Content type       | Rule-based Rust classifier               | Always captured                   |

Context is captured passively. **The app never degrades silently** — if a permission is missing, a persistent but non-intrusive indicator in the menu bar shows what is off and offers a one-click fix.

---

### 8.2 Permission Runway (First Launch)

Context capture is the product's core value proposition. The Permission Runway is the single most important UX moment in the product — it is not onboarding, it is the product working correctly from minute one.

#### The Browser URL Problem — AppleScript vs. Accessibility Tree

**Critical architectural decision:** Cleft does NOT use AppleScript to fetch browser URLs.

Why: AppleScript targeting another application (e.g. Chrome) triggers a separate macOS system dialog — "Cleft wants to control Google Chrome" — which is distinct from the Accessibility permission and fires per-browser, unpredictably, any time a new browser is opened. This completely bypasses the Permission Runway's controlled trust-building experience.

**The fix:** Cleft reads browser URLs exclusively via the Accessibility tree (`kAXURLAttribute` on the focused browser tab element), which falls under the single Accessibility permission already granted. This approach:

- Requires no additional permission dialog per browser
- Works for Chrome, Safari, Firefox, Arc, and any Chromium-based browser
- Returns the URL directly without AppleScript execution
- Is covered transparently in the Permission Runway screen

The tradeoff: Accessibility tree URL reading is marginally slower than AppleScript (~5ms vs ~2ms) and may not work for every browser version. When it fails, Cleft captures the window title only — no crash, no dialog, no disruption.

#### The Two Permissions — Explained Upfront

Cleft requires exactly two macOS permissions:

| Permission    | What It Covers                                               | When It's Asked                  |
| ------------- | ------------------------------------------------------------ | -------------------------------- |
| Accessibility | Window titles, active file paths, browser URLs (via AX tree) | First launch — Permission Runway |
| Automation    | NOT USED in V1 — explicitly avoided                          | Never in V1                      |

Both are explained on the Permission Runway screen before any system dialog appears.

#### Permission Runway Screen Design

One screen. Shown immediately on first launch. Not skippable — but not a wall either. The user is in control; Cleft is just being honest.

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│     Cleft needs one permission to work properly.      │
│                                                          │
│   ┌──────────────────────────────────────────────────┐   │
│   │  ✓  Accessibility                                │   │
│   │                                                  │   │
│   │  Reads:   Active app name + window title         │   │
│   │           Browser URL (via accessibility tree)   │   │
│   │           Active file in your code editor        │   │
│   │                                                  │   │
│   │  Never:   Screen content, keystrokes,            │   │
│   │           passwords, mouse position              │   │
│   │                                                  │   │
│   │  Stored:  Encrypted. 100% local. Always.         │   │
│   └──────────────────────────────────────────────────┘   │
│                                                          │
│   This is what lets you find:                            │
│   "that SQL I ran in TablePlus yesterday"                │
│   instead of scrolling through 400 raw clips.            │
│                                                          │
│          [ Grant Accessibility Permission → ]            │
│                                                          │
│          View exactly what this reads → github.com/...   │
│                                                          │
│   Note: macOS will show a system dialog next.            │
│   It will say "Cleft wants to control your computer." │
│   This is Apple's standard wording for Accessibility.    │
│   It does NOT mean Cleft controls your mouse or       │
│   keyboard. Click Allow.                                 │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**The pre-explanation of the system dialog wording is intentional.** The macOS Accessibility dialog uses alarming language ("wants to control your computer"). Users who see that dialog cold will deny it. Users who've just read an explanation of what it actually does will allow it. This is the most important copy in the entire product.

**If denied:** A clear, non-panicking message replaces the runway — "Context capture is off. Cleft still saves your clipboard history, but can't remember where you copied things from. Re-enable anytime: menu bar icon → Permissions → Enable Context Capture."

---

### 8.3 Universal Search

**V1 search stack: SQLite FTS5 only. No vector index. No embeddings.**

A lightweight embedding model spikes CPU at inference time, violating the < 1% CPU idle target. FTS5 with a tuned tokenizer covers 90%+ of real developer search patterns with zero overhead. Vector search moves to V2, gated on running at idle on a 2019 MacBook Pro without triggering the fan.

**Search pipeline:**

```
User types query (debounced 50ms in React)
↓
IPC command dispatched to Rust
↓
FTS5 keyword search against clips + context metadata    (< 10ms)
↓
Context field filters applied ("app:", "after:", "in:")
↓
Recency boost applied
↓
Lightweight metadata results returned to frontend       (< 50ms total)
↓
React renders result list — no full payloads loaded
↓
User selects item → full content fetched via IPC on demand
```

**FTS5 tokenizer — developer-aware:**

- `camelCase` → `camel` + `case`
- `snake_case` → `snake` + `case`
- `kebab-case` → `kebab` + `case`
- Hex values, version strings, file extensions preserved as discrete tokens
- SQL keywords, common CLI flags tokenized correctly

**Query modes:**

```
docker compose up              → keyword (FTS5)
that SQL query from yesterday  → keyword + recency weight
app:tableplus after:yesterday  → explicit context filter
```

---

### 8.4 IPC Payload Contract — Metadata Only

**This is a hard architectural rule, not a preference.**

The Tauri IPC bridge must never carry raw clipboard content. A developer copying a 10MB SQL dump, a large JSON payload, or an image asset would serialize the entire payload to JSON and pass it across the bridge — causing visible frame drops, blowing the < 100ms palette target, and violating the < 120MB memory budget.

**The contract:**

```
Rust saves full content → SQLite (encrypted)
Rust emits to frontend → ClipMetadata only

ClipMetadata {
  id: uuid,
  preview: String (max 200 chars, truncated),
  content_type: ContentType,
  source_app: String,
  window_title: String,
  timestamp: i64,
  is_favorite: bool,
  collection_id: Option<uuid>,
}
```

The frontend renders lists, search results, and the timeline using only `ClipMetadata`. Full content is fetched via an explicit async IPC command (`get_clip_content(id)`) only when the user:

- Selects an item to preview (expanded view)
- Presses Enter to paste
- Opens an item in full view

This keeps the event stream lean regardless of what was copied. A 10MB SQL dump and a 5-word text snippet produce identical IPC payloads.

---

### 8.5 Sensitive Content Detection

**Four-layer defense. In strict order. No exceptions.**

#### Layer 1 — Process Blacklist (Hardcoded, compiled into binary)

Cleft checks the active application bundle ID before reading any clipboard content. This list is compiled into the binary — not a config file, not a setting, not user-editable.

```
com.1password.1password
com.bitwarden.desktop
org.keepassxc.keepassxc
com.apple.keychainaccess
com.apple.Security
com.strongbox.mac
com.dashlane.dashlanephoneapp
com.lastpass.LastPass
```

If the frontmost app matches any entry: skip the poll cycle entirely. No clipboard read. No log entry. Nothing.

#### Layer 2 — User Blocklist (Configurable, bundle IDs)

`~/Library/Application Support/Cleft/blocklist.txt` — one bundle ID per line. Auditable, version-controllable, shareable across a team via dotfiles.

Ships pre-populated with commented examples:

```
# Add bundle IDs of apps you never want captured
# Find an app's bundle ID: mdls -name kMDItemCFBundleIdentifier /Applications/AppName.app
#
# com.apple.Terminal
# com.iterm2.iTerm2
# com.yourcompany.internal-tool
```

#### Layer 3 — Content Pattern Detection (Rust regex + heuristics)

Runs on clipboard content only after Layers 1 and 2 pass.

| Pattern                | Detection Method                                      |
| ---------------------- | ----------------------------------------------------- |
| AWS Access Keys        | `AKIA[0-9A-Z]{16}` regex                              |
| Generic API keys       | Shannon entropy > 4.5 bits/char on strings > 32 chars |
| Private keys           | PEM header pattern match                              |
| JWTs                   | Three-segment base64url structure                     |
| Credit card numbers    | Luhn algorithm validation on 13–19 digit sequences    |
| OTP codes              | 6–8 digit strings from known auth app processes       |
| SSH private keys       | OpenSSH/RSA/EC key header patterns                    |
| BIP39 recovery phrases | 12/24-word dictionary match against BIP39 wordlist    |

#### Layer 4 — Zero-Log Guarantee

Blocked content at any layer produces **no log entry of any kind**. Not the app name. Not a timestamp. Not a "blocked item" counter. A forensic attacker with full read access to the database, log files, and file system learns nothing about what was blocked or when.

---

### 8.6 Floating Command Palette

**Shortcut:** `⌘⇧V`

Opens in < 100ms over any application via Tauri's always-on-top window. The window is pre-rendered and hidden — it does not spawn on keypress, it reveals.

Contains:

- Search bar (auto-focused on open)
- Recent clips list — metadata only, renders instantly
- Content type badge per clip (`SQL`, `URL`, `Code`, `Color`, `Image`)
- Source app icon + window title + relative timestamp per clip
- Favorites pinned at top
- Collections sidebar (collapsible, `Tab` to focus)

Dismisses on `Esc` or focus loss. No dismiss animation — instant.

---

### 8.7 Rich Preview

Full content is loaded only when a clip is explicitly selected (keyboard or mouse). Renders in an expanded panel within the palette — no separate window.

| Content Type       | Preview Render                                                        |
| ------------------ | --------------------------------------------------------------------- |
| Code               | Syntax-highlighted via `tree-sitter` WASM, language label, line count |
| URL                | Page title + favicon (cached at capture time in Rust)                 |
| Image              | Full image render (loaded from SQLite blob on demand)                 |
| Color              | Large swatch + hex + RGB + HSL values                                 |
| Markdown           | Rendered HTML (via `pulldown-cmark` in Rust, sent as HTML string)     |
| JSON               | Formatted, collapsible tree (rendered in React)                       |
| SQL                | Syntax-highlighted                                                    |
| Email              | Domain badge + copy address button                                    |
| File path          | Directory breadcrumb with copy button                                 |
| Plain text (large) | Scrollable text area, character count                                 |

**No content is pre-fetched.** The palette list is pure metadata. Previews load in < 50ms on selection because they fetch a single row by primary key from an indexed, encrypted SQLite database.

---

### 8.8 Paste Stack

Copy multiple items sequentially. Paste them one at a time in order.

```
⌘C: John       → Stack position 1
⌘C: Smith      → Stack position 2
⌘C: Engineer   → Stack position 3
⌘C: Acme Inc   → Stack position 4

⌘⇧V → Select "Paste Stack" mode
Each ⌘V pastes the next item in sequence
Stack indicator in menu bar shows current position
```

Stack resets after all items are pasted, or manually cleared with `Esc` from the palette.

---

### 8.9 Favorites & Pins

Pin any clip to make it permanent:

- Never pruned by the FIFO rolling queue (see Section 14)
- Survive history clears and database migrations
- Appear at the top of the command palette
- Available via dedicated shortcut from any clip in the list (`⌘D` to pin/unpin)

---

### 8.10 Collections

Named folders for organizing clips. Created manually or auto-populated via Automation Rules.

Examples:

```
Work / Project Alpha
Work / SQL Queries
Personal / Recipes
Design / Brand Colors
Dev / Bash Commands
```

Collections appear in the palette sidebar. Selecting a collection filters the history to that collection only.

---

### 8.11 Automation Rules

Trigger-based rules that run at capture time in the Rust core — before the clip is emitted to the frontend.

**Triggers:**

- `Copied from` [bundle ID]
- `Copied from` [URL pattern]
- `Content type is` [code / image / URL / color / email / SQL / JSON]
- `Window title contains` [string]
- `Content contains` [text pattern]

**Actions:**

- Assign to collection
- Add tag
- Pin automatically
- Block (never save — extends Layer 2 for content-based rules)

**Examples:**

```
IF bundle_id == "com.tinyapp.TablePlus" AND window_title contains "prod"
THEN assign to "Work / SQL Queries" + add tag "production"

IF content_type == Color
THEN assign to "Design / Brand Colors"

IF url contains "github.com"
THEN add tag "dev"

IF bundle_id == "com.tinyapp.TablePlus"
THEN assign workspace "Work"
```

Maximum 10 active rules in V1.

---

### 8.12 Workspace Mode

Separate clipboard histories per context. Switching workspaces changes which history is visible and which automation rules apply.

Default workspaces: Personal, Work. User can add up to 5 in V1.

Workspaces do not share history. Pinned favorites are workspace-local.

---

### 8.13 Clipboard Timeline

Scrollable visual timeline of clipboard activity. Useful for end-of-session recovery.

```
09:30  [TablePlus]   SQL query       prod-db — TablePlus
09:31  [Chrome]      URL             github.com/org/repo/issues/42
09:32  [VS Code]     Error message   auth.ts — VS Code
09:33  [Figma]       Color #2563EB   Login screen — Figma
09:45  [iTerm2]      ████████████    Not captured (user blocklist)
```

Blocked app entries appear as redacted rows — the app name is shown (so users understand the gap) but the content is not. This is the only place blocked activity is acknowledged.

---

## 9. Features Explicitly Cut from V1

| Feature                         | Reason                                                                                                       |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Vector / semantic search        | CPU spikes at inference time violate idle performance target; V2 with idle-only, opt-in inference            |
| AppleScript browser URL capture | Triggers unpredictable per-browser Automation permission dialogs; replaced by Accessibility tree URL reading |
| Local LLM / AI summarization    | Violates memory and CPU targets; V2 only                                                                     |
| Cross-device sync               | Premature; architecture decision needs retention data first                                                  |
| Image OCR                       | Scope creep; V2 standalone feature                                                                           |
| Image background removal        | Not a developer workflow need                                                                                |
| PDF rendering                   | Not a clipboard workflow                                                                                     |
| Mobile companion                | Separate product                                                                                             |
| Team sharing                    | Separate V2 pricing tier                                                                                     |
| Browser extension               | V2; requires stable Rust core first                                                                          |
| Clipboard analytics             | V2                                                                                                           |
| Windows support                 | V2; macOS first for maximum polish and iteration speed                                                       |
| App Store distribution          | Accessibility API approval is inconsistent; direct download only for V1                                      |

> **The V1 test:** Would removing this feature cause a developer to not install Cleft? If the answer is no, it's cut.

---

## 10. Advanced Features — V2

| Feature                  | Notes                                                                                   |
| ------------------------ | --------------------------------------------------------------------------------------- |
| Semantic / vector search | Local embedding model, idle-only inference, opt-in toggle, gated on battery/power state |
| Cross-device sync        | E2E encrypted, CRDT-based, conflict-free, offline-first                                 |
| Windows support          | Tauri v2 port — Rust core is already cross-platform                                     |
| Browser extension        | Richer page context; no AppleScript required                                            |
| Local AI assistant       | Summarize, translate, rewrite, explain code — user-invoked only, never automatic        |
| Team shared clipboards   | Shared snippet libraries + team workspaces + admin controls                             |
| Clipboard analytics      | Most copied items, search trends, usage heatmaps                                        |
| Mobile companion         | View history, scan QR, receive copied text                                              |
| OCR on images            | Extract text from copied images automatically                                           |
| Clipboard relationships  | Auto-group related clips (title + URL + image + description)                            |

---

## 11. User Journey

### First Launch — Permission Runway

```
Install Cleft (< 2 min, direct download DMG, notarized)
↓
App opens to Permission Runway screen
↓
User reads exactly what Accessibility API reads and what it does NOT read
↓
User reads explanation of the macOS system dialog wording in advance
↓
Clicks "View source" if curious → GitHub permalink to context.rs
↓
Clicks "Grant Accessibility Permission"
↓
macOS dialog appears — user understands it, clicks Allow
↓
Cleft begins capturing with full context immediately
↓
No further setup required
```

If permission is denied, a functional reduced mode activates with a clear, persistent indicator. The permission prompt is one click away from the menu bar icon at all times.

---

### The Recovery Moment (The Hook)

```
Developer is 90 minutes into a session
↓
Needs the SQL query they ran against prod-db earlier
↓
Presses ⌘⇧V
↓
Types "prod users"
↓
Result appears instantly — TablePlus icon, "prod-db" window title, 90 min ago
↓
Presses Enter
↓
Pasted
↓
~3 minutes saved. Zero friction.
```

This is the moment that converts a trial user into a lifelong user. Everything in the product is designed to make it happen within the first hour of use.

---

## 12. UX Principles

- **Permission Runway on first launch** — one screen, honest, source-linkable, with advance explanation of macOS dialog wording
- **Explain system dialogs before they appear** — never let Apple's alarming permission language be the first thing a user reads
- **Never degrade silently** — missing permission = persistent indicator + one-click fix path
- **Zero-click organization** via automation rules — clips organized at capture time, before the palette is ever opened
- **Keyboard-first** — every action has a shortcut; mouse is always optional
- **No modal windows** — everything inline or in the palette
- **No notification banners** — Cleft never interrupts; the Permission Runway is the only proactive UI moment
- **Metadata-only rendering by default** — full content loaded on demand, never preloaded
- **Animations under 200ms** — fast enough to feel responsive, deliberate enough to feel premium
- **Dark and light mode** — system preference followed automatically
- **Minimal settings** — smart defaults cover 90% of users

---

## 13. Non-Functional Requirements

### Performance

| Operation                           | Target          | Notes                                             |
| ----------------------------------- | --------------- | ------------------------------------------------- |
| App startup (cold)                  | < 300ms         |                                                   |
| Clipboard capture (detect → save)   | < 10ms          | Content saved before context fetch begins         |
| NSPasteboard polling interval       | 250ms           |                                                   |
| Context capture (window title, URL) | < 50ms timeout  | Async, separate thread pool, never blocks capture |
| Search (up to 100K items)           | < 50ms          | FTS5, returns metadata only                       |
| Command palette open                | < 100ms         | Window pre-rendered, reveal not spawn             |
| Clip preview load (on select)       | < 50ms          | Single keyed SQLite read                          |
| IPC payload size                    | < 2KB per event | Metadata only; no raw content over bridge         |
| Memory at idle                      | < 120MB         |                                                   |
| CPU at idle                         | < 1%            |                                                   |
| CPU during clipboard capture        | < 5% spike      | Returns to < 1% within 500ms                      |

### Security

- SQLite database encrypted with AES-256 via SQLCipher
- Encryption key stored in macOS Keychain — never written to disk in plaintext
- Four-layer sensitive content defense (hardcoded blacklist → user blocklist → content patterns → zero-log guarantee)
- Blocked content produces zero log entries anywhere on the filesystem
- No telemetry, no analytics, no network calls by default
- No clipboard content ever transmitted without explicit user action
- No AppleScript execution in V1 — eliminates Automation permission surface entirely

### Reliability

- 99.9% crash-free sessions
- SQLite WAL mode — prevents corruption on crash
- `PRAGMA integrity_check` on every startup
- Automatic daily backup to `~/Library/Application Support/Cleft/backups/`
- 7-day rolling backup retention

### Accessibility

- Full keyboard navigation — no mouse required for any action
- VoiceOver support for all interactive elements
- High contrast mode (follows macOS Increase Contrast system setting)
- All keyboard shortcuts user-overridable
- Minimum readable font size: 13pt

---

## 14. Monetization

### Pricing

| Tier              | Price             | What You Get                                                                                |
| ----------------- | ----------------- | ------------------------------------------------------------------------------------------- |
| **Free**          | $0                | 500-clip rolling queue, full search, context capture, favorites, sensitive content blocking |
| **Pro**           | $29 one-time      | Unlimited history, collections, paste stack, automation rules, workspaces, rich previews    |
| **Pro + Updates** | $9 / year         | All Pro features + all future updates while subscription is active                          |
| **Teams**         | $8 / user / month | Shared clipboards, snippet libraries, admin controls, audit logs (V2)                       |

### The 500-Clip Free Tier — Rolling FIFO Queue

**Free tier behavior: rolling queue, not a hard wall.**

When a free user reaches clip 501, the oldest clip is deleted to make room. The tool continues saving every copy without interruption. This means:

- Free users always have a working clipboard manager
- Their "working memory" window is naturally bounded (~2–4 weeks of typical use)
- They feel the value being lost as old clips drop off — organic upgrade pressure, not artificial friction
- They are never surprised by a broken workflow

**What does NOT happen:**

- Cleft does not stop saving at clip 500
- Cleft does not show a blocking upgrade wall
- Cleft does not nag on every launch

**Pinned favorites are exempt from the FIFO queue.** A pinned clip never ages out, regardless of tier. This is a deliberate choice: it lets free users protect the things they truly care about, while still feeling the natural pressure of their rolling history window shrinking around them.

**The upgrade prompt:** Shown once, contextually, when the queue first wraps (clip 501 saved, clip 1 dropped): _"Your oldest clip just dropped off. Upgrade to Pro to keep your full history forever."_ Not shown again until the user actively visits Settings.

### Conversion Mechanics

- No feature nags, no countdown timers, no banners before the queue wraps
- Upgrade page explains exactly what Pro adds — no vague "premium features" language
- Teams tier is V2 — V1 go-to-market stays focused on individual developer conversion

---

## 15. Technical Architecture

### Stack Overview

```
┌──────────────────────────────────────────────────────────┐
│                   React + TypeScript                     │
│               (UI layer — display only)                  │
│          Tailwind CSS + Radix UI primitives              │
│     Renders metadata; fetches full content on demand     │
├──────────────────────────────────────────────────────────┤
│                   Tauri v2 Bridge                        │
│    IPC commands (invoke) + event stream (emit_all)       │
│         Payload contract: ClipMetadata only              │
├──────────────────────────────────────────────────────────┤
│                     Rust Core                            │
│   ┌──────────────┐  ┌───────────────┐  ┌─────────────┐  │
│   │  Clipboard   │  │    Context    │  │  Classifier │  │
│   │  Monitor     │  │    Capture    │  │  (rules)    │  │
│   │  250ms poll  │  │  async pool  │  │             │  │
│   │  NSPasteboard│  │  50ms timeout │  │             │  │
│   └──────────────┘  └───────────────┘  └─────────────┘  │
│   ┌──────────────┐  ┌───────────────┐  ┌─────────────┐  │
│   │  Sensitive   │  │  SQLite/FTS5  │  │  Automation │  │
│   │  Detection   │  │  + SQLCipher  │  │  Rules      │  │
│   │  4 layers    │  │  WAL mode     │  │             │  │
│   └──────────────┘  └───────────────┘  └─────────────┘  │
├──────────────────────────────────────────────────────────┤
│                    macOS APIs                            │
│   NSPasteboard │ NSWorkspace │ Accessibility API (AX)   │
│   Keychain │ NSProcessInfo │ NO AppleScript in V1        │
└──────────────────────────────────────────────────────────┘
```

---

### Frontend — React + TypeScript

**Framework:** React 18, TypeScript strict mode  
**Bundler:** Vite  
**Styling:** Tailwind CSS + CSS custom properties for theme tokens  
**Components:** Radix UI (accessible, unstyled, keyboard-navigable)  
**State:** Zustand — clip metadata list, search results, palette open state  
**Search:** Debounced at 50ms, dispatches `invoke('search_clips', { query })` IPC call  
**Events:** `listen('clip-added', handler)` — receives `ClipMetadata`, appends to list

**Frontend responsibilities:**

- Renders command palette, timeline, settings, collections views
- Maintains in-memory list of `ClipMetadata` objects
- Dispatches search queries to Rust via IPC
- Fetches full clip content via `invoke('get_clip_content', { id })` on demand only
- Handles keyboard navigation and global shortcut registration

**Frontend hard constraints:**

- Never reads or writes the clipboard directly
- Never polls for clipboard changes
- Never passes raw content payloads through any IPC call
- Never makes network requests (favicon cache is managed by Rust)

---

### Rust Core — `src-tauri/`

#### Clipboard Monitor (`src/clipboard.rs`)

Dedicated background thread. Polls `NSPasteboard` change count every 250ms. On change:

1. Read active app bundle ID from `NSWorkspace` (< 1ms, no permission required)
2. Check Layer 1 hardcoded blacklist → skip if match (no log)
3. Check Layer 2 user blocklist → skip if match (no log)
4. Read clipboard content from `NSPasteboard`
5. Check Layer 3 content patterns → skip if match (no log)
6. Classify content type (rule-based, Rust)
7. Save full content to SQLite (encrypted)
8. Spawn async context capture task with 50ms timeout (separate thread pool)
9. Emit `ClipMetadata` to frontend via `app_handle.emit_all("clip-added", &metadata)`

Context capture (step 8) updates the saved clip's metadata fields when it resolves. If the 50ms timeout fires, the clip is already saved with partial context — the window title field simply remains empty. No blocking. No retry. No crash.

```rust
// Pseudocode — illustrates the decoupling
pub fn start_clipboard_monitor(app_handle: AppHandle) {
    thread::spawn(move || {
        let mut last_count: i64 = -1;
        loop {
            let count = get_pasteboard_change_count();
            if count != last_count {
                last_count = count;

                let bundle_id = get_frontmost_bundle_id(); // NSWorkspace, instant

                if is_hardcoded_blocked(&bundle_id) { sleep(250ms); continue; }
                if is_user_blocked(&bundle_id) { sleep(250ms); continue; }

                if let Some(content) = read_pasteboard() {
                    if is_sensitive(&content) { sleep(250ms); continue; }

                    let content_type = classify(&content);
                    let clip_id = db::save_clip(&content, &bundle_id, content_type);

                    // Emit metadata immediately — context fetches async
                    let meta = ClipMetadata::from_saved(clip_id, &content, bundle_id);
                    app_handle.emit_all("clip-added", &meta).ok();

                    // Context capture: separate pool, 50ms hard timeout
                    let ah = app_handle.clone();
                    tokio::spawn(async move {
                        if let Ok(ctx) = timeout(50ms, fetch_context_via_ax()).await {
                            db::update_clip_context(clip_id, &ctx);
                            ah.emit_all("clip-context-updated", &(clip_id, ctx)).ok();
                        }
                        // Timeout: clip already saved, context stays empty. Done.
                    });
                }
            }
            sleep(Duration::from_millis(250));
        }
    });
}
```

#### Context Capture (`src/context.rs`)

Reads context via Accessibility tree only. No AppleScript. This is the file linked from the Permission Runway screen.

- `NSWorkspace.shared.frontmostApplication` → app name + bundle ID (no permission)
- `AXUIElementCopyAttributeValue(kAXTitleAttribute)` → window title (Accessibility)
- `AXUIElementCopyAttributeValue(kAXURLAttribute)` → browser URL (Accessibility tree, same permission)
- `kAXDocumentAttribute` on focused editor element → active file path (Accessibility)

Returns partial context gracefully if Accessibility permission is not granted or if the target app does not expose the requested AX attribute.

#### Content Classifier (`src/classifier.rs`)

Rule-based. No ML. No model files shipped. Classifies into:

`PlainText` | `Code` | `URL` | `SQL` | `JSON` | `Markdown` | `Color` | `Email` | `FilePath` | `Image` | `HTML`

Language detection for code uses keyword and syntax marker heuristics — sufficient and fast for V1.

#### Search (`src/search.rs`)

SQLite FTS5 with custom Unicode tokenizer:

- Prefix matching
- Boolean operators (`AND`, `OR`, `NOT`)
- Context field filters (`app:`, `after:`, `before:`, `in:`, `type:`)
- Recency-weighted BM25 ranking
- Returns `Vec<ClipMetadata>` — never raw content

#### Storage (`src/db.rs`)

- `rusqlite` + SQLCipher (AES-256 at rest)
- Encryption key from macOS Keychain via `security-framework` crate
- WAL mode on init
- `PRAGMA integrity_check` on startup
- FTS5 virtual table on `content` + `window_title` + `source_app` columns
- Rolling FIFO deletion trigger fires on insert when free tier clip count > 500 (skips pinned rows)

---

### IPC Contract — Enforced Types

```typescript
// TypeScript side — what the frontend is allowed to receive and request

interface ClipMetadata {
  id: string;
  preview: string; // max 200 chars
  content_type: ContentType;
  source_app: string;
  app_icon: string; // base64 PNG, 16x16
  window_title: string;
  timestamp: number;
  is_favorite: boolean;
  collection_id: string | null;
  tags: string[];
}

// The ONLY way to get full content:
invoke<string>("get_clip_content", { id: string });

// Search returns metadata only:
invoke<ClipMetadata[]>("search_clips", {
  query: string,
  filters: SearchFilters,
});
```

---

### Storage Layout

```
~/Library/Application Support/Cleft/
├── db/
│   └── clips.db              # AES-256 encrypted SQLite (SQLCipher)
├── backups/
│   ├── clips_2024-01-15.db   # Daily, 7-day rolling retention
│   └── clips_2024-01-16.db
├── blocklist.txt              # User blocklist — bundle IDs, one per line
└── rules.json                 # Automation rules
```

---

### Key Dependencies

| Dependency             | Purpose                                       |
| ---------------------- | --------------------------------------------- |
| `tauri` v2             | Desktop shell + IPC bridge                    |
| `rusqlite` + SQLCipher | Encrypted SQLite                              |
| `serde` + `serde_json` | IPC serialization                             |
| `objc2`                | NSPasteboard, NSWorkspace bindings            |
| `accessibility` crate  | AX tree — window titles, URLs, file paths     |
| `security-framework`   | macOS Keychain access for encryption key      |
| `tokio`                | Async runtime for context capture thread pool |
| `react` 18             | UI framework                                  |
| `typescript` 5.x       | Type safety                                   |
| `vite` 5.x             | Build tool                                    |
| `tailwindcss` 3.x      | Styling                                       |
| `@radix-ui/*`          | Accessible, keyboard-navigable UI primitives  |
| `zustand` 4.x          | Frontend state                                |

### Explicitly Excluded

| Excluded                 | Reason                                                          |
| ------------------------ | --------------------------------------------------------------- |
| Electron                 | Violates memory and startup targets                             |
| Node.js clipboard access | Cannot do passive OS-level monitoring                           |
| AppleScript              | Triggers unpredictable per-app Automation permission dialogs    |
| Local embedding model    | CPU spikes violate idle performance target; V2 only             |
| Any cloud calls in V1    | Trust model: zero network by default                            |
| App Store distribution   | Accessibility API approval inconsistent; direct download for V1 |

---

## 16. Security Architecture

### Threat Model

| Threat                                | Layer        | Mitigation                                                    |
| ------------------------------------- | ------------ | ------------------------------------------------------------- |
| Password manager content captured     | Layer 1      | Hardcoded bundle ID blacklist; fires before clipboard is read |
| Internal secret tool content captured | Layer 2      | User blocklist; same firing position                          |
| Unknown token format slips through    | Layer 3      | Shannon entropy detection catches high-entropy unknowns       |
| Forensic recovery of blocked content  | Layer 4      | Zero log entries for any blocked clip                         |
| Database read by attacker             | Storage      | AES-256 SQLCipher; key in Keychain                            |
| Automation permission abuse           | Architecture | AppleScript eliminated from V1 entirely                       |
| Sync interception (V2)                | V2           | E2E encrypted; server holds only ciphertext                   |
| Physical machine access               | V2           | Biometric unlock for protected collections                    |

### Open Source & Audit Plan

- Rust core engine open sourced at launch (MIT or Apache 2.0)
- Independent security audit commissioned before V1 public release
- Audit results published publicly — pass or fail, full report
- `security@Cleft.app` for responsible disclosure
- `context.rs` (the file that reads AX data) is directly linked from the Permission Runway screen

---

## 17. Go-To-Market Strategy

### Month 1 — Build in Public

- Weekly build updates on Twitter/X and a public dev blog
- Real screenshots, real tradeoffs, real bugs fixed — no vaporware aesthetic
- Rust core on GitHub from day one; stars are a leading indicator
- Write one post specifically about the AppleScript decision and the Accessibility tree alternative — developers will share it

### Month 2 — Private Beta (200 Developers)

- Recruit from r/programming, r/MacOS, Hacker News, developer Discord servers
- Criteria: heavy clipboard users who've complained about existing tools publicly
- Real conversations, not surveys — 3 qualitative interviews per week
- Primary beta health metrics: Accessibility permission grant rate + Day 7 retention
- Ship changelog every week without fail

### Month 3 — Raycast Plugin Launch

- Ship a Raycast plugin before the standalone app goes public
- Surfaces Cleft's context-aware search inside Raycast's command palette
- Zero paid acquisition; Raycast plugin store reaches tens of thousands of developers
- Trojan horse: trust is built inside a tool they already love

### Month 4 — Product Hunt Launch

- Launch on a Tuesday
- Headline: _"The clipboard that remembers why you copied something"_
- 50 beta users with real use cases ready to comment on launch day
- Direct download DMG only — no App Store wait, no approval risk

### Ongoing

- One deep workflow post per month: _"How I use Cleft for X"_
- Personal responses to every review and mention for the first 6 months
- Open source contributions from the community treated as first-class — good PRs merged publicly

---

## 18. Competitive Advantage

Cleft's moat is not a feature list. Features get copied. The moat is **trust + context + speed**, compounding as the user's history grows and becomes irreplaceable.

| Advantage                                    | Why It's Hard to Copy                                                                  |
| -------------------------------------------- | -------------------------------------------------------------------------------------- |
| Context capture via AX tree (no AppleScript) | Requires OS-level architecture commitment; can't be bolted onto existing tools         |
| Hardcoded security blacklist                 | An architectural decision cloud-first tools structurally can't make                    |
| IPC metadata contract                        | Performance discipline that Electron apps cannot match without full rewrites           |
| Decoupled context capture with timeout       | Eliminates the thread-blocking risk that would freeze lesser implementations           |
| Local-first + open source core               | Cloud-first pivot is painful and slow; open source builds compounding trust            |
| Rust + FTS5 search                           | Sub-50ms search on 100K items is structurally unavailable to Electron competitors      |
| Permission Runway with pre-explained dialogs | Trust built at first launch; requires genuine honesty commitment, not just good design |
| Developer community moat                     | Each satisfied developer reaches hundreds of peers via word of mouth                   |

### The One-Sentence Pitch

> _"Your clipboard, but it remembers everything — including why you copied it."_

---

_Cleft PRD v4.0 — Internal Use_  
_Supersedes v3.0. Changes: AppleScript eliminated — replaced by Accessibility tree URL capture; IPC payload contract enforced (metadata only, full content on demand); clipboard monitor decoupled from context capture via async thread pool with 50ms hard timeout; 500-clip free tier changed from hard wall to rolling FIFO queue with pinned-clip exemption; Automation permission surface removed entirely from V1; Permission Runway updated to explain macOS dialog wording in advance; new success metric for Automation permission grant rate removed (no longer applicable); security threat model updated._

| Developer community moat | Each satisfied developer reaches hundreds of peers via word of mouth |

### The One-Sentence Pitch

> _"Your clipboard, but it remembers everything — including why you copied it."_

---

_Cleft PRD v4.0 — Internal Use_  
_Supersedes v3.0. Changes: AppleScript eliminated — replaced by Accessibility tree URL capture; IPC payload contract enforced (metadata only, full content on demand); clipboard monitor decoupled from context capture via async thread pool with 50ms hard timeout; 500-clip free tier changed from hard wall to rolling FIFO queue with pinned-clip exemption; Automation permission surface removed entirely from V1; Permission Runway updated to explain macOS dialog wording in advance; new success metric for Automation permission grant rate removed (no longer applicable); security threat model updated._
