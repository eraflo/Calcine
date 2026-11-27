import { describe, expect, it } from "vitest";
import type { DocumentContext } from "@/lib/api";
import { documentParts, documentQueries, withDocuments, withPassages } from "./prompt";

const context = (mode: DocumentContext["mode"]): DocumentContext => ({
  mode,
  text: "DOCS",
  tokens: 10,
  sources: [{ documentId: "a", name: "a.pdf", kind: "pdf", page: 1, excerpt: "…" }],
  passagesRead: 1,
  passagesTotal: 4,
});

describe("prompt", () => {
  it("searches with the message, then the user's previous ones", () => {
    const earlier = [
      { role: "user", content: "first" },
      { role: "assistant", content: "answer" },
      { role: "user", content: "second" },
      { role: "user", content: "  " },
    ];
    expect(documentQueries("third", earlier)).toEqual(["third", "second", "first"]);
  });

  it("puts whole documents in the system prompt and passages before the message", () => {
    expect(documentParts(context("full"))).toEqual({ system: "DOCS" });
    expect(documentParts(context("passages"))).toEqual({ prefix: "DOCS" });
    expect(documentParts({ ...context("passages"), sources: [] })).toEqual({});
    expect(documentParts(null)).toEqual({});
  });

  it("joins documents and passages to the prompt and the message", () => {
    expect(withDocuments("Be brief.", "DOCS")).toBe("Be brief.\n\nDOCS");
    expect(withDocuments("", "DOCS")).toBe("DOCS");
    expect(withDocuments(" Be brief. ", undefined)).toBe("Be brief.");
    expect(withPassages("Question?", "PASSAGES\n")).toBe("PASSAGES\n\nQuestion?");
    expect(withPassages("Question?", undefined)).toBe("Question?");
  });
});
