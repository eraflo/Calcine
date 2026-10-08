import type { Conversation } from "../store";

/** Words of the export, in the UI language. */
export type ExportLabels = {
  title: string;
  you: string;
  model: string;
  reasoning: string;
  exported: string;
  image: string;
  recording: string;
  stopped: string;
};

/**
 * A conversation as Markdown: one section per message, reasoning folded in a
 * `<details>` block, attachments named (their data isn't kept).
 */
export function toMarkdown(conversation: Conversation, labels: ExportLabels, date: Date): string {
  const lines = [
    `# ${labels.title}`,
    "",
    `${labels.model}: \`${conversation.modelId}\` · ${labels.exported} ${date.toISOString().slice(0, 10)}`,
  ];
  for (const message of conversation.messages) {
    if (!message.content && !message.attachments?.length && !message.reasoning) continue;
    lines.push("", `## ${message.role === "user" ? labels.you : conversation.modelId}`, "");
    for (const attachment of message.attachments ?? []) {
      const kind = attachment.kind === "image" ? labels.image : labels.recording;
      lines.push(`*${kind}: ${attachment.name}*`, "");
    }
    if (message.reasoning) {
      lines.push(
        "<details>",
        `<summary>${labels.reasoning}</summary>`,
        "",
        message.reasoning.trim(),
        "",
        "</details>",
        "",
      );
    }
    if (message.content) lines.push(message.content.trim());
    if (message.stopped) lines.push("", `*${labels.stopped}*`);
  }
  return `${lines.join("\n")}\n`;
}
