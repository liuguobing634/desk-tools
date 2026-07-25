# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

A Tauri v2 + SvelteKit 5 + TypeScript desktop app for managing daily todos and Markdown notes. Data is persisted as files (JSON/Markdown) in the platform's app data directory.

## Build & Run

```bash
npm run tauri dev    # Start in development mode (Vite dev server + Tauri window)
npm run tauri build  # Production build (Vite build + Rust compile)
npm run check        # Type-check the frontend (svelte-check)
```

There are no tests currently.

## Architecture

```
Frontend (SvelteKit)           Backend (Rust / Tauri)
─────────────────────────     ─────────────────────────
src/routes/+page.svelte        src-tauri/src/lib.rs          (command registration)
  ├── TodoPanel.svelte           src-tauri/src/commands/
  │   uses services/todo.ts        ├── todo.rs               (2 commands: load/save)
  └── NotePanel.svelte             └── note.rs               (10 commands for notes + groups + images)
      uses services/note.ts      src-tauri/src/storage.rs    (app data path helpers)
```

- **SPA with SSG**: Single route `/` with `prerender = true`, `ssr = false` in `+layout.ts`
- **IPC layer**: Frontend components call `invoke()` from `@tauri-apps/api/core`; the service files in `src/lib/services/` wrap these calls
- **State management**: Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Module switching**: `activeModule` toggles between Todo/Note panels, persisted in `localStorage`

## Data Persistence

- **Todos**: Single `todos.json` in app data dir (`storage.rs` provides `app_data_file_path`)
- **Notes**: Individual `.md` files in app data dir; group assignment stored as `<!-- group: <uuid> -->` HTML comment in each file
- **Groups**: Individual `.json` files under `groups/` subdirectory
- **Images**: Copied to `images/` subdirectory, imported via native file dialog (`rfd` crate)

## Key Dependencies

- **Frontend**: `@tauri-apps/api` v2, `marked` (Markdown rendering), Svelte 5, Vite 6
- **Backend**: `tauri` v2 (feature `protocol-asset`), `tauri-plugin-opener` v2, `serde`/`serde_json`, `uuid` v1, `rfd` (native file dialogs)
- **Build**: `@sveltejs/adapter-static` for SSG output
