/**
 * Read or stamp the app version (single source: `[workspace.package]` in
 * Cargo.toml, mirrored in package.json).
 *
 *   bun scripts/version.ts current        # print the version, e.g. 0.1.0
 *   bun scripts/version.ts beta <build>   # stamp and print the next beta, e.g. 0.2.0-beta.42
 *
 * Betas from `dev` target the next minor release, so they always sort after
 * the latest stable and before the release they lead to.
 */
import { join } from "node:path";

const ROOT = join(import.meta.dir, "..");
const CARGO = join(ROOT, "Cargo.toml");
const PACKAGE = join(ROOT, "package.json");
const WORKSPACE_VERSION = /(\[workspace\.package\][^[]*?\nversion\s*=\s*")([^"]+)(")/;

export function nextBeta(version: string, build: number): string {
  const match = /^(\d+)\.(\d+)\.(\d+)/.exec(version);
  if (!match) throw new Error(`not a semver version: ${version}`);
  const [, major, minor] = match;
  return `${major}.${Number(minor) + 1}.0-beta.${build}`;
}

export async function currentVersion(): Promise<string> {
  const cargo = await Bun.file(CARGO).text();
  const version = WORKSPACE_VERSION.exec(cargo)?.[2];
  if (!version) throw new Error("no [workspace.package] version in Cargo.toml");
  return version;
}

export async function stamp(version: string) {
  const cargo = await Bun.file(CARGO).text();
  await Bun.write(CARGO, cargo.replace(WORKSPACE_VERSION, `$1${version}$3`));
  const pkg = await Bun.file(PACKAGE).json();
  pkg.version = version;
  await Bun.write(PACKAGE, `${JSON.stringify(pkg, null, 2)}\n`);
}

if (import.meta.main) {
  const [command, build] = process.argv.slice(2);
  const version = await currentVersion();
  if (command === "current") {
    console.log(version);
  } else if (command === "beta" && build && /^\d+$/.test(build)) {
    const beta = nextBeta(version, Number(build));
    await stamp(beta);
    console.log(beta);
  } else {
    console.error("usage: bun scripts/version.ts current | beta <build-number>");
    process.exit(1);
  }
}
