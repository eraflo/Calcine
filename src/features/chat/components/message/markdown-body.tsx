import "katex/dist/katex.min.css";
import { memo } from "react";
import Markdown, { type Components, type Options } from "react-markdown";
import rehypeKatex from "rehype-katex";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import { withMath } from "../../lib/math";
import { prettyJson } from "../../lib/request";
import { MermaidDiagram } from "./mermaid-diagram";

/** JSON replies (structured output) show as a formatted code block. */
export function asMarkdown(content: string): string {
  const json = prettyJson(content);
  return json === null ? content : `\`\`\`json\n${json}\n\`\`\``;
}

const remarkPlugins: Options["remarkPlugins"] = [
  remarkGfm,
  [remarkMath, { singleDollarTextMath: false }],
];
// A formula with a mistake shows in red instead of breaking the reply.
const rehypePlugins: Options["rehypePlugins"] = [
  [rehypeKatex, { throwOnError: false, strict: "ignore" }],
];

const components: Components = {
  pre({ node, children, ...props }) {
    const code = node?.children[0];
    if (
      code?.type === "element" &&
      code.tagName === "code" &&
      String(code.properties.className ?? "").includes("language-mermaid")
    ) {
      const text = code.children.map((child) => ("value" in child ? child.value : "")).join("");
      return <MermaidDiagram code={text.trimEnd()} />;
    }
    return <pre {...props}>{children}</pre>;
  },
};

/** A reply in Markdown, with formulas (KaTeX) and Mermaid diagrams. */
export const MarkdownBody = memo(function MarkdownBody({ text }: { text: string }) {
  return (
    <div className="markdown text-sm leading-relaxed">
      <Markdown remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins} components={components}>
        {withMath(text)}
      </Markdown>
    </div>
  );
});
