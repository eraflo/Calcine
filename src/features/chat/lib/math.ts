/**
 * Formulas as models write them, for remark-math: `\[…\]` and `$$…$$` on
 * their own lines are display math; `\(…\)` and `$…$` are inline. Inline
 * dollars become `$$…$$` (remark-math's inline form when single dollars are
 * off), so prices like "$5 and $10" stay text: a formula doesn't start with
 * a digit or a space, nor end with a space. Code is left alone.
 */
export function withMath(text: string): string {
  if (!/[$\\]/.test(text)) return text;
  return text
    .split(CODE)
    .map((part, index) => (index % 2 === 1 ? part : formulas(part)))
    .join("");
}

/** Fenced blocks and inline code, kept as they are (captured by `split`). */
const CODE = /(```[\s\S]*?(?:```|$)|`[^`\n]*`)/;

function formulas(text: string): string {
  return text
    .replace(/\\\[([\s\S]+?)\\\]/g, (_, math: string) => `\n$$\n${math.trim()}\n$$\n`)
    .replace(/\\\((.+?)\\\)/g, (_, math: string) => `$$${math.trim()}$$`)
    .replace(/(?<![\\$\w])\$(?![\s\d$])([^$\n]*?[^\s$\\])\$(?![\w$])/g, "$$$$$1$$$$");
}
