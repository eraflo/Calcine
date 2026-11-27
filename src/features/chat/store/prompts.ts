/** System prompts saved for reuse. */

import { newId } from "./conversations";
import type { ChatState, SetChat } from "./types";

export function promptActions(set: SetChat): Pick<ChatState, "savePrompt" | "removePrompt"> {
  return {
    // Saving under an existing name replaces that prompt.
    savePrompt: (name, text) =>
      set((state) => ({
        prompts: [
          ...state.prompts.filter((prompt) => prompt.name !== name),
          { id: newId(), name, text },
        ],
      })),
    removePrompt: (id) =>
      set((state) => ({ prompts: state.prompts.filter((prompt) => prompt.id !== id) })),
  };
}
