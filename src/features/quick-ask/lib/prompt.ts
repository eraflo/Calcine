import { type ChatSettings, withDefaults } from "@/features/chat/lib/request";

/** One-click requests about the copied text. */
export const QUICK_ACTIONS = ["summarize", "translate", "fix", "explain"] as const;
export type QuickAction = (typeof QUICK_ACTIONS)[number];

/** What's sent: the question, then the copied text when it's included. */
export function quickPrompt(question: string, copied: string | null): string {
  const asked = question.trim();
  if (!copied) return asked;
  return `${asked}\n\n"""\n${copied}\n"""`;
}

/** The chat's settings and model, as the main window last saved them. The
 * quick window reads them without loading the chat (which it must not write). */
export function chatPreferences(): { settings: ChatSettings; modelId: string | null } {
  try {
    const saved = JSON.parse(localStorage.getItem("calcine.chat") ?? "null")?.state;
    const active = saved?.conversations?.find(
      (conversation: { id: string }) => conversation.id === saved.activeId,
    );
    return {
      settings: withDefaults(saved?.settings),
      modelId: active?.modelId ?? saved?.lastModelId ?? null,
    };
  } catch {
    return { settings: withDefaults(undefined), modelId: null };
  }
}

/** Quick answers come fast: no thinking, no structured output. */
export function quickSettings(settings: ChatSettings): ChatSettings {
  return { ...settings, think: false, outputFormat: "text" };
}
