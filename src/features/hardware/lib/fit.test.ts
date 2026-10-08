import { describe, expect, it } from "vitest";
import { comfortableModelBytes, fitsOnDisk, memoryFit } from "./fit";

const GIB = 1024 ** 3;
const memory = { totalBytes: 32 * GIB, availableBytes: 12 * GIB };

describe("memoryFit", () => {
  it("fits when free memory covers the model and its overhead", () => {
    expect(memoryFit(3 * GIB, memory)).toBe("fits");
    expect(memoryFit(10 * GIB, memory)).toBe("fits");
  });

  it("is tight when other apps would have to close", () => {
    expect(memoryFit(11 * GIB, memory)).toBe("tight");
    expect(memoryFit(21 * GIB, memory)).toBe("tight");
  });

  it("is too big beyond most of the device memory", () => {
    expect(memoryFit(22 * GIB, memory)).toBe("too_big");
  });
});

describe("comfortableModelBytes", () => {
  it("leaves room for the context", () => {
    expect(comfortableModelBytes(memory)).toBe(10 * GIB);
  });
});

describe("fitsOnDisk", () => {
  const disk = { path: "C:\\", totalBytes: 512 * GIB, availableBytes: 5 * GIB };
  it("keeps a margin", () => {
    expect(fitsOnDisk(3 * GIB, disk)).toBe(true);
    expect(fitsOnDisk(4.5 * GIB, disk)).toBe(false);
  });
});
