import { memo } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { prettyJson } from "../../lib/request";

/** JSON replies (structured output) show as a formatted code block. */
export function asMarkdown(content: string): string {
  const json = prettyJson(content);
  return json === null ? content : `\`\`\`json\n${json}\n\`\`\``;
}

export const MarkdownBody = memo(function MarkdownBody({ text }: { text: string }) {
  return (
    <div className="markdown text-sm leading-relaxed">
      <Markdown remarkPlugins={[remarkGfm]}>{text}</Markdown>
    </div>
  );
});
