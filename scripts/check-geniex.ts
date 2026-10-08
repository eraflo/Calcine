/**
 * Check that the GenieX installer pinned in `runtime/geniex.json` is still
 * published with the pinned size. Calcine downloads it on first launch and
 * verifies it against the pinned SHA-256, which ships inside the app.
 *
 *   bun scripts/check-geniex.ts
 */
import { join } from "node:path";

type Asset = { name: string; url: string; size: number; sha256: string };
type Pin = { version: string; assets: Record<string, Asset> };

const PLATFORM = "windows-arm64";

const pin: Pin = await Bun.file(join(import.meta.dir, "..", "runtime", "geniex.json")).json();
const asset = pin.assets[PLATFORM];
if (!asset) fail(`runtime/geniex.json has no "${PLATFORM}" asset`);
if (!/^[0-9a-f]{64}$/.test(asset.sha256)) fail(`"${asset.sha256}" isn't a SHA-256`);

const response = await fetch(asset.url, { method: "HEAD" });
if (!response.ok) fail(`HEAD ${asset.url} → ${response.status}`);
const length = Number(response.headers.get("content-length"));
if (length !== asset.size) fail(`size mismatch: remote ${length}, pinned ${asset.size}`);
console.log(`GenieX ${pin.version}: ${asset.name} is available (${asset.size} bytes)`);

function fail(message: string): never {
  console.error(`check-geniex: ${message}`);
  process.exit(1);
}
