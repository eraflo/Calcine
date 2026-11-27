/** JSON in structured output: the schema the user typed, and replies. */

/** The schema typed in the settings, or why it can't be used. */
export function parseSchema(
  text: string,
): { schema: object } | { error: "not-json" | "not-object" } {
  try {
    const schema: unknown = JSON.parse(text);
    if (typeof schema !== "object" || schema === null || Array.isArray(schema)) {
      return { error: "not-object" };
    }
    return { schema };
  } catch {
    return { error: "not-json" };
  }
}

/** A reply that is JSON, pretty-printed; `null` for anything else. */
export function prettyJson(text: string): string | null {
  const trimmed = text.trim();
  if (!/^[[{]/.test(trimmed)) return null;
  try {
    return JSON.stringify(JSON.parse(trimmed), null, 2);
  } catch {
    return null;
  }
}
