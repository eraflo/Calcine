/**
 * Documents attached to a conversation. Their text stays in Calcine's data
 * folder (read by the backend into passages); the chat only keeps their
 * description, and asks before each message what the model should read.
 */

import type { ContextMode, DocumentContext, DocumentInfo, Source } from "@/lib/api";
import { replyReserve } from "./context";

export type { ContextMode, DocumentContext, DocumentInfo, Source };

/** Image files vision models take, as attachments. */
const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];

export function extensionOf(path: string): string {
  const name = path.split(/[\\/]/).at(-1) ?? "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export function fileName(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}

export function isImagePath(path: string): boolean {
  return IMAGE_EXTENSIONS.includes(extensionOf(path));
}

/** Used when the model's window is unknown. */
const UNKNOWN_WINDOW_BUDGET = 2048;
/** Documents always get at least this share of the room left after the
 * reply (older messages are forgotten to make it)… */
const MIN_SHARE = 0.35;
/** …and at most this share, so the conversation still fits. */
const MAX_SHARE = 0.65;

/**
 * Tokens the documents may take in the next request: what the conversation
 * leaves free, between {@link MIN_SHARE} and {@link MAX_SHARE} of the room
 * left after the reply.
 */
export function documentBudget(
  window: number | null,
  conversationTokens: number,
  maxTokens: number,
): number {
  if (window === null) return UNKNOWN_WINDOW_BUDGET;
  const room = window - replyReserve(maxTokens, window);
  const free = room - conversationTokens;
  const budget = Math.min(Math.max(free, room * MIN_SHARE), room * MAX_SHARE);
  return Math.max(0, Math.floor(budget));
}

/** What to look for in the documents: the message, then the user's two
 * previous ones, for follow-up questions. */
export function documentQueries(
  text: string,
  earlier: readonly { role: string; content: string }[],
): string[] {
  const previous = earlier
    .filter((message) => message.role === "user" && message.content.trim())
    .slice(-2)
    .reverse()
    .map((message) => message.content);
  return [text, ...previous];
}

/** Where the documents go in the request. Read whole, they join the system
 * prompt, where they stay the same from one message to the next (GenieX
 * reuses what it already read); passages go before the message they
 * answer. */
export function documentParts(context: DocumentContext | null): {
  system?: string;
  prefix?: string;
} {
  if (!context?.text || context.sources.length === 0) return {};
  return context.mode === "full" ? { system: context.text } : { prefix: context.text };
}

/** The system prompt with the documents, before any summary. */
export function withDocuments(systemPrompt: string, documents: string | undefined): string {
  const prompt = systemPrompt.trim();
  if (!documents) return prompt;
  return prompt ? `${prompt}\n\n${documents}` : documents;
}

/** The user's message with the passages that may answer it. */
export function withPassages(content: string, passages: string | undefined): string {
  if (!passages) return content;
  return `${passages}\n${content}`;
}

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
