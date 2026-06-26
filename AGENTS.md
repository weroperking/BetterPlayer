# BetterPlayer Agent Guide

## Product direction
- BetterPlayer is a Tauri v2 desktop media player with a React/TypeScript overlay and an mpv/libmpv playback backend.
- Keep MVP scope intentionally small: local files only, session-only queue, native OS titlebar, no persistence, no network streams, no settings/theme system.
- The backend/frontend command and event contract in the spec files is authoritative; update both specs if a contract change is unavoidable.

## Project layout
- `src-tauri/` contains the Rust Tauri host, player commands, queue state, folder scanning, and mpv integration boundary.
- `src/` contains the React UI, native drag/drop hook, player store, and Tauri command wrappers.
- `src/styles.css` centralizes BetterPlayer visual tokens and MVP overlay styling.
- `src-tauri/lib/` is reserved for Windows libmpv runtime binaries (`libmpv-wrapper.dll`, `libmpv-2.dll`). Do not commit large third-party binaries unless the project owner explicitly asks.

## Backend conventions
- Keep all frontend-callable functions in `src-tauri/src/commands.rs` and expose them with `#[tauri::command]`.
- Keep media discovery in `src-tauri/src/library.rs`; MVP folder scans must stay non-recursive.
- Keep mpv-specific work behind `src-tauri/src/mpv.rs` so the command contract remains stable while the engine implementation evolves.
- Emit low-frequency `player:time-sync` events only on play/pause/seek and coarse correction ticks; never stream per-frame property updates to the frontend.

## Frontend conventions
- Keep `html`, `body`, and non-interactive overlay regions transparent and click-through; re-enable pointer events only on actual controls.
- Use the native Tauri webview drag/drop API, not browser `drop` events, and keep duplicate-drop protection for identical path lists within about 300ms.
- Use predictive seek-bar rendering with `requestAnimationFrame` and direct style writes; avoid Zustand updates at 60fps.
- Preserve the QQPlayer-inspired mood without copying QQPlayer art assets.

## Testing expectations
- Run `npm install` before frontend checks when dependencies are unavailable.
- Run `npm run build` for frontend/type checks when dependencies are installed.
- Run `cargo check` from `src-tauri/` for Rust checks when crates are available.
- If network policy blocks npm or crates.io, report the exact command and limitation in the final response.
