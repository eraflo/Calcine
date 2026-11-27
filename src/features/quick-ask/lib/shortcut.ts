/** What a key press says, as far as shortcuts go. */
export type KeyPress = {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
};

const MODIFIER_CODES = /^(Control|Alt|Shift|Meta|OS)(Left|Right)?$/;

/**
 * The shortcut a key press makes, written as the shell reads it
 * (`Ctrl+Shift+Space`), or `null` while only modifiers are held. A global
 * shortcut needs a modifier, or it would swallow a key everywhere.
 */
export function shortcutOf(press: KeyPress): string | null {
  if (MODIFIER_CODES.test(press.code)) return null;
  const modifiers = [
    press.ctrlKey && "Ctrl",
    press.altKey && "Alt",
    press.shiftKey && "Shift",
    press.metaKey && "Super",
  ].filter(Boolean);
  if (modifiers.length === 0) return null;
  const key = press.code.replace(/^Key(?=[A-Z]$)/, "").replace(/^Digit(?=\d$)/, "");
  return [...modifiers, key].join("+");
}

/** The keys of a shortcut, to show one by one. */
export function keysOf(shortcut: string): string[] {
  return shortcut.split("+").map((key) => (key === "Super" ? "Win" : key));
}
