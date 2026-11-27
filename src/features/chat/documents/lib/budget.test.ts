import { describe, expect, it } from "vitest";
import { documentBudget } from "./budget";

describe("documentBudget", () => {
  it("gives documents what the conversation leaves, within bounds", () => {
    // 4096 window, 1024 kept for the reply: 3072 of room.
    expect(documentBudget(4096, 200, 1024)).toBe(Math.floor(3072 * 0.65));
    expect(documentBudget(4096, 2500, 1024)).toBe(Math.floor(3072 * 0.35));
    expect(documentBudget(4096, 1500, 1024)).toBe(3072 - 1500);
    expect(documentBudget(null, 0, 1024)).toBe(2048);
  });
});
