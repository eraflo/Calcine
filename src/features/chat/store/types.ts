/** What the chat keeps: conversations, their messages, saved prompts. */

import type { ContextMode, DocumentInfo, Source } from "@/lib/api";
import type { Attachment } from "../lib/attachments";
import type { ChatSettings } from "../lib/request";
import type { StreamStats } from "../lib/sse";

export type ChatMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  /** Images and recordings; their data is in `useMediaPayloads`. */
  attachments?: Attachment[];
  /** Documents attached with this message (they then belong to the
   * conversation). */
  documents?: DocumentInfo[];
  /** Where the reply may come from, in the attached documents. */
  sources?: Source[];
  /** Whether the model read the documents whole, or passages of them. */
  documentMode?: ContextMode;
  /** Passages read, out of all of them, and the tokens they took, for
   * this reply. */
  passagesRead?: { read: number; total: number; tokens?: number };
  reasoning?: string;
  stats?: StreamStats;
  error?: string;
  /** Stopped by the user before the end. */
  stopped?: boolean;
  /** Every version of a regenerated reply, the one shown included. */
  versions?: ReplyVersion[];
  /** Which of `versions` is shown. */
  version?: number;
};

/** One version of a reply. */
export type ReplyVersion = Pick<
  ChatMessage,
  "reasoning" | "stats" | "error" | "stopped" | "sources" | "documentMode" | "passagesRead"
> & { content: string };

export type Conversation = {
  id: string;
  title: string;
  modelId: string;
  messages: ChatMessage[];
  updatedAt: number;
  /** The first message the model still sees, when the conversation
   * outgrew its context window. */
  contextFrom?: string;
  /** What came before `contextFrom`, summarized by the model. */
  summary?: string;
  /** Documents the model can read, in the order they were attached. */
  documents?: DocumentInfo[];
};

/** A system prompt saved for reuse. */
export type SavedPrompt = { id: string; name: string; text: string };

/** The saved chat state and its actions. */
export type ChatState = {
  conversations: Conversation[];
  activeId: string | null;
  settings: ChatSettings;
  /** Model for the next new conversation. */
  lastModelId: string | null;
  create: (modelId: string) => string;
  select: (id: string | null) => void;
  remove: (id: string) => void;
  setModel: (id: string, modelId: string) => void;
  addMessage: (id: string, message: ChatMessage) => void;
  updateMessage: (id: string, messageId: string, patch: Partial<ChatMessage>) => void;
  /** Where the model's view starts, and the summary of what's before. */
  setContext: (id: string, context: Pick<Conversation, "contextFrom" | "summary">) => void;
  setSettings: (patch: Partial<ChatSettings>) => void;
  /** Attach documents to a conversation (those already there are skipped). */
  addDocuments: (id: string, documents: DocumentInfo[]) => void;
  /** Stop showing a document to the model; its text is deleted. */
  removeDocument: (id: string, documentId: string) => void;
  /** Start another version of a reply, shown empty while it's written. */
  newVersion: (id: string, messageId: string) => void;
  /** Show another version of a reply. */
  selectVersion: (id: string, messageId: string, index: number) => void;
  /** Change a user message and drop what came after it. */
  editMessage: (id: string, messageId: string, content: string) => void;
  prompts: SavedPrompt[];
  savePrompt: (name: string, text: string) => void;
  removePrompt: (id: string) => void;
};

/** How a slice of the store changes the state (zustand's `set`). */
export type SetChat = (update: (state: ChatState) => Partial<ChatState>) => void;
