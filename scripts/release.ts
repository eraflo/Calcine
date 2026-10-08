/**
 * Prepare a stable release on `dev`: bump the version everywhere, refresh
 * Cargo.lock, add the changelog section, and commit.
 *
 *   bun scripts/release.ts 1.0.0
 *
 * Then open a pull request from `dev` to `main`. When it's merged,
 * `.github/workflows/release.yml` tags `v1.0.0` and publishes the signed
 * installer. See docs/RELEASING.md.
 */
import { $ } from "bun";
import { changelogSection, parseStable } from "./lib/changelog";
import { currentVersion, stamp } from "./version";

if (import.meta.main) {
  const version = process.argv[2] ?? "";
  if (!parseStable(version)) {
    console.error("usage: bun scripts/release.ts <major.minor.patch>");
    process.exit(1);
  }
  const branch = (await $`git rev-parse --abbrev-ref HEAD`.text()).trim();
  if (branch !== "dev") throw new Error(`prepare releases on dev, not ${branch}`);
  if ((await $`git status --porcelain`.text()).trim()) {
    throw new Error("commit or stash your changes first");
  }

  const previous = await currentVersion();
  const lastTag = (
    await $`git describe --tags --abbrev=0 --match v[0-9]* --exclude *-*`.nothrow().text()
  ).trim();
  const range = lastTag ? `${lastTag}..HEAD` : "HEAD";
  const log = await $`git log ${range} --format=%H%x09%s --no-merges`.text();
  const commits = log
    .split("\n")
    .filter(Boolean)
    .map((line) => {
      const [hash = "", subject = ""] = line.split("\t");
      return { hash, subject };
    });

  await stamp(version);
  await $`cargo update --workspace --offline`.quiet();

  const date = new Date().toISOString().slice(0, 10);
  const section = changelogSection(version, date, commits);
  const changelog = Bun.file("CHANGELOG.md");
  const existing = (await changelog.exists()) ? await changelog.text() : "# Changelog\n";
  const [title, ...rest] = existing.split("\n");
  await Bun.write("CHANGELOG.md", `${title}\n\n${section}\n${rest.join("\n").trimStart()}`);

  await $`git add Cargo.toml Cargo.lock package.json CHANGELOG.md`;
  await $`git commit -m ${`chore(release): v${version}`}`;
  console.log(`Prepared v${version} (was ${previous}). Push dev and open a PR to main.`);
}
