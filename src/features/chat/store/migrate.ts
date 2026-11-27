/** Chat state saved by earlier versions, brought up to date. */

import { DEFAULT_SETTINGS, POWER_MODES, withDefaults } from "../lib/request";
import type { ChatState } from "./types";

/** Bump when the saved shape changes, and handle the old one here. */
export const CHAT_VERSION = 5;

export function migrateChat(persisted: unknown, version: number): ChatState {
  const state = persisted as Pick<ChatState, "conversations" | "settings" | "prompts">;
  if (version < 2) {
    // Titles are shown translated when empty; power modes lost their labels.
    state.conversations = state.conversations.map((c) =>
      c.title === "New chat" ? { ...c, title: "" } : c,
    );
    if (!POWER_MODES.includes(state.settings.powerMode)) {
      state.settings = { ...state.settings, powerMode: DEFAULT_SETTINGS.powerMode };
    }
  }
  // New settings get their defaults.
  state.settings = withDefaults(state.settings);
  state.prompts ??= [];
  return state as ChatState;
}
