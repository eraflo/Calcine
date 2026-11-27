import { describe, expect, it } from "vitest";
import type { Conversation } from "../store";
import { type ExportLabels, toMarkdown } from "./export";

const labels: ExportLabels = {
  title: "Colors",
  you: "You",
  model: "Model",
  reasoning: "Reasoning",
  exported: "exported",
  image: "Image",
  document: "Document",
  sources: "Sources",
  page: "p. {page}",
  recording: "Recording",
  stopped: "Stopped",
};

const conversation: Conversation = {
  id: "c",
  title: "Colors",
  modelId: "qualcomm/Qwen3-4B",
  updatedAt: 0,
  messages: [
    {
      id: "1",
      role: "user",
      content: "What color is this?",
      attachments: [{ id: "a", kind: "image", name: "photo.jpg" }],
    },
    { id: "2", role: "assistant", content: "Red.", reasoning: "It looks red." },
    { id: "3", role: "assistant", content: "", error: "failed" },
    { id: "4", role: "assistant", content: "Partial", stopped: true },
  ],
};

describe("toMarkdown", () => {
  it("writes one section per message, with reasoning folded", () => {
    expect(toMarkdown(conversation, labels, new Date("2026-10-08T12:00:00Z"))).toBe(
      [
        "# Colors",
        "",
        "Model: `qualcomm/Qwen3-4B` · exported 2026-10-08",
        "",
        "## You",
        "",
        "*Image: photo.jpg*",
        "",
        "What color is this?",
        "",
        "## qualcomm/Qwen3-4B",
        "",
        "<details>",
        "<summary>Reasoning</summary>",
        "",
        "It looks red.",
        "",
        "</details>",
        "",
        "Red.",
        "",
        "## qualcomm/Qwen3-4B",
        "",
        "Partial",
        "",
        "*Stopped*",
        "",
      ].join("\n"),
    );
  });

  it("names attached documents and the pages a reply came from", () => {
    const document = {
      id: "d",
      name: "report.pdf",
      kind: "pdf" as const,
      pages: 12,
      chars: 1000,
      bytes: 2000,
      passages: 30,
    };
    const source = (page: number | null) => ({
      documentId: "d",
      name: "report.pdf",
      kind: "pdf" as const,
      page,
      excerpt: "…",
    });
    const markdown = toMarkdown(
      {
        ...conversation,
        messages: [
          { id: "1", role: "user", content: "Budget?", documents: [document] },
          {
            id: "2",
            role: "assistant",
            content: "4 M€.",
            sources: [source(7), source(2)],
            documentMode: "passages",
          },
        ],
      },
      labels,
      new Date("2026-10-08T12:00:00Z"),
    );
    expect(markdown).toContain("*Document: report.pdf*\n\nBudget?");
    expect(markdown).toContain("4 M€.\n\n*Sources: report.pdf (p. 2, p. 7)*");
  });
});
