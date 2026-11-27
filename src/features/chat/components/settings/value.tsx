import type { ReactNode } from "react";

/** A setting's current value, next to its label. */
export function Value({ children }: { children: ReactNode }) {
  return (
    <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{children}</span>
  );
}
