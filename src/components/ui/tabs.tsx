import type * as React from "react";
import { cn } from "@/lib/utils";

export type Tab<T extends string> = { value: T; label: React.ReactNode };

/** An underlined tab bar. The panel below is up to the caller. */
export function Tabs<T extends string>({
  tabs,
  value,
  onChange,
  label,
  className,
}: {
  tabs: readonly Tab<T>[];
  value: T;
  onChange: (value: T) => void;
  label: string;
  className?: string;
}) {
  const move = (event: React.KeyboardEvent, index: number) => {
    const step = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
    if (!step) return;
    event.preventDefault();
    const next = tabs[(index + step + tabs.length) % tabs.length];
    if (next) onChange(next.value);
  };

  return (
    <div role="tablist" aria-label={label} className={cn("flex gap-4 border-b", className)}>
      {tabs.map((tab, index) => {
        const selected = tab.value === value;
        return (
          <button
            key={tab.value}
            type="button"
            role="tab"
            aria-selected={selected}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(tab.value)}
            onKeyDown={(event) => move(event, index)}
            className={cn(
              "-mb-px border-b-2 px-0.5 pb-2 text-sm font-medium transition-colors outline-none focus-visible:text-foreground",
              selected
                ? "border-primary text-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground",
            )}
          >
            {tab.label}
          </button>
        );
      })}
    </div>
  );
}
