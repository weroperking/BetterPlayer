import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { MediaItem } from './store';

export const mediaExtensions = ['mp4','mkv','avi','mov','webm','m4v','mp3','flac','wav','m4a','ogg','aac'];

export async function openFileDialog() {
  const selected = await open({ multiple: false, filters: [{ name: 'Media', extensions: mediaExtensions }] });
  if (typeof selected === 'string') await invoke('open_path', { path: selected });
}

export async function openDroppedPath(path: string) {
  try {
    await scanFolder(path);
  } catch {
    await invoke('open_path', { path });
  }
}

export async function openPath(path: string) {
  await invoke('open_path', { path });
}

export async function scanFolder(path: string): Promise<MediaItem[]> {
  return await invoke('scan_folder', { path });
}

export const commands = {
  play: () => invoke('play'),
  pause: () => invoke('pause'),
  togglePlay: () => invoke('toggle_play'),
  seek: (positionSeconds: number) => invoke('seek', { positionSeconds }),
  setVolume: (volume: number) => invoke('set_volume', { volume }),
  toggleMute: () => invoke('toggle_mute'),
  next: () => invoke('queue_next'),
  previous: () => invoke('queue_previous'),
};
