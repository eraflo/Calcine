// Parsing what people type in the chat settings.

export function clamp(value: number, min: number, max: number) {
  return Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : min;
}

/** Empty means "default"; anything else is clamped to the range. */
export function parseOptional(
  text: string,
  min: number,
  max: number,
  integer: boolean,
): number | null {
  if (text.trim() === "") return null;
  const value = Number(text);
  if (!Number.isFinite(value)) return null;
  return clamp(integer ? Math.round(value) : value, min, max);
}

/** Stop sequences are typed with `\n` and `\t` for line breaks and tabs. */
export function unescapeStop(text: string): string {
  return text.replaceAll("\\n", "\n").replaceAll("\\t", "\t");
}

export function escapeStop(sequence: string): string {
  return sequence.replaceAll("\n", "\\n").replaceAll("\t", "\\t");
}
