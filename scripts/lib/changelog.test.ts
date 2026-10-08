import { describe, expect, it } from "vitest";
import { changelogSection, parseStable } from "./changelog";

describe("parseStable", () => {
  it("accepts stable versions only", () => {
    expect(parseStable("1.0.0")).toEqual([1, 0, 0]);
    expect(parseStable("1.0.0-beta.3")).toBeNull();
    expect(parseStable("v1.0.0")).toBeNull();
  });
});

describe("changelogSection", () => {
  it("groups Conventional Commits and skips chores", () => {
    const section = changelogSection("1.0.0", "2026-10-09", [
      { hash: "aaaaaaa111", subject: "feat(chat): images for vision models" },
      { hash: "bbbbbbb222", subject: "fix: request log shows stream errors" },
      { hash: "ccccccc333", subject: "chore: bump deps" },
      { hash: "ddddddd444", subject: "feat!: new API key format" },
    ]);
    expect(section).toBe(
      [
        "## 1.0.0 (2026-10-09)",
        "",
        "### Breaking changes",
        "",
        "- new API key format (ddddddd)",
        "",
        "### Features",
        "",
        "- **chat:** images for vision models (aaaaaaa)",
        "- new API key format (ddddddd)",
        "",
        "### Fixes",
        "",
        "- request log shows stream errors (bbbbbbb)",
        "",
      ].join("\n"),
    );
  });
});
