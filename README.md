# Tempus — Desktop Time Tracker

A minimal, always-on-top time tracker built with **Tauri 2** + **Dioxus 0.7** (Rust/WASM frontend) + **SQLite**.

Tracks work hours per-entry with project tagging, exports to Markdown, and stays out of your way.

_**Important**: Don't use it, it's AI slop for my use-case._

## Features

- **Compact floating timer** — 320×160px always-on-top window with start/stop/reset
- **Project management** — create/delete projects, assign entries on start
- **Entry history** — browse, filter by project, delete individual or clear all
- **Always-on-top toggle** — persist preference across restarts
- **Markdown export** — export all entries as a table (`timesheet.md`)
- **Multi-window** — compact timer, settings, and new-entry windows managed by Tauri

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| Frontend | Dioxus 0.7 (Rust → WASM) |
| Backend | Rust (Tauri commands) |
| Database | SQLite via rusqlite |
| Styling | Vanilla CSS (dark theme, frosted glass) |

## Development

```bash
# Install dependencies (Rust + wasm32 target)
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli

# Run in dev mode (hot-reload on port 1420)
cargo tauri dev

# Build release
cargo tauri build
```

### Layout

```
src/                  # Dioxus frontend (WASM)
├── main.rs           # Entry point, launches App
├── app.rs            # Root component, routes by query param
├── bridge.rs         # Tauri invoke bindings
├── state.rs          # AppState / TimerState signals
├── full_app_window.rs # Full settings/entries window
└── components/
    ├── compact_timer.rs      # Small always-on-top timer
    ├── timer_display.rs      # HH:MM:SS display
    ├── new_entry_window.rs   # Title + project entry form
    ├── entry_row.rs          # Single entry in history list
    ├── navigation.rs         # Tab bar (Timer / Setup / Settings)
    └── project_selector.rs   # Project filter pills

src-tauri/src/        # Tauri backend
├── main.rs           # Binary entry point
├── lib.rs            # App setup, all #[tauri::command] definitions
├── db.rs             # SQLite migration + Database struct
├── models.rs         # Entry / Project structs
└── commands/
    ├── entries.rs    # CRUD + start/stop/resume
    ├── projects.rs   # CRUD for projects
    ├── settings.rs   # Key-value settings store
    ├── export.rs     # Markdown export
    └── window.rs     # Window position / always-on-top persistence
```

## Configuration

Window placement on Wayland (Hyprland) is controlled via window rules:

```
windowrulev2 = float, class:^(com.d-kja-hours)$
windowrulev2 = move 100% 0%, class:^(com.d-kja-hours)$
```

## Tests

```bash
# Backend unit tests (in-memory SQLite)
cargo test -p hours

# Frontend is pure Dioxus/WASM — no test runner set up yet
```

