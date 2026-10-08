/**
 * Pin the latest stable GenieX in `runtime/geniex.json` (the version bundled
 * in Calcine's installer), from Qualcomm's release index.
 *
 *   bun scripts/geniex-bump.ts            # update the pin if a newer stable exists
 *
 * Prints the new version, or nothing when the pin is current. The download
 * is verified against the manifest's SHA-256 and size before pinning.
 */
import { join } from "node:path";

const ENDPOINT = "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex";
const PIN = join(import.meta.dir, "..", "runtime", "geniex.json");

type Asset = {
  name: string;
  url: string;
  size: number;
  sha256: string;
  kind: string;
  platform: string;
  arch: string;
};

const index = (await (await fetch(`${ENDPOINT}/index.json`)).json()) as { latest_stable: string };
const pin = await Bun.file(PIN).json();
const latest = index.latest_stable;
if (!latest || Bun.semver.order(latest.slice(1), pin.version.slice(1)) <= 0) process.exit(0);

const manifest = (await (await fetch(`${ENDPOINT}/manifest-${latest}.json`)).json()) as {
  assets: Asset[];
};
const asset = manifest.assets.find(
  (candidate) =>
    candidate.kind === "cli-installer" &&
    candidate.platform === "windows" &&
    candidate.arch === "arm64",
);
if (!asset) throw new Error(`${latest} has no Windows ARM64 installer`);

const bytes = new Uint8Array(await (await fetch(asset.url)).arrayBuffer());
const digest = new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
if (bytes.length !== asset.size || digest !== asset.sha256.toLowerCase()) {
  throw new Error(`${asset.name} doesn't match its manifest (size or SHA-256)`);
}
await Bun.write(join(import.meta.dir, "..", "geniex-cli-setup.exe"), bytes);

pin.version = latest;
pin.assets["windows-arm64"] = {
  name: asset.name,
  url: asset.url,
  size: asset.size,
  sha256: digest,
};
await Bun.write(PIN, `${JSON.stringify(pin, null, 2)}\n`);
console.log(latest);
