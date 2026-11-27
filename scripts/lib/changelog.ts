/** Pure helpers for scripts/release.ts (tested without Bun). */

export type Commit = { hash: string; subject: string; body?: string };

/**
 * Changelog headings for feature scopes. A feature without a scope goes
 * under "New"; an unknown scope is capitalized.
 */
const SCOPE_TITLES: Record<string, string> = {
  activity: "Activity",
  api: "Local API",
  app: "App",
  chat: "Chat",
  cli: "Command line",
  discover: "Discover",
  gateway: "Local API",
  hardware: "Hardware",
  models: "Models",
  network: "Local network",
  runtime: "GenieX",
  server: "Server",
  settings: "Settings",
  ui: "Interface",
};

/** Sections after the features, in order: `[type, scopes or "any", title]`. */
const OTHER_SECTIONS: [type: string, scopes: readonly string[] | "any", title: string][] = [
  ["fix", "any", "Fixes"],
  ["perf", "any", "Performance"],
  ["security", "any", "Security"],
  // GenieX pins and dependency updates change what users get.
  ["chore", ["runtime", "deps"], "Changes"],
];

/** Lines are wrapped here, as in the hand-written sections. */
const WIDTH = 94;

type Parsed = Commit & { type: string; scope?: string; breaking: boolean; title: string };

/** `x.y.z` as numbers, or `null` for anything else (pre-releases included). */
export function parseStable(version: string): [number, number, number] | null {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version.trim());
  return match ? [Number(match[1]), Number(match[2]), Number(match[3])] : null;
}

/**
 * Markdown for the changelog, from Conventional Commits. An entry is the
 * subject, then the first paragraph of the body, written for users (later
 * paragraphs are for developers). Features are grouped by scope; a fix
 * with the scope of a feature of the same release (or, for a feature
 * without one, a word of its title) fixes that unreleased feature, so it
 * only adds its hash to the feature's entry.
 */
export function changelogSection(version: string, date: string, commits: Commit[]): string {
  const parsed = commits.map(parse).filter((commit): commit is Parsed => commit !== null);
  const features = parsed.filter((commit) => commit.type === "feat");
  const folded = new Map<Parsed, string[]>(features.map((feature) => [feature, []]));
  const shown = parsed.filter((commit) => {
    if (commit.type !== "fix" || !commit.scope) return true;
    const scope = commit.scope;
    const feature = features.find((candidate) =>
      candidate.scope
        ? candidate.scope === scope
        : new RegExp(`\\b${scope}\\b`, "i").test(candidate.title),
    );
    if (!feature) return true;
    folded.get(feature)?.push(commit.hash);
    return false;
  });
  const entry = (commit: Parsed) => render(commit, folded.get(commit) ?? []);

  const lines = [`## ${version} (${date})`, ""];
  const section = (title: string, entries: Parsed[]) => {
    if (entries.length > 0) lines.push(`### ${title}`, "", ...entries.map(entry), "");
  };
  section(
    "Breaking changes",
    shown.filter((commit) => commit.breaking),
  );
  const groups = new Map<string, Parsed[]>();
  for (const feature of shown.filter((commit) => commit.type === "feat")) {
    const title = feature.scope ? scopeTitle(feature.scope) : "New";
    groups.set(title, [...(groups.get(title) ?? []), feature]);
  }
  const titles = [...groups.keys()].sort((a, b) => Number(b === "New") - Number(a === "New"));
  for (const title of titles) section(title, groups.get(title) ?? []);
  for (const [type, scopes, title] of OTHER_SECTIONS) {
    section(
      title,
      shown.filter(
        (commit) =>
          commit.type === type &&
          (scopes === "any" || (commit.scope !== undefined && scopes.includes(commit.scope))),
      ),
    );
  }
  return `${lines.join("\n").trimEnd()}\n`;
}

function parse(commit: Commit): Parsed | null {
  const match = /^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$/.exec(commit.subject.trim());
  if (!match) return null;
  const [, type = "", scope, bang, title = ""] = match;
  const breaking = Boolean(bang) || /^BREAKING[ -]CHANGE:/m.test(commit.body ?? "");
  return { ...commit, type, ...(scope ? { scope } : {}), breaking, title };
}

function scopeTitle(scope: string): string {
  return SCOPE_TITLES[scope] ?? scope.charAt(0).toUpperCase() + scope.slice(1);
}

/** The body's first paragraph, unless it's only trailers (`Co-Authored-By: …`). */
export function userParagraph(body: string | undefined): string | null {
  const paragraphs = (body ?? "")
    .split(/\n\s*\n/)
    .map((paragraph) => paragraph.trim())
    .filter(Boolean);
  const first = paragraphs[0];
  if (!first) return null;
  const trailers = first.split("\n").every((line) => /^[A-Za-z][\w-]*: \S/.test(line.trim()));
  return trailers ? null : first;
}

function render(commit: Parsed, fixes: string[]): string {
  const hashes = [commit.hash, ...fixes].map((hash) => hash.slice(0, 7)).join(", ");
  const title = commit.title.charAt(0).toUpperCase() + commit.title.slice(1);
  const paragraph = userParagraph(commit.body);
  if (!paragraph) return wrap(`- ${title} (${hashes})`);
  const lines = paragraph.split("\n").map((line) => line.trim());
  // A list in the body becomes a list under the entry.
  if (lines.every((line) => /^[-*] |^ {0,2}\S/.test(line)) && lines[0]?.startsWith("- ")) {
    const items = paragraph
      .split(/\n(?=\s*[-*] )/)
      .map((item) => item.replace(/^\s*[-*] /, "").replace(/\s*\n\s*/g, " "));
    return [
      wrap(`- **${title}** (${hashes})`),
      ...items.map((item) => wrap(`  - ${item}`, "    ")),
    ].join("\n");
  }
  return wrap(`- **${title}.** ${lines.join(" ")} (${hashes})`);
}

/** Wrap at {@link WIDTH}, continuing lines under the text. */
function wrap(text: string, indent = "  "): string {
  const lines: string[] = [];
  const lead = /^ */.exec(text)?.[0] ?? "";
  let line = lead;
  for (const word of text.slice(lead.length).split(" ")) {
    if (line.trim() && line.length + 1 + word.length > WIDTH) {
      lines.push(line);
      line = indent + word;
    } else {
      line = line.trim() ? `${line} ${word}` : line + word;
    }
  }
  if (line) lines.push(line);
  return lines.join("\n");
}
