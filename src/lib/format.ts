const IEC_UNITS = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"] as const;

/**
 * Format a byte count in IEC units, the way the GenieX CLI does
 * (`3.0 GiB`, `512 MiB`): one decimal below 10, none above.
 */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  if (bytes < 1024) return `${bytes} B`;

  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < IEC_UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = value < 10 ? 1 : 0;
  return `${value.toFixed(digits)} ${IEC_UNITS[unit]}`;
}

/** Sum of the given byte counts. */
export function totalBytes(sizes: readonly number[]): number {
  return sizes.reduce((sum, size) => sum + size, 0);
}

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** `just now`, `5 min ago`, `3 h ago`, `2 days ago`, then a date. */
export function formatRelative(timestampMs: number, nowMs: number = Date.now()): string {
  const elapsed = Math.max(0, nowMs - timestampMs);
  if (elapsed < MINUTE) return "just now";
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)} min ago`;
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)} h ago`;
  if (elapsed < 7 * DAY) {
    const days = Math.floor(elapsed / DAY);
    return days === 1 ? "yesterday" : `${days} days ago`;
  }
  return new Date(timestampMs).toLocaleDateString();
}
