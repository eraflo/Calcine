import { describe, expect, it } from "vitest";
import { searchConversations } from "./search";

const conversation = (id: string, title: string, updatedAt: number, ...contents: string[]) => ({
  id,
  title,
  updatedAt,
  messages: contents.map((content, index) => ({ id: `${id}-${index}`, content })),
});

const conversations = [
  conversation("a", "Recette de crêpes", 1, "Quels ingrédients ?", "De la farine, des œufs…"),
  conversation(
    "b",
    "NPU",
    3,
    "Explique ce qu'est un NPU.",
    "Un NPU est une puce dédiée à l'inférence des réseaux de neurones, très économe en énergie.",
  ),
  conversation("c", "Voyage", 2, "Je pars à Montréal en été, que prévoir pour la météo ?"),
];

describe("searchConversations", () => {
  it("finds every word, without case or accents", () => {
    const hits = searchConversations(conversations, "econome ENERGIE");
    expect(hits.map((hit) => hit.conversationId)).toEqual(["b"]);
    expect(hits[0]?.messageId).toBe("b-1");
    const [hit] = hits;
    const marked = hit?.marks.map(([start, end]) => hit.snippet.slice(start, end));
    expect(marked).toEqual(["économe", "énergie"]);
  });

  it("puts title matches first, then recent ones", () => {
    const hits = searchConversations(
      [...conversations, conversation("d", "Autre", 9, "Une crêpe ?")],
      "crepe",
    );
    expect(hits.map((hit) => [hit.conversationId, hit.messageId])).toEqual([
      ["a", null],
      ["d", "d-0"],
    ]);
  });

  it("keeps a short excerpt around the match", () => {
    const long = `${"Avant. ".repeat(40)}Le mot Montréal est ici. ${"Après. ".repeat(40)}`;
    const [hit] = searchConversations([conversation("e", "", 1, long)], "montreal");
    expect(hit?.snippet.startsWith("…")).toBe(true);
    expect(hit?.snippet.endsWith("…")).toBe(true);
    expect(hit?.snippet).toContain("Montréal");
    expect(hit?.snippet.length).toBeLessThan(160);
  });

  it("copes with messages without text", () => {
    const old = { id: "f", title: "Vieille", updatedAt: 1, messages: [{ id: "f-0" }] };
    expect(searchConversations([old, ...conversations], "vieille")).toHaveLength(1);
    expect(searchConversations([old, ...conversations], "npu")).toHaveLength(1);
  });

  it("needs two letters", () => {
    expect(searchConversations(conversations, "e")).toEqual([]);
    expect(searchConversations(conversations, "  ")).toEqual([]);
  });
});
