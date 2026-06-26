# BetterPlayer — Backend & Engine Spec

## Context

BetterPlayer is a custom desktop media player. The pitch: a sleek, QQPlayer-style
UI sitting on top of a real media engine (mpv), instead of a bloated native
player or a thin wrapper around Chromium's `<video>` tag.

Decisions already locked for this project — **do not re-derive or re-debate
these**:

- **Shell:** Tauri v2 (Rust), not Electron, not Electrobun. Electrobun is too
  young for this much native-embedding work; Tauri's ecosystem already has
  maintained mpv-embedding plugins.
- **Engine:** mpv via **libmpv**, not VLC. libmpv's render API is built for
  embedding; VLC's engine is heavier and harder to overlay a custom UI on.
- **Embedding model:** `tauri-plugin-libmpv` (direct native embed via window
  handle), **not** `tauri-plugin-mpv` (separate process + JSON-IPC). A
  detached mpv process desyncs from the UI window on resize/drag — direct
  embed lets the OS keep them in sync for free.
- **Frontend:** React + TypeScript + Tailwind + Radix UI + Framer Motion +
  Zustand. Covered in the companion doc, `BetterPlayer-Frontend-Spec.md`.
- **Dev platform:** Windows 10/11 first. macOS and Linux are a later porting
  phase, not part of this build.

This document covers **only** backend/Rust/Tauri-host responsibilities. The
**Frontend↔Backend Contract** section below is duplicated verbatim in the
frontend doc — if you change a command or event here, update both files in
the same change.

## Non-Goals for MVP

Do not build these yet, even if they seem like natural next steps:

- Custom window chrome / titlebar (native OS titlebar + controls for now)
- Network stream URLs (local files only — video and audio)
- Persistent library database across app restarts (session-only queue)
- Subtitle track switching, shader/upscale profiles, playback-speed control
- Multi-window, picture-in-picture
- Installers, code signing, auto-update

If a task seems to require one of these, stop and flag it instead of
expanding scope.

## Tech stack (pinned)

- Tauri v2 (Rust)
- `tauri-plugin-libmpv` (Rust side) + `tauri-plugin-libmpv-api` (the JS
  package the frontend calls)
- `tauri-plugin-dialog` (native "Open File" dialog)
- `tauri-plugin-fs` (folder scanning, scoped to user-selected paths only)
- Target: Windows 10/11, WebView2

## Project layout

```
betterplayer/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs      # all #[tauri::command] functions
│   │   ├── library.rs       # folder scan / media item discovery
│   │   └── mpv.rs           # thin wrapper around tauri-plugin-libmpv setup
│   ├── lib/                 # libmpv-wrapper.dll, libmpv-2.dll (Windows)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
├── src/                      # React frontend — see Frontend spec
└── package.json
```

## mpv embedding setup

- Install via `npm run tauri add libmpv`, then follow the Windows setup:
  download `libmpv-wrapper.dll` from the plugin's releases and the matching
  `libmpv-2.dll` build, place both in `src-tauri/lib/`, and declare them as
  bundled resources in `tauri.conf.json` (`bundle.resources`).
- The window must be created with `transparent: true` so the video surface
  shows through beneath the React overlay. The frontend's `html`/`body` must
  be transparent for this to work — coordinate with the frontend doc.
- Recommended init flags: `--hwdec=auto-safe`, `--keep-open=yes`,
  `--force-window=yes` (keeps the render surface alive even with nothing
  loaded yet).

## File loading paths (MVP)

1. **Open File dialog** — `tauri-plugin-dialog`'s `open()` with
   `{ multiple: false, filters: [{ name: "Media", extensions: [...] }] }`.
