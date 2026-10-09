import { describe, expect, it } from "vitest";
import { contextStart, fillLevel, replyReserve, totalTokens } from "./context";

const roles = ["user", "assistant", "user", "assistant", "user", "assistant", "user"];
const tokens = [100, 100, 100, 100, 100, 100, 50];

describe("contextStart", () => {
  it("keeps everything that fits", () => {
    expect(contextStart(tokens, roles, 20, 1000)).toBe(0);
    expect(contextStart(tokens, roles, 20, 670)).toBe(0);
  });

  it("cuts down to three quarters of the room, at a user turn", () => {
    // Three quarters of 600 is 450: from turn 2, 20 + 450 doesn't fit it.
    expect(contextStart(tokens, roles, 20, 600)).toBe(4);
    // Three quarters of 300 is 225: only the last turn fits it.
    expect(contextStart(tokens, roles, 20, 300)).toBe(6);
  });

  it("falls back to the whole room, then gives up", () => {
    // Three quarters of 80 is 60: the last turn (20 + 50) only fits 80.
    expect(contextStart(tokens, roles, 20, 80)).toBe(6);
    expect(contextStart(tokens, roles, 20, 60)).toBeNull();
  });
});

describe("helpers", () => {
  it("adds up a count and reserves room for the reply", () => {
    expect(totalTokens({ perMessage: [10, 20], overhead: 5, exact: true })).toBe(35);
    expect(replyReserve(1024, 4096)).toBe(1024);
    expect(replyReserve(8192, 4096)).toBe(2048);
  });

  it("grades how full the context is", () => {
    expect(fillLevel(1000, 4096)).toBe("ok");
    expect(fillLevel(3100, 4096)).toBe("high");
    expect(fillLevel(4000, 4096)).toBe("full");
  });
});
