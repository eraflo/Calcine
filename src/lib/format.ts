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
