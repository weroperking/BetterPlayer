use crate::{library, mpv::MpvController};
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Emitter, State};

pub struct PlayerState {
    pub mpv: Mutex<MpvController>,
    pub queue: Mutex<Vec<library::MediaItem>>,
    pub current_index: Mutex<isize>,
}

impl Default for PlayerState {
    fn default() -> Self { Self { mpv: Mutex::new(MpvController::new()), queue: Mutex::new(Vec::new()), current_index: Mutex::new(-1) } }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TimeSyncPayload { position_seconds: f64, is_playing: bool, synced_at: u128 }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VolumePayload { volume: f64, muted: bool }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueuePayload { items: Vec<library::MediaItem>, current_index: isize }

#[derive(Clone, Serialize)]
struct ErrorPayload { message: String, path: Option<String> }

fn now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0) }

fn emit_time(app: &AppHandle, mpv: &MpvController) {
    let _ = app.emit("player:time-sync", TimeSyncPayload { position_seconds: mpv.position_seconds(), is_playing: mpv.is_playing(), synced_at: now_ms() });
}

#[tauri::command]
pub fn open_path(app: AppHandle, state: State<PlayerState>, path: String) -> Result<(), String> {
    let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?;
    match mpv.load_path(&path) {
        Ok(payload) => { app.emit("player:loaded", payload).map_err(|e| e.to_string())?; emit_time(&app, &mpv); Ok(()) }
        Err(message) => { let _ = app.emit("player:error", ErrorPayload { message: message.clone(), path: Some(path) }); Err(message) }
    }
}

#[tauri::command]
pub fn scan_folder(app: AppHandle, state: State<PlayerState>, path: String) -> Result<Vec<library::MediaItem>, String> {
    let items = library::scan_folder(&PathBuf::from(&path))?;
    *state.queue.lock().map_err(|_| "queue lock failed".to_string())? = items.clone();
    *state.current_index.lock().map_err(|_| "queue index lock failed".to_string())? = if items.is_empty() { -1 } else { 0 };
    app.emit("player:queue-updated", QueuePayload { items: items.clone(), current_index: if items.is_empty() { -1 } else { 0 } }).map_err(|e| e.to_string())?;
    if let Some(first) = items.first() { open_path(app, state, first.path.clone())?; }
    Ok(items)
}

#[tauri::command]
pub fn play(app: AppHandle, state: State<PlayerState>) -> Result<(), String> { let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?; mpv.play(); emit_time(&app, &mpv); Ok(()) }
#[tauri::command]
pub fn pause(app: AppHandle, state: State<PlayerState>) -> Result<(), String> { let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?; mpv.pause(); emit_time(&app, &mpv); Ok(()) }
#[tauri::command]
pub fn toggle_play(app: AppHandle, state: State<PlayerState>) -> Result<(), String> { let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?; mpv.toggle_play(); emit_time(&app, &mpv); Ok(()) }
#[tauri::command]
pub fn seek(app: AppHandle, state: State<PlayerState>, position_seconds: f64) -> Result<(), String> { let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?; mpv.seek(position_seconds); emit_time(&app, &mpv); Ok(()) }
#[tauri::command]
pub fn set_volume(app: AppHandle, state: State<PlayerState>, volume: f64) -> Result<(), String> { let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?; mpv.set_volume(volume); app.emit("player:volume-changed", VolumePayload { volume: mpv.volume(), muted: mpv.muted() }).map_err(|e| e.to_string()) }
#[tauri::command]
pub fn toggle_mute(app: AppHandle, state: State<PlayerState>) -> Result<(), String> { let mut mpv = state.mpv.lock().map_err(|_| "player state lock failed".to_string())?; mpv.toggle_mute(); app.emit("player:volume-changed", VolumePayload { volume: mpv.volume(), muted: mpv.muted() }).map_err(|e| e.to_string()) }

#[tauri::command]
pub fn queue_next(app: AppHandle, state: State<PlayerState>) -> Result<(), String> { move_queue(app, state, 1) }
#[tauri::command]
pub fn queue_previous(app: AppHandle, state: State<PlayerState>) -> Result<(), String> { move_queue(app, state, -1) }

fn move_queue(app: AppHandle, state: State<PlayerState>, delta: isize) -> Result<(), String> {
    let items = state.queue.lock().map_err(|_| "queue lock failed".to_string())?.clone();
    if items.is_empty() { return Ok(()); }

    let next = {
        let mut index = state.current_index.lock().map_err(|_| "queue index lock failed".to_string())?;
        let next = (*index + delta).clamp(0, items.len() as isize - 1);
        if next == *index { return Ok(()); }
        *index = next;
        next
    };

    app.emit("player:queue-updated", QueuePayload { items: items.clone(), current_index: next }).map_err(|e| e.to_string())?;
    open_path(app, state, items[next as usize].path.clone())
}
