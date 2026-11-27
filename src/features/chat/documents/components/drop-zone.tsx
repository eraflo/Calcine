import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { FileUp } from "lucide-react";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { documentMessages as messages } from "../messages";

/**
 * Files dropped anywhere on the window are attached to the next message.
 * Tauri hands over file paths (the webview never sees the files), so the
 * whole window is the target; the overlay covers `children`.
 */
export function DropZone({
  enabled,
  media,
  onDrop,
  children,
  className,
}: {
  enabled: boolean;
  /** Images are accepted too. */
  media: boolean;
  onDrop: (paths: string[]) => void;
  children: ReactNode;
  className?: string;
}) {
  const t = useT(messages);
  const [over, setOver] = useState(false);
  // The latest callback, without listening again on every render.
  const drop = useRef(onDrop);
  drop.current = onDrop;

  useEffect(() => {
    if (!enabled || !isTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const { type } = event.payload;
        if (type === "enter" || type === "over") setOver(true);
        else if (type === "leave") setOver(false);
        else if (type === "drop") {
          setOver(false);
          if (event.payload.paths.length > 0) drop.current(event.payload.paths);
        }
      })
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      });
    return () => {
      cancelled = true;
      unlisten?.();
      setOver(false);
    };
  }, [enabled]);

  return (
    <div className={cn("relative", className)}>
      {children}
      {over && (
        <div
          className="pointer-events-none absolute inset-3 z-40 flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed border-primary/60 bg-background/85 backdrop-blur-sm animate-in fade-in-0"
          role="status"
        >
          <span className="flex size-12 items-center justify-center rounded-full bg-primary/12 text-primary">
            <FileUp className="size-6" />
          </span>
          <span className="text-sm font-medium">{t("dropTitle")}</span>
          <span className="max-w-sm text-center text-xs text-muted-foreground">
            {media ? t("dropHintMedia") : t("dropHint")}
          </span>
        </div>
      )}
    </div>
  );
}
