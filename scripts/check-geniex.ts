/**
 * Check that the GenieX packages pinned in `runtime/geniex.json` (Windows
 * installer, Linux archive) are still published with the pinned size.
 * Calcine downloads its platform's on first launch and verifies it against
 * the pinned SHA-256, which ships inside the app.
 *
 *   bun scripts/check-geniex.ts
 */
import { join } from "node:path";

type Asset = { name: string; url: string; size: number; sha256: string };
type Pin = { version: string; assets: Record<string, Asset> };

const PLATFORMS = ["windows-arm64", "linux-arm64"];

const pin: Pin = await Bun.file(join(import.meta.dir, "..", "runtime", "geniex.json")).json();
for (const platform of PLATFORMS) {
  const asset = pin.assets[platform];
  if (!asset) fail(`runtime/geniex.json has no "${platform}" asset`);
  if (!asset.name.includes(pin.version)) fail(`${asset.name} isn't GenieX ${pin.version}`);
  if (!/^[0-9a-f]{64}$/.test(asset.sha256)) fail(`"${asset.sha256}" isn't a SHA-256`);

  const response = await fetch(asset.url, { method: "HEAD" });
  if (!response.ok) fail(`HEAD ${asset.url} → ${response.status}`);
  const length = Number(response.headers.get("content-length"));
  if (length !== asset.size) fail(`size mismatch: remote ${length}, pinned ${asset.size}`);
  console.log(`GenieX ${pin.version}: ${asset.name} is available (${asset.size} bytes)`);
}

function fail(message: string): never {
  console.error(`check-geniex: ${message}`);
  process.exit(1);
}
