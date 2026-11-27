/** Finding words in the conversations, for the command palette. */

// Saved conversations may be older than their type: a message may have no
// text (only an attachment), a conversation no title.
type Searchable = {
  id: string;
  title?: string;
  updatedAt: number;
  messages: readonly { id: string; content?: string }[];
};

export type SearchHit = {
  conversationId: string;
  title: string;
  /** The message to show (`null`: the title matched, not a message). */
  messageId: string | null;
  /** Text around the first match. */
  snippet: string;
  /** Where the words are in `snippet`, to highlight them. */
  marks: [start: number, end: number][];
};

/** Shortest query searched: one letter matches everything. */
export const MIN_QUERY = 2;
const MAX_HITS = 8;
/** Characters kept around the match. */
const CONTEXT = 36;

/**
 * The conversations where every word of `query` appears, in the title or in
 * one message, ignoring case and accents. Title matches come first, then
 * the most recent conversations; one hit per conversation.
 */
export function searchConversations(
  conversations: readonly Searchable[],
  query: string,
): SearchHit[] {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (words.join("").length < MIN_QUERY) return [];
  const has = (text: string | undefined) => {
    const folded = fold(text ?? "");
    return words.every((word) => folded.includes(word));
  };

  const hits: (SearchHit & { titled: boolean; updatedAt: number })[] = [];
  for (const conversation of conversations) {
    if (has(conversation.title)) {
      const message = conversation.messages[0];
      hits.push({
        conversationId: conversation.id,
        title: conversation.title ?? "",
        messageId: null,
        ...excerpt(message?.content ?? "", words),
        titled: true,
        updatedAt: conversation.updatedAt,
      });
      continue;
    }
    const message = conversation.messages.find((candidate) => has(candidate.content));
    if (message) {
      hits.push({
        conversationId: conversation.id,
        title: conversation.title ?? "",
        messageId: message.id,
        ...excerpt(message.content ?? "", words),
        titled: false,
        updatedAt: conversation.updatedAt,
      });
    }
  }
  return hits
    .sort((a, b) => Number(b.titled) - Number(a.titled) || b.updatedAt - a.updatedAt)
    .slice(0, MAX_HITS)
    .map(({ titled: _, updatedAt: __, ...hit }) => hit);
}

/** Lower case without accents. Each character keeps its place (NFD, then
 * combining marks dropped one for one would shift them): letters are folded
 * one at a time. */
function fold(text: string): string {
  let folded = "";
  for (const char of text) {
    const base = char.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
    // Ligatures and the like may grow: keep one character per character.
    folded += base.length === char.length ? base : char.toLowerCase();
  }
  return folded;
}

/** Text around the first word found, on one line, with every word marked. */
function excerpt(text: string, words: string[]): Pick<SearchHit, "snippet" | "marks"> {
  const flat = text.replace(/\s+/g, " ").trim();
  const folded = fold(flat);
  const first = Math.min(
    ...words.map((word) => folded.indexOf(word)).filter((index) => index >= 0),
  );
  const at = Number.isFinite(first) ? first : 0;
  let start = Math.max(0, at - CONTEXT);
  let end = Math.min(flat.length, at + CONTEXT * 3);
  // Whole words at the edges.
  if (start > 0) start = flat.indexOf(" ", start) + 1 || start;
  if (end < flat.length) end = flat.lastIndexOf(" ", end) > at ? flat.lastIndexOf(" ", end) : end;
  const prefix = start > 0 ? "…" : "";
  const snippet = `${prefix}${flat.slice(start, end)}${end < flat.length ? "…" : ""}`;

  const marks: [number, number][] = [];
  const window = folded.slice(start, end);
  for (const word of words) {
    let from = 0;
    for (let index = window.indexOf(word); index >= 0; index = window.indexOf(word, from)) {
      marks.push([prefix.length + index, prefix.length + index + word.length]);
      from = index + word.length;
    }
  }
  return { snippet, marks: merge(marks) };
}

/** Overlapping marks as one, in order. */
function merge(marks: [number, number][]): [number, number][] {
  const sorted = [...marks].sort((a, b) => a[0] - b[0]);
  const merged: [number, number][] = [];
  for (const [start, end] of sorted) {
    const last = merged.at(-1);
    if (last && start <= last[1]) last[1] = Math.max(last[1], end);
    else merged.push([start, end]);
  }
  return merged;
}
