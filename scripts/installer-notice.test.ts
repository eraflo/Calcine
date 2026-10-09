import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = join(__dirname, "..");
const read = (path: string) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");

// The installer shows this file on its license page: SignPath requires the
// privacy policy to be shown during installation.
describe("installer license page", () => {
  const notice = read("src-tauri/windows/license-and-privacy.txt");

  it("ends with Calcine's license, unchanged", () => {
    expect(notice.endsWith(read("LICENSE"))).toBe(true);
  });

  it("summarizes the privacy policy and links to it", () => {
    expect(notice).toContain("PRIVACY");
    expect(notice).toContain("https://github.com/eraflo/Calcine/blob/main/PRIVACY.md");
  });

  it("is what the installer uses", () => {
    const config = JSON.parse(read("src-tauri/tauri.conf.json"));
    expect(config.bundle.licenseFile).toBe("windows/license-and-privacy.txt");
  });
});
