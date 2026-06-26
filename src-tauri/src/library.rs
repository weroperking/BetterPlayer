use serde::Serialize;
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub path: String,
    pub file_name: String,
    pub duration_seconds: Option<f64>,
    pub kind: MediaKind,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind { Video, Audio }

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm", "m4v"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "m4a", "ogg", "aac"];

pub fn media_kind(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) { Some(MediaKind::Video) }
    else if AUDIO_EXTENSIONS.contains(&ext.as_str()) { Some(MediaKind::Audio) }
    else { None }
}

pub fn scan_folder(path: &Path) -> Result<Vec<MediaItem>, String> {
    if !path.is_dir() { return Err(format!("{} is not a folder", path.display())); }
    let mut items = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let entry_path = entry.path();
        if !entry_path.is_file() { continue; }
        if let Some(kind) = media_kind(&entry_path) {
            items.push(MediaItem { file_name: entry.file_name().to_string_lossy().to_string(), path: entry_path.to_string_lossy().to_string(), duration_seconds: None, kind });
        }
    }
    items.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    Ok(items)
}
