/**
 * Write the manifest the Tauri updater reads (`latest.json`), from the
 * signed installer of a release build.
 *
 *   bun scripts/updater-manifest.ts <version> <out.json> [notes-file]
 *
 * The installer and its `.sig` come from `target/release/bundle/nsis/`; the
 * download URL points at the GitHub release `v<version>`.
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

export function manifest(
  version: string,
  installer: string,
  signature: string,
  notes: string,
  now: Date = new Date(),
): UpdaterManifest {
  const url = `https://github.com/${REPOSITORY}/releases/download/v${version}/${encodeURIComponent(installer)}`;
  const entry = { signature: signature.trim(), url };
  return {
    version,
    notes,
    pub_date: now.toISOString(),
    // The updater looks up `<os>-<arch>-<installer>` first, then `<os>-<arch>`.
    platforms: { "windows-aarch64-nsis": entry, "windows-aarch64": entry },
  };
}

if (import.meta.main) {
  const [version, out, notesFile] = process.argv.slice(2);
  if (!version || !out) {
    console.error("usage: bun scripts/updater-manifest.ts <version> <out.json> [notes-file]");
    process.exit(1);
  }
  const bundle = join(import.meta.dir, "..", "target", "release", "bundle", "nsis");
  const files = await readdir(bundle);
  const installer = files.find((file) => file.endsWith("-setup.exe") && file.includes(version));
  if (!installer) throw new Error(`no ${version} installer in ${bundle}`);
  const signature = await Bun.file(join(bundle, `${installer}.sig`)).text();
  const notes = notesFile ? await Bun.file(notesFile).text() : "";
  await Bun.write(
    out,
    `${JSON.stringify(manifest(version, installer, signature, notes), null, 2)}\n`,
  );
  console.log(`${out}: Calcine ${version} → ${installer}`);
}
