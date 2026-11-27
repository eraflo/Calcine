/** Documents and sources as the conversation keeps them. */

import type { DocumentInfo, Source } from "@/lib/api";

/** Documents not already attached to the conversation. */
export function newDocuments(
  existing: readonly DocumentInfo[],
  added: readonly DocumentInfo[],
): DocumentInfo[] {
  const known = new Set(existing.map((document) => document.id));
  return added.filter((document) => !known.has(document.id));
}

/** Sources grouped by document, pages in order, for the line under a reply. */
export function groupSources(sources: readonly Source[]): { name: string; sources: Source[] }[] {
  const groups = new Map<string, { name: string; sources: Source[] }>();
  for (const source of sources) {
    const group = groups.get(source.documentId) ?? { name: source.name, sources: [] };
    group.sources.push(source);
    groups.set(source.documentId, group);
  }
  for (const group of groups.values()) {
    group.sources.sort((a, b) => (a.page ?? 0) - (b.page ?? 0));
  }
  return [...groups.values()];
}
