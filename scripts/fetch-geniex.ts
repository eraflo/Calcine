/**
 * Download the pinned GenieX installer into `src-tauri/resources/geniex/` so
 * release builds can bundle it (see `src-tauri/tauri.release.conf.json`).
 *
 *   bun scripts/fetch-geniex.ts            # download (skipped if already verified)
 *   bun scripts/fetch-geniex.ts --check    # only verify the remote file exists and matches the pinned size
 *
 * The file is verified against the pinned SHA-256 before it is kept.
 */
import { mkdir, rename, rm } from "node:fs/promises";
import { join } from "node:path";

type Asset = { name: string; url: string; size: number; sha256: string };
type Pin = { version: string; assets: Record<string, Asset> };

const ROOT = join(import.meta.dir, "..");
const PLATFORM = "windows-arm64";
const OUTPUT_DIR = join(ROOT, "src-tauri", "resources", "geniex");
const OUTPUT_FILE = join(OUTPUT_DIR, "geniex-cli-setup.exe");

const pin: Pin = await Bun.file(join(ROOT, "runtime", "geniex.json")).json();
const asset = pin.assets[PLATFORM];
if (!asset) fail(`runtime/geniex.json has no "${PLATFORM}" asset`);

if (process.argv.includes("--check")) {
  const response = await fetch(asset.url, { method: "HEAD" });
  if (!response.ok) fail(`HEAD ${asset.url} → ${response.status}`);
  const length = Number(response.headers.get("content-length"));
  if (length !== asset.size) fail(`size mismatch: remote ${length}, pinned ${asset.size}`);
  console.log(`GenieX ${pin.version}: ${asset.name} is available (${asset.size} bytes)`);
  process.exit(0);
}

if ((await Bun.file(OUTPUT_FILE).exists()) && (await sha256(OUTPUT_FILE)) === asset.sha256) {
  console.log(`GenieX ${pin.version} already fetched and verified`);
  process.exit(0);
}

console.log(`Downloading GenieX ${pin.version} (${(asset.size / 1024 ** 2).toFixed(1)} MiB)…`);
await mkdir(OUTPUT_DIR, { recursive: true });
const partial = `${OUTPUT_FILE}.partial`;
const response = await fetch(asset.url);
if (!response.ok) fail(`GET ${asset.url} → ${response.status}`);
await Bun.write(partial, response);

const actual = await sha256(partial);
if (actual !== asset.sha256) {
  await rm(partial, { force: true });
  fail(`checksum mismatch: expected ${asset.sha256}, got ${actual}`);
}
await rename(partial, OUTPUT_FILE);
console.log(`GenieX ${pin.version} verified → ${OUTPUT_FILE}`);

async function sha256(path: string): Promise<string> {
  const hasher = new Bun.CryptoHasher("sha256");
  for await (const chunk of Bun.file(path).stream()) hasher.update(chunk);
  return hasher.digest("hex");
}

function fail(message: string): never {
  console.error(`fetch-geniex: ${message}`);
  process.exit(1);
}
