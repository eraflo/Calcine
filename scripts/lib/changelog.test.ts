import { describe, expect, it } from "vitest";
import { changelogSection, parseStable, userParagraph } from "./changelog";

const TRAILER = "\n\nCo-Authored-By: Someone <someone@example.com>";

describe("parseStable", () => {
  it("accepts stable versions only", () => {
    expect(parseStable("1.0.0")).toEqual([1, 0, 0]);
    expect(parseStable("1.0.0-beta.3")).toBeNull();
    expect(parseStable("v1.0.0")).toBeNull();
  });
});

describe("userParagraph", () => {
  it("takes the first paragraph, not trailers", () => {
    expect(userParagraph(`For users.\n\nFor developers.${TRAILER}`)).toBe("For users.");
    expect(userParagraph(TRAILER)).toBeNull();
    expect(userParagraph(undefined)).toBeNull();
  });
});

describe("changelogSection", () => {
  it("writes entries from the body, grouped by scope", () => {
    const section = changelogSection("1.2.0", "2026-11-27", [
      {
        hash: "aaaaaaa111",
        subject: "feat(chat): images for vision models",
        body: `Drop an image in the chat\nto ask about it.\n\nThe gateway resizes it.${TRAILER}`,
      },
      { hash: "bbbbbbb222", subject: "fix: request log shows stream errors" },
      { hash: "ccccccc333", subject: "chore: bump tooling" },
      { hash: "ddddddd444", subject: "feat!: new API key format" },
      { hash: "eeeeeee555", subject: "chore(runtime): pin GenieX v0.12.0" },
      { hash: "fffffff666", subject: "refactor(chat): split the store" },
    ]);
    expect(section).toBe(
      [
        "## 1.2.0 (2026-11-27)",
        "",
        "### Breaking changes",
        "",
        "- New API key format (ddddddd)",
        "",
        "### New",
        "",
        "- New API key format (ddddddd)",
        "",
        "### Chat",
        "",
        "- **Images for vision models.** Drop an image in the chat to ask about it. (aaaaaaa)",
        "",
        "### Fixes",
        "",
        "- Request log shows stream errors (bbbbbbb)",
        "",
        "### Changes",
        "",
        "- Pin GenieX v0.12.0 (eeeeeee)",
        "",
      ].join("\n"),
    );
  });

  it("folds fixes of a feature released with them", () => {
    const section = changelogSection("1.2.0", "2026-11-27", [
      { hash: "bbbbbbb222", subject: "fix(activity): chart hover" },
      { hash: "aaaaaaa111", subject: "feat(activity): activity dashboard" },
      { hash: "ccccccc333", subject: "fix(chat): long replies scroll" },
    ]);
    expect(section).toContain("- Activity dashboard (aaaaaaa, bbbbbbb)");
    expect(section).toContain("### Fixes\n\n- Long replies scroll (ccccccc)");
    expect(section).not.toContain("Chart hover");

    // A feature without a scope: its title says what it is.
    const unscoped = changelogSection("1.2.0", "2026-11-27", [
      { hash: "bbbbbbb222", subject: "fix(activity): chart hover" },
      { hash: "aaaaaaa111", subject: "feat: activity dashboard" },
    ]);
    expect(unscoped).toContain("- Activity dashboard (aaaaaaa, bbbbbbb)");
    expect(unscoped).not.toContain("### Fixes");
  });

  it("keeps a list in the body as a list, and wraps long lines", () => {
    const section = changelogSection("1.2.0", "2026-11-27", [
      {
        hash: "aaaaaaa111",
        subject: "feat(chat): regenerate a reply, edit a message",
        body: "- Regenerate the last reply; earlier versions stay, browsed\n  under the reply.\n- Edit a message you sent.",
      },
      {
        hash: "bbbbbbb222",
        subject: "feat: battery saver",
        body: "On battery, chat requests that don't choose a power mode run in the one picked in Settings, for the chat and for apps.",
      },
    ]);
    expect(section).toContain(
      [
        "- **Regenerate a reply, edit a message** (aaaaaaa)",
        "  - Regenerate the last reply; earlier versions stay, browsed under the reply.",
        "  - Edit a message you sent.",
      ].join("\n"),
    );
    for (const line of section.split("\n")) expect(line.length).toBeLessThanOrEqual(94);
    expect(section).toContain(
      "- **Battery saver.** On battery, chat requests that don't choose a power mode run in the one\n  picked",
    );
  });
});
