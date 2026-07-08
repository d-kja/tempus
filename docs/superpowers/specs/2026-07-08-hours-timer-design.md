# Hours — Floating Timer App

## Overview

A floating Tauri + Dioxus desktop timer app for tracking work sessions. Sits at top-right corner, compact by default, expands on double-click. Monochrome Tailwind zinc palette. SQLite storage with Markdown export.

## Architecture

- **Backend**: Rust (Tauri 2) — handles SQLite, file I/O, Markdown export via `#[tauri::command]`
- **Frontend**: Dioxus 0.6 (web platform, served via `dx serve`) — pure UI in WASM, calls Tauri `invoke` for all data ops
- **Communication**: `serde` typed structs over Tauri IPC bridge

## Data Model (SQLite)

### `entries` table
| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | autoincrement |
| `title` | TEXT | required |
| `description` | TEXT | nullable |
| `start_time` | TEXT | ISO 8601 datetime |
| `end_time` | TEXT | nullable, set on stop |
| `project_id` | INTEGER | FK → projects.id, nullable |
| `created_at` | TEXT | ISO 8601 |
| `updated_at` | TEXT | ISO 8601 |

### `projects` table
| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | autoincrement |
| `name` | TEXT | unique, required |
| `created_at` | TEXT | ISO 8601 |

### `settings` table (key-value store)
| Column | Type | Notes |
|--------|------|-------|
| `key` | TEXT PK | setting name |
| `value` | TEXT | JSON-encoded value |

Default settings:
- `always_on_top`: `true`
- `export_path`: `""` (user-configured)
- `window_x`, `window_y`: `null` (auto top-right on first launch)

## Tauri Commands

### Entry Commands
- `start_entry(title: String, description: Option<String>, project_id: Option<i64>) → Entry`
  - Auto-stops any currently-active entry before creating new one
  - Sets `start_time = now`, returns created entry
- `stop_entry() → Option<Entry>`
  - Sets `end_time = now` on current active entry
  - Returns the stopped entry, or `None` if no active entry
- `get_current_entry() → Option<Entry>`
  - Returns active entry (entry where `end_time IS NULL`), or `None`
- `get_entries(limit: Option<i64>, offset: Option<i64>) → Vec<Entry>`
  - Returns entries ordered by `start_time DESC`
- `update_entry(id: i64, title: Option<String>, description: Option<String>, project_id: Option<Option<i64>>) → Entry`
  - Partial update: `None` means keep existing, `Some(None)` clears project
- `delete_entry(id: i64) → ()`

### Project Commands
- `create_project(name: String) → Project`
- `get_projects() → Vec<Project>`
- `delete_project(id: i64) → ()`

### Settings Commands
- `get_settings() → HashMap<String, String>`
- `update_settings(settings: HashMap<String, String>) → ()`

### Export Command
- `export_markdown(path: String) → Result<(), String>`
  - Reads all entries (with project names via JOIN)
  - Writes markdown table with columns: Date | Hour Span | Title/Description | Projects

## UI / Window Behavior

### Window Configuration
- `always_on_top: true` (Tauri config, toggleable via settings)
- Start position: `(screen_width - window_width, 0)`
- Draggable, remembers position (saved to settings)
- Decorations: chromeless (no OS title bar). Draggable from any point on the window via CSS `drag-region`.

### Modes

**Compact (320×160)**:
- Shows: current timer display (large monospace), Start/Stop button, gear icon
- No entry list, no project tags
- Double-click → Expanded mode

**Expanded (340×480)**:
- Shows: timer, recent entries list, navigation to Setup/Settings
- Auto-collapse after 30s of no mouse/keyboard interaction within expanded view. Timer resets on any interaction (click, keypress, input focus). Timer still runs during collapsed state.
- User can manually collapse by double-clicking again

### Component Tree
```
App
├── CompactTimer        (mini view)
│   ├── TimerDisplay
│   ├── StartStopButton
│   └── GearIcon → opens Expanded/Settings
└── ExpandedView
    ├── TimerSection
    │   ├── TimerDisplay (large)
    │   ├── TitleInput
    │   ├── ProjectSelector
    │   └── StartStopResumeButton
    ├── RecentEntries
    │   └── EntryRow[] (title, project tag, time span)
    └── Navigation (bottom tabs or sidebar)
        ├── TimerPage (default)
        ├── SetupPage (export path, project CRUD)
        └── SettingsPage (always-on-top toggle)
```

### State Machine
```
IDLE ──start──→ RUNNING
RUNNING ──stop──→ STOPPED
STOPPED ──resume──→ RUNNING
STOPPED ──start_new──→ RUNNING (auto-stops & saves previous)
IDLE/RUNNING/STOPPED ──delete──→ IDLE
```

## Timer Continuous Workflow

1. User is idle → clicks Start
2. App creates entry with `start_time = now`, switches to RUNNING
3. User clicks Stop → sets `end_time = now`, switches to STOPPED
4. User can Resume (re-opens with new start_time) or Start New (auto-stops previous, creates new)
5. If user starts new while another is running: auto `stop_entry()` on current, then `start_entry()` with new data

## Styling

Tailwind zinc palette, monochrome:

| Token | Tailwind | Hex |
|-------|----------|-----|
| bg | `zinc-50` | `#fafafa` |
| bg-card | `zinc-100` | `#f4f4f5` |
| border | `zinc-200` | `#e4e4e7` |
| text-secondary | `zinc-500` | `#71717a` |
| text-primary | `zinc-900` | `#18181b` |
| accent | `zinc-800` | `#27272a` |
| hover | `zinc-200` | `#e4e4e7` |
| danger | `red-600` | `#dc2626` |

- Font: system sans-serif (timer digits: monospace)
- Border radius: `8px` (window), `6px` (controls)
- No shadows, no gradients — clean border-based separation
- Buttons: text-only with hover background

## Out of Scope (for now)
- Dark mode
- Notifications / reminders
- Cloud sync
- Analytics / charts
- Multiple workspaces
- i18n

## File Structure (target)

```
hours/
├── Cargo.toml              # workspace root
├── Dioxus.toml
├── src/
│   ├── main.rs             # entry point, launch App
│   ├── app.rs              # App component + routing
│   ├── components/
│   │   ├── compact_timer.rs
│   │   ├── expanded_view.rs
│   │   ├── timer_display.rs
│   │   ├── entry_row.rs
│   │   ├── project_selector.rs
│   │   ├── setup_page.rs
│   │   └── settings_page.rs
│   ├── state.rs            # shared state signals
│   └── tauri_bridge.rs     # invoke wrappers + types
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs          # Tauri run() setup
│       ├── db.rs           # SQLite init + migrations
│       ├── models.rs       # serde structs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── entries.rs
│       │   ├── projects.rs
│       │   ├── settings.rs
│       │   └── export.rs
│       └── export.rs       # Markdown export logic
└── assets/
    └── styles.css           # Tailwind base layer
```
