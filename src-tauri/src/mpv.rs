use serde::Serialize;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedPayload {
    pub path: String,
    pub title: String,
    pub duration_seconds: f64,
    pub has_video: bool,
    pub has_audio: bool,
}

#[derive(Default)]
pub struct MpvController {
    loaded: Option<LoadedPayload>,
    is_playing: bool,
    volume: f64,
    muted: bool,
    position_seconds: f64,
}

impl MpvController {
    pub fn new() -> Self { Self { volume: 75.0, ..Self::default() } }

    pub fn load_path(&mut self, path: &str) -> Result<LoadedPayload, String> {
        let title = Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string());
        let has_video = crate::library::media_kind(Path::new(path)).map(|kind| matches!(kind, crate::library::MediaKind::Video)).unwrap_or(true);
        let payload = LoadedPayload { path: path.to_string(), title, duration_seconds: 0.0, has_video, has_audio: true };
        self.loaded = Some(payload.clone());
        self.position_seconds = 0.0;
        self.is_playing = true;
        Ok(payload)
    }

    pub fn play(&mut self) { self.is_playing = true; }
    pub fn pause(&mut self) { self.is_playing = false; }
    pub fn toggle_play(&mut self) { self.is_playing = !self.is_playing; }
    pub fn seek(&mut self, position_seconds: f64) { self.position_seconds = position_seconds.max(0.0); }
    pub fn set_volume(&mut self, volume: f64) { self.volume = volume.clamp(0.0, 100.0); }
    pub fn toggle_mute(&mut self) { self.muted = !self.muted; }
    pub fn is_playing(&self) -> bool { self.is_playing }
    pub fn position_seconds(&self) -> f64 { self.position_seconds }
    pub fn volume(&self) -> f64 { self.volume }
    pub fn muted(&self) -> bool { self.muted }
}
