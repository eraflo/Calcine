import { beforeEach, describe, expect, it } from "vitest";
import { chatPreferences, quickPrompt } from "./prompt";
import { keysOf, shortcutOf } from "./shortcut";

const press = (
  code: string,
  held: Partial<Record<"ctrl" | "alt" | "shift" | "meta", boolean>>,
) => ({
  code,
  ctrlKey: held.ctrl ?? false,
  altKey: held.alt ?? false,
  shiftKey: held.shift ?? false,
  metaKey: held.meta ?? false,
});

describe("shortcutOf", () => {
  it("writes shortcuts as the shell reads them", () => {
    expect(shortcutOf(press("Space", { ctrl: true, shift: true }))).toBe("Ctrl+Shift+Space");
    expect(shortcutOf(press("KeyK", { alt: true }))).toBe("Alt+K");
    expect(shortcutOf(press("Digit1", { ctrl: true, meta: true }))).toBe("Ctrl+Super+1");
    expect(shortcutOf(press("F9", { shift: true }))).toBe("Shift+F9");
  });

  it("waits for a key, and needs a modifier", () => {
    expect(shortcutOf(press("ControlLeft", { ctrl: true }))).toBeNull();
    expect(shortcutOf(press("KeyA", {}))).toBeNull();
  });

  it("shows Super as Win", () => {
    expect(keysOf("Super+Shift+Q")).toEqual(["Win", "Shift", "Q"]);
  });
});

describe("quickPrompt", () => {
  it("quotes the copied text after the question", () => {
    expect(quickPrompt(" Summarize. ", "Some text")).toBe('Summarize.\n\n"""\nSome text\n"""');
    expect(quickPrompt("Hi", null)).toBe("Hi");
  });
});

describe("chatPreferences", () => {
  beforeEach(() => localStorage.clear());

  it("reads the chat's settings and model without its store", () => {
    localStorage.setItem(
      "calcine.chat",
      JSON.stringify({
        state: {
          settings: { temperature: 0.2 },
          activeId: "c2",
          lastModelId: "a/older",
          conversations: [{ id: "c2", modelId: "a/active" }],
        },
      }),
    );
    const preferences = chatPreferences();
    expect(preferences.modelId).toBe("a/active");
    expect(preferences.settings.temperature).toBe(0.2);
    expect(preferences.settings.maxTokens).toBeGreaterThan(0);
  });

  it("falls back to defaults", () => {
    localStorage.setItem("calcine.chat", "{not json");
    expect(chatPreferences().modelId).toBeNull();
  });
});
