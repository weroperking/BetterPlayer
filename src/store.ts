import { create } from 'zustand';

export type MediaItem = { path: string; fileName: string; durationSeconds?: number; kind: 'video' | 'audio' };
export type Loaded = { path: string; title: string; durationSeconds: number; hasVideo: boolean; hasAudio: boolean };

type PlayerState = {
  loaded?: Loaded;
  queue: MediaItem[];
  currentIndex: number;
  isPlaying: boolean;
  volume: number;
  muted: boolean;
  positionAtSync: number;
  syncTimestamp: number;
  error?: string;
  setLoaded: (loaded: Loaded) => void;
  setTimeSync: (positionSeconds: number, isPlaying: boolean, syncedAt: number) => void;
  setVolume: (volume: number, muted: boolean) => void;
  setQueue: (items: MediaItem[], currentIndex: number) => void;
  setError: (error?: string) => void;
};

export const usePlayerStore = create<PlayerState>((set) => ({
  queue: [],
  currentIndex: -1,
  isPlaying: false,
  volume: 75,
  muted: false,
  positionAtSync: 0,
  syncTimestamp: Date.now(),
  setLoaded: (loaded) => set({ loaded, error: undefined, positionAtSync: 0, syncTimestamp: Date.now() }),
  setTimeSync: (positionAtSync, isPlaying, syncTimestamp) => set({ positionAtSync, isPlaying, syncTimestamp }),
  setVolume: (volume, muted) => set({ volume, muted }),
  setQueue: (queue, currentIndex) => set({ queue, currentIndex }),
  setError: (error) => set({ error }),
}));
