import { describe, expect, it } from "vitest";
import { manifest } from "./updater-manifest";

describe("updater manifest", () => {
  it("points both Windows ARM64 keys at the release asset", () => {
    const result = manifest(
      "1.0.0",
      "Calcine_1.0.0_arm64-setup.exe",
      "c2lnbmF0dXJl\n",
      "Notes",
      new Date("2026-10-09T10:00:00Z"),
    );
    expect(result.version).toBe("1.0.0");
    expect(result.pub_date).toBe("2026-10-09T10:00:00.000Z");
    const entry = result.platforms["windows-aarch64"];
    expect(entry).toEqual(result.platforms["windows-aarch64-nsis"]);
    expect(entry?.signature).toBe("c2lnbmF0dXJl");
    expect(entry?.url).toBe(
      "https://github.com/eraflo/Calcine/releases/download/v1.0.0/Calcine_1.0.0_arm64-setup.exe",
    );
  });
});
