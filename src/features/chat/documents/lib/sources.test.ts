import { describe, expect, it } from "vitest";
import type { DocumentInfo, Source } from "@/lib/api";
import { groupSources, newDocuments } from "./sources";

const source = (documentId: string, page: number | null): Source => ({
  documentId,
  name: `${documentId}.pdf`,
  kind: "pdf",
  page,
  excerpt: "…",
});

describe("sources", () => {
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
