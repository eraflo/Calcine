import { useId } from "react";
import { cn } from "@/lib/utils";

/**
 * A small area chart of 0–100 values, drawn in `currentColor` (set it with a
 * text color class such as `text-npu`). The newest value is on the right;
 * `length` slots keep the scale steady while the history fills up.
 */
export function Sparkline({
  values,
  length,
  className,
}: {
  values: readonly number[];
  length: number;
  className?: string;
}) {
  const gradient = useId();
  const width = 100;
  const height = 32;
  const step = width / Math.max(1, length - 1);
  const offset = length - values.length;
  const firstX = offset * step;
  const points = values.map((value, index) => {
    const x = (offset + index) * step;
    const y = height - (Math.min(100, Math.max(0, value)) / 100) * (height - 2) - 1;
    return `${x.toFixed(2)},${y.toFixed(2)}`;
  });

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      className={cn("h-8 w-full overflow-visible", className)}
      aria-hidden
    >
      <defs>
        <linearGradient id={gradient} x1="0" x2="0" y1="0" y2="1">
          <stop offset="0%" stopColor="currentColor" stopOpacity={0.35} />
          <stop offset="100%" stopColor="currentColor" stopOpacity={0} />
        </linearGradient>
      </defs>
      {points.length > 1 && (
        <>
          <polygon
            points={`${firstX.toFixed(2)},${height} ${points.join(" ")} ${width},${height}`}
            fill={`url(#${gradient})`}
          />
          <polyline
            points={points.join(" ")}
            fill="none"
            stroke="currentColor"
            strokeWidth={1.5}
            strokeLinejoin="round"
            vectorEffect="non-scaling-stroke"
          />
        </>
      )}
    </svg>
  );
}
