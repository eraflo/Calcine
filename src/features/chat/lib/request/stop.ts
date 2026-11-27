/** Stop sequences, cut in the chat for GenieX versions that ignore them. */

/** Where the first stop sequence starts in `text`, if any. */
export function stopIndex(text: string, stops: readonly string[]): number | null {
  let first: number | null = null;
  for (const stop of stops) {
    if (!stop) continue;
    const index = text.indexOf(stop);
    if (index !== -1 && (first === null || index < first)) first = index;
  }
  return first;
}
