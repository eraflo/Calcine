import { describe, expect, it } from "vitest";
import { LINUX, manifest, WINDOWS } from "./updater-manifest";

describe("updater manifest", () => {
  it("points each platform's keys at its release asset", () => {
    const result = manifest(
      "1.0.0",
      [
        { file: "Calcine_1.0.0_arm64-setup.exe", signature: "c2lnbmF0dXJl\n", platforms: WINDOWS },
        { file: "Calcine_1.0.0_arm64.deb", signature: "ZGVi", platforms: LINUX },
      ],
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
    expect(result.platforms["linux-aarch64-deb"]).toEqual({
      signature: "ZGVi",
      url: "https://github.com/eraflo/Calcine/releases/download/v1.0.0/Calcine_1.0.0_arm64.deb",
    });
    expect(result.platforms["linux-aarch64"]).toEqual(result.platforms["linux-aarch64-deb"]);
  });

  it("leaves Linux out when it wasn't built", () => {
    const result = manifest(
      "1.0.0",
      [{ file: "Calcine_1.0.0_arm64-setup.exe", signature: "x", platforms: WINDOWS }],
      "",
    );
    expect(Object.keys(result.platforms)).toEqual(WINDOWS);
  });
});
