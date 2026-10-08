import { getLanguage, type Language } from "@/i18n";

/** IEC units as the GenieX CLI writes them, and as Windows shows them in French. */
const UNITS: Record<Language, readonly string[]> = {
  en: ["B", "KiB", "MiB", "GiB", "TiB", "PiB"],
  fr: ["o", "Ko", "Mo", "Go", "To", "Po"],
};

/**
 * Format a byte count in binary units, the way the GenieX CLI does
 * (`3.0 GiB`, `512 MiB`): one decimal below 10, none above.
 */
export function formatBytes(bytes: number, language: Language = getLanguage()): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  const units = UNITS[language];
  if (bytes < 1024) return `${bytes} ${units[0]}`;

  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = value < 10 ? 1 : 0;
  const number = value.toLocaleString(language, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
  return `${number} ${units[unit]}`;
}

/** Sum of the given byte counts. */
export function totalBytes(sizes: readonly number[]): number {
  return sizes.reduce((sum, size) => sum + size, 0);
}

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** `just now`, `5 minutes ago`, `3 hours ago`, `yesterday`, then a date. */
export function formatRelative(
  timestampMs: number,
  nowMs: number = Date.now(),
  language: Language = getLanguage(),
): string {
  const elapsed = Math.max(0, nowMs - timestampMs);
  const relative = new Intl.RelativeTimeFormat(language, { numeric: "auto" });
  if (elapsed < MINUTE) return relative.format(0, "second");
  if (elapsed < HOUR) return relative.format(-Math.floor(elapsed / MINUTE), "minute");
  if (elapsed < DAY) return relative.format(-Math.floor(elapsed / HOUR), "hour");
  if (elapsed < 7 * DAY) return relative.format(-Math.floor(elapsed / DAY), "day");
  return new Date(timestampMs).toLocaleDateString(language);
}

/** `1.2M`, `45K`, `812`: compact counts for downloads and likes. */
export function formatCount(count: number, language: Language = getLanguage()): string {
  return new Intl.NumberFormat(language, { notation: "compact", maximumFractionDigits: 1 }).format(
    count,
  );
}

/** A number with a fixed count of decimals, in the UI language (`80.3`, `80,3`). */
export function formatNumber(
  value: number,
  digits: number,
  language: Language = getLanguage(),
): string {
  return value.toLocaleString(language, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
}
