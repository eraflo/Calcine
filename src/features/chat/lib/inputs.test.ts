import { describe, expect, it } from "vitest";
import { escapeStop, parseOptional, unescapeStop } from "./inputs";

describe("parseOptional", () => {
  it("treats an empty field as the default", () => {
    expect(parseOptional("", 0, 1, false)).toBeNull();
    expect(parseOptional("  ", 0, 1, false)).toBeNull();
  });

  it("clamps to the range and rounds integers", () => {
    expect(parseOptional("1.5", 0, 1, false)).toBe(1);
    expect(parseOptional("-3", -1, 999, true)).toBe(-1);
    expect(parseOptional("40.6", 0, 1000, true)).toBe(41);
    expect(parseOptional("0.25", 0, 1, false)).toBe(0.25);
  });
});

describe("stop sequences", () => {
  it("turns \\n and \\t into line breaks and tabs, and back", () => {
    expect(unescapeStop("\\n\\nUser:")).toBe("\n\nUser:");
    expect(escapeStop("\n\tEnd")).toBe("\\n\\tEnd");
  });
});
