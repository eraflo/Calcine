/**
 * Quick questions: a small window opened by a global shortcut, over
 * whatever is in front, to ask a model something without opening Calcine.
 *
 * - `quick-ask-app`: the window's root (main.tsx picks it by window label)
 * - `use-handoff`: in Calcine's window, a quick question continued in the chat
 * - `lib/shortcut`: key presses as shortcuts, for the Settings card
 */

export { keysOf, shortcutOf } from "./lib/shortcut";
export { QuickAskApp } from "./quick-ask-app";
export { useQuickAskHandoff } from "./use-handoff";
