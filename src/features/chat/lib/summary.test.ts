import { describe, expect, it } from "vitest";
import { cleanSummary, summaryMessages, withSummary } from "./summary";

describe("withSummary", () => {
  it("adds the summary after the system prompt", () => {
    expect(withSummary(" Be brief. ", "We picked Rust.")).toBe(
      "Be brief.\n\nSummary of the earlier part of this conversation, which you can no longer see:\nWe picked Rust.",
    );
    expect(withSummary("", "We picked Rust.")).toMatch(/^Summary of the earlier part/);
    expect(withSummary("Be brief.", undefined)).toBe("Be brief.");
    expect(withSummary("Be brief.", "  ")).toBe("Be brief.");
  });
});

describe("summaryMessages", () => {
  const forgotten = [
    { role: "user" as const, content: "I like jazz. Reply OK." },
    { role: "assistant" as const, content: "OK" },
    { role: "assistant" as const, content: " " },
  ];

  it("sends the conversation as the chat did, then asks for the summary", () => {
    const messages = summaryMessages("Be brief.", "Alice likes tea.", forgotten);
    expect(messages[0]).toEqual({
      role: "system",
      content: withSummary("Be brief.", "Alice likes tea."),
    });
    expect(messages.slice(1, 3)).toEqual(forgotten.slice(0, 2));
    expect(messages).toHaveLength(4);
    expect(messages[3]?.role).toBe("user");
    expect(messages[3]?.content).toContain("including the summary of earlier messages");
  });

  it("leaves out an empty system prompt", () => {
    const messages = summaryMessages("", undefined, forgotten);
    expect(messages[0]?.role).toBe("user");
    expect(messages.at(-1)?.content).not.toContain("earlier messages");
  });
});

describe("cleanSummary", () => {
  it("drops headings, thinking and repeated sentences", () => {
    expect(cleanSummary("<think>hmm</think>\n**Summary:** Alice likes tea.")).toBe(
      "Alice likes tea.",
    );
    expect(
      cleanSummary("The user likes tea. They saw notes. They saw notes. They saw notes!"),
    ).toBe("The user likes tea. They saw notes.");
  });

  it("cuts a summary that ran on", () => {
    const long = Array.from({ length: 200 }, (_, index) => `Fact ${index}.`).join(" ");
    expect(cleanSummary(long).length).toBeLessThanOrEqual(1201);
  });
});
