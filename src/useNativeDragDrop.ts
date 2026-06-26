import { getCurrentWebview } from '@tauri-apps/api/webview';
import { useEffect, useRef, useState } from 'react';
import { openDroppedPath, scanFolder } from './tauri';

export function useNativeDragDrop() {
  const [isOver, setIsOver] = useState(false);
  const lastDrop = useRef<{ key: string; at: number }>({ key: '', at: 0 });

  useEffect(() => {
    let unlisten: undefined | (() => void);
    getCurrentWebview().onDragDropEvent(async (event) => {
      const payload = event.payload;
      if (payload.type === 'over') setIsOver(true);
      if (payload.type === 'leave') setIsOver(false);
      if (payload.type === 'drop') {
        setIsOver(false);
        const paths = [...payload.paths].sort();
        const key = paths.join('\n');
        const now = Date.now();
        if (key === lastDrop.current.key && now - lastDrop.current.at < 300) return;
        lastDrop.current = { key, at: now };
        if (paths.length === 1) await openDroppedPath(paths[0]);
        if (paths.length > 1) await scanFolder(paths[0]);
      }
    }).then((fn) => { unlisten = fn; });
    return () => unlisten?.();
  }, []);

  return isOver;
}
