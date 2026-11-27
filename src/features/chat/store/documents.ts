/** Documents attached to conversations. */

import { isTauri } from "@tauri-apps/api/core";
import { commands } from "@/lib/api";
import { newDocuments } from "../documents/lib/sources";
import type { ChatState, SetChat } from "./types";
import { mapConversation } from "./update";

/** Delete documents' text from disk (outside Tauri, there's none). */
export function forgetDocuments(ids: string[]) {
  if (ids.length === 0 || !isTauri()) return;
  void commands.removeDocuments(ids);
}

export function documentActions(set: SetChat): Pick<ChatState, "addDocuments" | "removeDocument"> {
  return {
    addDocuments: (id, documents) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({
          ...c,
          documents: [...(c.documents ?? []), ...newDocuments(c.documents ?? [], documents)],
        })),
      })),
    removeDocument: (id, documentId) => {
      forgetDocuments([documentId]);
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({
          ...c,
          documents: c.documents?.filter((document) => document.id !== documentId),
        })),
      }));
    },
  };
}
