import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useRef, useState } from "react";

/**
 * Files and folders dropped on the window (Tauri gives their paths, which a
 * browser drop doesn't). Calls `onDrop` with the first path and returns
 * whether something is being dragged over the window.
 */
export function useFileDrop(onDrop: (path: string) => void): boolean {
  const [dragging, setDragging] = useState(false);
  const callback = useRef(onDrop);
  callback.current = onDrop;

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (payload.type === "enter" || payload.type === "over") {
          setDragging(true);
        } else if (payload.type === "leave") {
          setDragging(false);
        } else {
          setDragging(false);
          const [first] = payload.paths;
          if (first) callback.current(first);
        }
      })
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return dragging;
}
