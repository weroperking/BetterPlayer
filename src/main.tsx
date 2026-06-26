import React, { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { listen } from '@tauri-apps/api/event';
import { motion, AnimatePresence } from 'framer-motion';
import { Maximize, Pause, Play, SkipBack, SkipForward, SlidersHorizontal, Square, Volume2, Camera, FolderOpen } from 'lucide-react';
import './styles.css';
import { useNativeDragDrop } from './useNativeDragDrop';
import { commands, openFileDialog, openPath } from './tauri';
import { Loaded, MediaItem, usePlayerStore } from './store';

function formatTime(seconds: number) {
  if (!Number.isFinite(seconds)) return '0:00';
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60).toString().padStart(2, '0');
  return `${mins}:${secs}`;
}

function EmptyState() {
  return <div className="empty interactive"><div><div className="orb"><FolderOpen size={58} /></div><button className="open-btn" onClick={openFileDialog}>▷ Open File ⌄</button></div></div>;
}

function SeekBar() {
  const fillRef = useRef<HTMLDivElement>(null);
  const timeRef = useRef<HTMLSpanElement>(null);
  const { loaded, positionAtSync, syncTimestamp, isPlaying } = usePlayerStore();
  useEffect(() => {
    let raf = 0;
    const tick = () => {
      const elapsed = isPlaying ? (Date.now() - syncTimestamp) / 1000 : 0;
      const position = Math.min((loaded?.durationSeconds ?? 0), positionAtSync + elapsed);
      const pct = loaded?.durationSeconds ? (position / loaded.durationSeconds) * 100 : 0;
      if (fillRef.current) fillRef.current.style.width = `${pct}%`;
      if (timeRef.current) timeRef.current.textContent = `${formatTime(position)} / ${formatTime(loaded?.durationSeconds ?? 0)}`;
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [loaded?.durationSeconds, positionAtSync, syncTimestamp, isPlaying]);

  function onSeek(event: React.MouseEvent<HTMLDivElement>) {
    if (!loaded?.durationSeconds) return;
    const rect = event.currentTarget.getBoundingClientRect();
    commands.seek(((event.clientX - rect.left) / rect.width) * loaded.durationSeconds);
  }

  return <div className="seek seek-bar-hit-area" onClick={onSeek}><div className="track"><div className="fill" ref={fillRef} /></div><span className="time" ref={timeRef}>0:00 / 0:00</span></div>;
}

function ControlBar() {
  const { isPlaying, volume, muted } = usePlayerStore();
  return <div className="controls interactive"><SeekBar /><div className="bar"><div className="cluster"><button className="icon-btn" onClick={() => commands.pause()}><Square size={18}/></button><button className="icon-btn" onClick={() => commands.previous()}><SkipBack/></button><button className="icon-btn play-ring" onClick={() => commands.togglePlay()}>{isPlaying ? <Pause/> : <Play/>}</button><button className="icon-btn" onClick={() => commands.next()}><SkipForward/></button><button title={`${muted ? 'Muted' : volume}`} className="icon-btn" onClick={() => commands.toggleMute()}><Volume2/></button></div><div className="spacer"/><div className="cluster"><button className="icon-btn utility"><Camera/></button><button className="icon-btn utility"><SlidersHorizontal/></button><button className="icon-btn utility"><Maximize/></button></div></div></div>;
}

function QueuePanel() {
  const { queue, currentIndex } = usePlayerStore();
  if (queue.length < 2) return null;
  return <aside className="queue interactive"><h2>Session Queue</h2>{queue.map((item, index) => <button className={index === currentIndex ? 'active' : ''} key={item.path} onClick={() => openPath(item.path)}><strong>{item.fileName}</strong><br/><small>{item.kind}</small></button>)}</aside>;
}


function App() {
  const over = useNativeDragDrop();
  const { loaded, error, setLoaded, setTimeSync, setVolume, setQueue, setError } = usePlayerStore();
  const [showGhost, setShowGhost] = useState(false);

  useEffect(() => {
    const unsubs = [
      listen<Loaded>('player:loaded', (e) => setLoaded(e.payload)),
      listen<{ positionSeconds: number; isPlaying: boolean; syncedAt: number }>('player:time-sync', (e) => setTimeSync(e.payload.positionSeconds, e.payload.isPlaying, e.payload.syncedAt)),
      listen<{ volume: number; muted: boolean }>('player:volume-changed', (e) => setVolume(e.payload.volume, e.payload.muted)),
      listen<{ items: MediaItem[]; currentIndex: number }>('player:queue-updated', (e) => setQueue(e.payload.items, e.payload.currentIndex)),
      listen<{ message: string }>('player:error', (e) => setError(e.payload.message)),
    ];
    return () => { unsubs.forEach(async (u) => (await u)()); };
  }, [setLoaded, setTimeSync, setVolume, setQueue, setError]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.code === 'Space') { event.preventDefault(); commands.togglePlay(); setShowGhost(true); setTimeout(() => setShowGhost(false), 420); }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  const showFallback = !loaded || !loaded.hasVideo;
  return <main className="app"><section className="surface">{showFallback && <EmptyState />}</section>{over && <div className="drop">Drop media to play</div>}{error && <div className="error interactive">{error}</div>}<QueuePanel/><AnimatePresence>{showGhost && <motion.div className="ghost" initial={{opacity:0, scale:.8}} animate={{opacity:1, scale:1}} exit={{opacity:0, scale:1.2}}><Play size={120}/></motion.div>}</AnimatePresence><ControlBar/></main>;
}

createRoot(document.getElementById('root')!).render(<App />);
