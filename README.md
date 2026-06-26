# BetterPlayer

A free and open source media player alternative to QQPlayer with mpv capabilities.

## What is implemented in this MVP scaffold

- Tauri v2 desktop shell with a transparent window for an embedded video surface.
- Rust command/event contract for opening files, scanning folders, playback controls, volume, mute, and queue navigation.
- React/TypeScript overlay with an empty/audio fallback state, bottom controls, predictive seek bar, session queue, and native drag/drop handling.
- `AGENTS.md` with project guidance for future agents and contributors.

> Current backend note: `src-tauri/src/mpv.rs` is the mpv integration boundary. It preserves the command/event contract, but real playback still requires wiring the controller to `tauri-plugin-libmpv` and adding the Windows libmpv DLLs.

## Prerequisites

Install these on your machine before running the helper script:

- Node.js 20+ and npm.
- Rust stable with Cargo.
- Tauri v2 system prerequisites for your OS.
- For Windows playback: `src-tauri/lib/libmpv-wrapper.dll` and `src-tauri/lib/libmpv-2.dll`.

## Helper script

Use the bash helper for setup, dev, build, and checks:

```bash
./scripts/betterplayer.sh doctor
./scripts/betterplayer.sh setup
./scripts/betterplayer.sh dev
```

The same commands are also exposed through npm where useful:

```bash
npm run doctor
npm run setup
npm start
npm run check
npm test
```

## Command reference

| Command | Purpose |
| --- | --- |
| `./scripts/betterplayer.sh setup` | Checks Node/Rust tools, installs Node dependencies if needed, and fetches Rust crates. |
| `./scripts/betterplayer.sh dev` | Runs setup, then starts the Tauri desktop app in development mode. |
| `./scripts/betterplayer.sh frontend` | Starts only the Vite frontend server. Useful for UI-only work. |
| `./scripts/betterplayer.sh check` | Runs TypeScript and Rust compile checks. |
| `./scripts/betterplayer.sh test` | Runs the frontend build check and Rust tests. |
| `./scripts/betterplayer.sh build` | Builds the frontend and Tauri app bundle. |
| `./scripts/betterplayer.sh clean` | Removes generated `dist/` and `src-tauri/target/` output. |
| `./scripts/betterplayer.sh doctor` | Prints detected Node, npm, Rust, Cargo, dependency, and libmpv status. |

## Manual testing flow

1. Run `./scripts/betterplayer.sh doctor` and confirm Node, npm, Rust, and Cargo are detected.
2. Run `./scripts/betterplayer.sh setup` to install/fetch dependencies.
3. Add the required Windows libmpv DLLs under `src-tauri/lib/` if you want real mpv playback.
4. Run `./scripts/betterplayer.sh dev`.
5. In the app, click **Open File** and choose a local video or audio file.
6. Try dragging a single media file into the window; it should open once.
7. Try dragging or scanning a folder; supported media files should populate the session queue.
8. Use play/pause, previous/next, seek, volume, and mute controls and watch for UI state updates.

If setup fails with `403 Forbidden` for npm or crates.io, your shell/network policy is blocking dependency downloads. Fix the proxy/registry access and rerun `./scripts/betterplayer.sh setup`.