2. **Native OS drag-and-drop** — via the webview's native
   `onDragDropEvent`, **not** the browser's `onDrop`/`ondragover`, which
   never fire for OS-level file drops inside a Tauri webview (the native
   window layer intercepts the drag first). Requires
   `dragDropEnabled: true` in `tauri.conf.json` (this is the default, so
   usually nothing to change — but if drops silently stop working, check
   this flag first).
   - ⚠️ **Known Tauri v2 quirk:** a single user drop can fire the drop event
     twice with different event IDs. Whichever side owns the dedupe logic
     (recommend: the frontend hook, see frontend doc) must ignore a second
     event carrying an identical path list within a short window (e.g.
     ~300ms) so one drop doesn't double-queue a file.
3. **Folder scan ("mini library")** — `tauri-plugin-fs` `readDir`,
   **non-recursive** for MVP, filtered to supported extensions (below),
   returned as a `MediaItem[]` that becomes the session queue.

## Supported extensions (MVP)

- Video: `mp4`, `mkv`, `avi`, `mov`, `webm`, `m4v`
- Audio: `mp3`, `flac`, `wav`, `m4a`, `ogg`, `aac`

This list only governs what the folder scanner treats as "playable" for
listing purposes — actual playback works on anything libmpv supports.
Network streams are explicitly out of scope (see Non-Goals).

## Frontend↔Backend Contract (authoritative — duplicated in Frontend doc)

### Commands (frontend → backend)

| Command | Params | Returns | Notes |
|---|---|---|---|
| `open_path` | `{ path: string }` | `void` | Loads a file into mpv (`loadfile`). Used by the dialog and single-file drop. |
| `scan_folder` | `{ path: string }` | `MediaItem[]` | Non-recursive; becomes the new session queue. |
| `play` | – | `void` | |
| `pause` | – | `void` | |
| `toggle_play` | – | `void` | |
| `seek` | `{ positionSeconds: number }` | `void` | Absolute seek. |
| `set_volume` | `{ volume: number }` | `void` | 0–100. |
| `toggle_mute` | – | `void` | |
| `queue_next` | – | `void` | No-op if queue empty or at end. |
| `queue_previous` | – | `void` | |

### Events (backend → frontend)

| Event | Payload | Frequency |
|---|---|---|
| `player:loaded` | `{ path, title, durationSeconds, hasVideo, hasAudio }` | On every new file load |
| `player:time-sync` | `{ positionSeconds, isPlaying, syncedAt }` | On play/pause/seek, plus a ~1/sec correction tick. **Never on every mpv property tick** — see Frontend doc for why this matters. |
| `player:volume-changed` | `{ volume, muted }` | On change |
| `player:queue-updated` | `{ items: MediaItem[], currentIndex }` | After `scan_folder` or a multi-file drop |
| `player:error` | `{ message, path? }` | On failure |
| `player:end-of-file` | `{ path }` | On natural playback end |

### `MediaItem` shape

```ts
type MediaItem = {
  path: string;
  fileName: string;
  durationSeconds?: number;
  kind: "video" | "audio";
};
```

## Error handling

- mpv init failure → emit `player:error`; show a native error dialog at
  startup rather than a silent blank window.
- Unsupported/corrupt file → emit `player:error` with the file path, then
  auto-advance the queue (skip), don't crash.
- Missing libmpv binaries at runtime → fail fast with a clear message, not
  a frozen or blank UI.

## Acceptance criteria — Backend MVP is "done" when:

- [ ] App launches on Windows 10/11 to a blank, responsive window
- [ ] Open File dialog loads and plays a local video file (audio + video)
- [ ] Open File dialog loads and plays a local audio-only file
- [ ] Dropping a single file plays it exactly once (no duplicate-event bug)
- [ ] Dropping a folder scans it and queues all supported files
- [ ] play / pause / seek / volume commands all work and emit correct events
- [ ] `player:time-sync` fires at the agreed low frequency, not per-tick
- [ ] Closing and reopening the app never crashes (no persistence required)

If any of these aren't true, the backend isn't MVP-complete yet — keep
working here before moving on to polish or later-phase features.
