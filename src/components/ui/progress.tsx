import { cn } from "@/lib/utils";

/** A thin bar. `value` is 0–100; `null` shows an indeterminate shimmer. */
export function Progress({
  value,
  className,
  barClassName,
  label,
}: {
  value: number | null;
  className?: string;
  /** Override the bar color, e.g. `bg-info`. */
  barClassName?: string;
  label: string;
}) {
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={value ?? undefined}
      className={cn("h-1.5 w-full overflow-hidden rounded-full bg-muted", className)}
    >
      <div
        className={cn(
          "h-full rounded-full bg-primary transition-[width] duration-300 ease-out",
          value === null && "w-1/3 animate-pulse",
          barClassName,
        )}
        style={value === null ? undefined : { width: `${value}%` }}
      />
    </div>
  );
}
