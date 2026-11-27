import { describe, expect, it } from "vitest";
import {
  type DocumentContext,
  type DocumentInfo,
  documentBudget,
  documentParts,
  documentQueries,
  extensionOf,
  fileName,
  groupSources,
  isImagePath,
  newDocuments,
  type Source,
  withDocuments,
  withPassages,
} from "./documents";

const source = (documentId: string, page: number | null): Source => ({
  documentId,
  name: `${documentId}.pdf`,
  kind: "pdf",
  page,
  excerpt: "…",
});

const context = (mode: DocumentContext["mode"]): DocumentContext => ({
  mode,
  text: "DOCS",
  tokens: 10,
  sources: [source("a", 1)],
  passagesRead: 1,
  passagesTotal: 4,
});

describe("documents", () => {
  it("reads file names and extensions from Windows and Unix paths", () => {
    expect(fileName("C:\\Users\\me\\Report 2027.PDF")).toBe("Report 2027.PDF");
    expect(extensionOf("C:\\Users\\me\\Report 2027.PDF")).toBe("pdf");
    expect(extensionOf("/home/me/.bashrc")).toBe("");
    expect(isImagePath("/tmp/photo.JPEG")).toBe(true);
    expect(isImagePath("/tmp/notes.md")).toBe(false);
  });

  it("gives documents what the conversation leaves, within bounds", () => {
    // 4096 window, 1024 kept for the reply: 3072 of room.
    expect(documentBudget(4096, 200, 1024)).toBe(Math.floor(3072 * 0.65));
    expect(documentBudget(4096, 2500, 1024)).toBe(Math.floor(3072 * 0.35));
    expect(documentBudget(4096, 1500, 1024)).toBe(3072 - 1500);
    expect(documentBudget(null, 0, 1024)).toBe(2048);
  });

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
    expect(withDocuments("Be brief.", "DOCS")).toBe("Be brief.\n\nDOCS");
    expect(withDocuments("", "DOCS")).toBe("DOCS");
    expect(withDocuments(" Be brief. ", undefined)).toBe("Be brief.");
    expect(withPassages("Question?", "PASSAGES\n")).toBe("PASSAGES\n\nQuestion?");
    expect(withPassages("Question?", undefined)).toBe("Question?");
  });

  it("skips documents already attached", () => {
    const doc = (id: string) => ({ id }) as DocumentInfo;
    expect(newDocuments([doc("a")], [doc("a"), doc("b")]).map((d) => d.id)).toEqual(["b"]);
  });

  it("groups sources by document, pages in order", () => {
    const groups = groupSources([source("a", 3), source("b", null), source("a", 1)]);
    expect(groups.map((group) => group.name)).toEqual(["a.pdf", "b.pdf"]);
    expect(groups[0]?.sources.map((s) => s.page)).toEqual([1, 3]);
  });
});
