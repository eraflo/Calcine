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
});
