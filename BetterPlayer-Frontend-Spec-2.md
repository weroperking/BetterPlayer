# BetterPlayer — Frontend UI/UX Spec

## Context

BetterPlayer is a custom desktop media player: a QQPlayer-style skin sitting
on top of a libmpv playback engine, run inside a Tauri v2 shell. This doc
covers **only** the React/UI/UX layer. The Rust/engine side is covered in
the companion doc, `BetterPlayer-Backend-Spec.md`. The **Frontend↔Backend
Contract** section below is duplicated verbatim in that doc — treat it as a
fixed interface; if you need to change a command or event shape, update
both files together.

Decisions already locked — do not re-derive these:

- React + TypeScript + Tailwind CSS + Radix UI primitives + Framer Motion +
  Zustand, built with Vite (Tauri v2's default scaffold).
- **Native OS titlebar for MVP** — no custom window chrome. The visual goal
  is QQPlayer's *in-content* look (the video + control bar), not a replaced
  Windows frame. Custom chrome is a later phase.
- The video itself is painted by libmpv directly into the native window;
  React only renders a **transparent overlay** on top of it.

### Design reference — confirmed from screenshots

Reference screenshots of QQPlayer's current build were reviewed. Observed,
not guessed:

- **Background:** near-black throughout — video surface, control bar, and
  app frame all sit on the same dark base, no charcoal/gray panels.
- **Accent color:** a single vivid sky-blue, used consistently for every
  active/interactive element — transport icons, the highlighted item in the
  right-click menu, the empty-state glow. There is no second accent color.
- **Empty state (no file loaded):** a centered glowing icon over a radial
  blue-into-violet vignette, with "▷ Open File ⌄" in light gray/white text
  beneath it. ⚠️ Build an **original** glow/icon treatment in this spirit —
  don't trace or reproduce QQPlayer's specific illustrated mark, since that
  artwork itself isn't ours to copy. Matching the *color palette and mood*
  is the goal, not the literal asset.
- **Bottom transport bar:** left-aligned cluster — Stop, Previous, Play/Pause
  (the only button with a circular ring drawn around it, making it the
  clear visual anchor of the bar), Next, Volume — all rendered as accent-blue
  line icons. Right-aligned cluster — Snapshot, Settings/Equalizer,
  Fullscreen — same icon style but in a muted slate-gray, *not* the accent
  blue, visually marking them as secondary/utility actions.
- **Context menus** (e.g. right-click): dark gray panel, the
  hovered/selected row gets a solid accent-blue fill with white text,
  disabled items render in low-contrast muted gray.
- No screenshot showed an in-progress seek bar (both references are the
  empty/no-file state), so the playing-state progress bar styling below is
  still an informed assumption, not confirmed.

Centralize every value below in one token file so future adjustments stay a
one-file change.

## Non-Goals for MVP

- Custom titlebar / window controls (native chrome for now)
- Theme/skin switcher, settings panel
- Subtitle track UI
- Library search, filtering, thumbnails, persistence across restarts
- Playback-speed control
- Fullscreen/multi-monitor edge-case handling beyond basic OS fullscreen

If you find yourself building one of these, stop and flag it — it's a later
phase, not MVP.

## Design tokens (confirmed from reference)

```css
--bp-bg: #08090b;                          /* video surface + app frame */
--bp-bg-panel-alpha: rgba(8, 9, 11, 0.78); /* control bar, over the video */
--bp-accent: #1e9fff;                      /* transport icons, progress fill,
                                               selected menu rows, glow */
--bp-accent-hover: #4db4ff;
--bp-utility-icon: #7c8895;                /* snapshot/settings/fullscreen —
                                               deliberately not the accent */
--bp-text-primary: #f5f6f7;
--bp-text-secondary: #9aa1ab;
--bp-text-disabled: #5a5d63;
--bp-danger: #ff5c5c;
--bp-hero-glow: radial-gradient(
  circle at 50% 70%,
  rgba(30, 159, 255, 0.35) 0%,
  rgba(91, 47, 168, 0.25) 45%,
  transparent 75%
);
```

Typography: one clean, neutral UI sans for everything (e.g. Inter or the
system font stack) — a media player's chrome should disappear, not draw
attention to itself. Don't introduce a second display typeface; there's no
headline content here to differentiate.

Motion: keep Framer Motion scoped to *functional* transitions — control bar
fade in/out, button press feedback, queue panel slide — not decorative
flourishes. Respect `prefers-reduced-motion` for all of these. A media
player with extra ambient animation reads as try-hard, not premium.

## Layout structure

- Single full-window video surface — the libmpv embed paints here. This
  region, up through `html`/`body`, must stay `background: transparent`.
- **Empty state (no file loaded):** an original glow/icon treatment using
  `--bp-hero-glow`, centered, with "▷ Open File ⌄" beneath it in
  `--bp-text-primary`. This is also the drop target for the initial
  drag-and-drop hint.
- Overlay control bar pinned to the bottom: auto-hides after ~2.5s of no
  mouse movement, fades back in on mouse-move or keyboard activity.
  - **Left cluster:** Stop, Previous, Play/Pause, Next, Volume — accent-blue
    icons. Play/Pause is the only one with a circular ring around it; that
    ring is the bar's single visual anchor, don't add rings to anything else.
  - **Right cluster:** Snapshot, Settings, Fullscreen — same icon style,
    but in `--bp-utility-icon` (muted), not the accent color, so they read
    as secondary actions.
- A large, semi-transparent "ghost" play/pause icon flashes briefly in the
  center on click or spacebar — standard media-player feedback pattern.
- **Audio-only files:** libmpv won't paint a video frame, so show the same
  empty-state-style fallback (glow + icon) instead of a black rectangle.
  Read the `hasVideo` flag from the `player:loaded` event to decide which
  state to render.

## Click-through overlay (critical implementation detail)

```css
html, body { pointer-events: none; }
```

Then explicitly re-enable on interactive elements only:

```css
.control-bar, .control-bar * , .seek-bar-hit-area { pointer-events: auto; }
```

This lets native video-surface interactions (e.g. double-click for OS
fullscreen) pass through the transparent areas, while every actual control
stays clickable. Test this specifically — it's the easiest thing to get
subtly wrong and not notice until someone tries to double-click the video.

## Seek bar — predictive clock architecture

Do **not** re-render on every mpv tick. The backend only emits
`player:time-sync` on play/pause/seek plus a ~1/sec correction (see
contract below). On the frontend:

1. Keep local state: `{ positionAtSync, syncTimestamp, isPlaying }`.
2. Run a `requestAnimationFrame` loop. While playing, compute the displayed
   position as `positionAtSync + (now - syncTimestamp)`.
3. On each new `player:time-sync`, silently reconcile (snap to the
   authoritative value) — don't animate a correction, just update.
4. Update the seek bar's visual fill via a **direct ref/style write**, not
   a Zustand-driven re-render every frame. Only commit to Zustand on
   discrete user actions (drag start/end, pause) so the rest of the control
   bar doesn't re-render 60×/second.

## Drag-and-drop UI

- Use a thin hook, e.g. `useNativeDragDrop()`, that wraps the webview's
  native `onDragDropEvent` — **not** the browser's `onDrop`. The browser
  event never fires for OS file drops inside a Tauri webview.
- Show a full-window dashed-border highlight while `event.payload.type ===
  "over"`; clear it on `"drop"` or `"leave"`.
- ⚠️ Tauri v2 has fired duplicate drop events (different event IDs, same
  paths) for a single user action in some versions. The hook must dedupe:
  ignore a second event with an identical path list arriving within ~300ms
  of the first.

## Mini-library / queue panel

- Appears after a folder scan or a multi-file drop.
- Simple vertical list: filename, duration, a small video/audio kind icon.
- Click a row to play it; Up/Down or Next/Prev buttons navigate the queue.
- No search, no thumbnails, no persistence — session-only, matching the
  backend's Non-Goals.

## Frontend↔Backend Contract (authoritative — duplicated in Backend doc)

### Commands (frontend → backend)

| Command | Params | Returns | Notes |
|---|---|---|---|
| `open_path` | `{ path: string }` | `void` | Used by the dialog and single-file drop |
| `scan_folder` | `{ path: string }` | `MediaItem[]` | Becomes the new session queue |
| `play` | – | `void` | |
| `pause` | – | `void` | |
| `toggle_play` | – | `void` | |
| `seek` | `{ positionSeconds: number }` | `void` | Absolute seek |
| `set_volume` | `{ volume: number }` | `void` | 0–100 |
| `toggle_mute` | – | `void` | |
| `queue_next` | – | `void` | |
| `queue_previous` | – | `void` | |

### Events (backend → frontend)

| Event | Payload | Frequency |
|---|---|---|
| `player:loaded` | `{ path, title, durationSeconds, hasVideo, hasAudio }` | On every new file load |
| `player:time-sync` | `{ positionSeconds, isPlaying, syncedAt }` | Play/pause/seek + ~1/sec correction — never per-tick |
| `player:volume-changed` | `{ volume, muted }` | On change |
| `player:queue-updated` | `{ items: MediaItem[], currentIndex }` | After scan or multi-file drop |
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

## Suggested component breakdown

- `PlayerSurface` — owns the transparent region; renders the audio-only
  fallback art when needed
- `ControlBar` — auto-hide wrapper, hosts play/pause/seek/volume
- `SeekBar` — Radix `Slider` + the predictive-clock logic above
- `VolumeControl` — Radix `Slider` in a small popover
- `QueuePanel` — the mini-library list
- `DropOverlay` — full-window highlight during drag-over
- `usePlayerStore` (Zustand) — `paused`, `volume`, `muted`, `currentItem`,
  `queue`, `lastSync: { position, timestamp }`

## Copy & empty-state tone

Keep all UI text in the player's own plain voice: say what happened and
what to do about it, don't apologize, don't be clever. E.g. an unsupported
file shows "Can't play this file — skipping to the next one," not a vague
generic error. An empty queue says "Drop files or a folder here to start
playing," not "No items found."

## Accessibility floor (don't skip this for MVP)

- Visible keyboard focus states on every control (play, seek, volume,
  queue rows) — easy to lose track of when everything is custom-styled
- Respect `prefers-reduced-motion` for the control-bar fade and ghost-icon
  animations
- All controls operable by keyboard alone (space = play/pause, arrows =
  seek/volume), not just mouse

## Acceptance criteria — Frontend MVP is "done" when:

- [ ] Window is fully transparent behind the video — no white flash on load
- [ ] Control bar auto-hides/shows with no flicker
- [ ] Seek bar visibly glides at 60fps during playback with no re-render
  jank (check via React DevTools Profiler — the rest of the control bar
  should not re-render every frame)
- [ ] Dragging/clicking the seek bar scrubs mpv correctly and resyncs
  cleanly afterward
- [ ] Dropping one file plays it exactly once (no duplicate-event bug)
- [ ] Dropping a folder populates the queue panel
- [ ] Audio-only files show the fallback art state, never a black rect
- [ ] Volume/mute controls work and reflect state changes from elsewhere
- [ ] Every control is keyboard-reachable with a visible focus ring

If any of these aren't true, the frontend isn't MVP-complete yet — finish
these before reaching for later-phase polish (theming, subtitles, custom
chrome).
