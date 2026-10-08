/** Pure helpers for scripts/release.ts (tested without Bun). */

export type Commit = { hash: string; subject: string };

const SECTIONS: [type: string, title: string][] = [
  ["feat", "Features"],
  ["fix", "Fixes"],
  ["perf", "Performance"],
  ["security", "Security"],
];

/** `x.y.z` as numbers, or `null` for anything else (pre-releases included). */
export function parseStable(version: string): [number, number, number] | null {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version.trim());
  return match ? [Number(match[1]), Number(match[2]), Number(match[3])] : null;
}

/** Markdown for the changelog, from Conventional Commit subjects. */
export function changelogSection(version: string, date: string, commits: Commit[]): string {
  const lines = [`## ${version} (${date})`, ""];
  const breaking = commits.filter((commit) => /^\w+(\(.+\))?!:/.test(commit.subject));
  if (breaking.length > 0) {
    lines.push("### Breaking changes", "", ...breaking.map(entry), "");
  }
  for (const [type, title] of SECTIONS) {
    const matching = commits.filter((commit) =>
      new RegExp(`^${type}(\\(.+\\))?!?:`).test(commit.subject),
    );
    if (matching.length > 0) lines.push(`### ${title}`, "", ...matching.map(entry), "");
  }
  return `${lines.join("\n").trimEnd()}\n`;
}

function entry({ hash, subject }: Commit): string {
  const text = subject.replace(/^\w+(\((.+)\))?!?:\s*/, (_, __, scope) =>
    scope ? `**${scope}:** ` : "",
  );
  return `- ${text} (${hash.slice(0, 7)})`;
}
