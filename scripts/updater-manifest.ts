/**
 * Write the manifest the Tauri updater reads (`latest.json`), from the
 * signed packages of a release build.
 *
 *   bun scripts/updater-manifest.ts <version> <out.json> [notes-file] [linux-dir]
 *
 * The Windows installer and its `.sig` come from `target/release/bundle/nsis/`;
 * the Linux `.deb` and its `.sig`, when built, from `linux-dir`. Download URLs
 * point at the GitHub release `v<version>`.
 */
import { readdir } from "node:fs/promises";
import { join } from "node:path";

const REPOSITORY = "eraflo/Calcine";

export type UpdaterManifest = {
  version: string;
  notes: string;
  pub_date: string;
  platforms: Record<string, { signature: string; url: string }>;
};

/** A signed package, and the updater keys that should install it. */
export type Package = { file: string; signature: string; platforms: string[] };

// The updater looks up `<os>-<arch>-<package>` first, then `<os>-<arch>`.
export const WINDOWS = ["windows-aarch64-nsis", "windows-aarch64"];
export const LINUX = ["linux-aarch64-deb", "linux-aarch64"];

export function manifest(
  version: string,
  packages: Package[],
  notes: string,
  now: Date = new Date(),
): UpdaterManifest {
  const platforms: UpdaterManifest["platforms"] = {};
  for (const { file, signature, platforms: keys } of packages) {
    const url = `https://github.com/${REPOSITORY}/releases/download/v${version}/${encodeURIComponent(file)}`;
    for (const key of keys) platforms[key] = { signature: signature.trim(), url };
  }
  return { version, notes, pub_date: now.toISOString(), platforms };
}

async function signed(dir: string, matches: (file: string) => boolean, platforms: string[]) {
  const file = (await readdir(dir)).find(matches);
  if (!file) throw new Error(`no package in ${dir}`);
  const signature = await Bun.file(join(dir, `${file}.sig`)).text();
  return { file, signature, platforms };
}

if (import.meta.main) {
  const [version, out, notesFile, linuxDir] = process.argv.slice(2);
  if (!version || !out) {
    console.error(
      "usage: bun scripts/updater-manifest.ts <version> <out.json> [notes-file] [linux-dir]",
    );
    process.exit(1);
  }
  const nsis = join(import.meta.dir, "..", "target", "release", "bundle", "nsis");
  const packages = [
    await signed(nsis, (file) => file.endsWith("-setup.exe") && file.includes(version), WINDOWS),
  ];
  if (linuxDir) packages.push(await signed(linuxDir, (file) => file.endsWith(".deb"), LINUX));
  const notes = notesFile ? await Bun.file(notesFile).text() : "";
  await Bun.write(out, `${JSON.stringify(manifest(version, packages, notes), null, 2)}\n`);
  console.log(`${out}: Calcine ${version} → ${packages.map((p) => p.file).join(", ")}`);
}
