import { describe, expect, it } from "vitest";
import { isOrigin } from "./system-cards";

describe("isOrigin", () => {
  it("matches what the gateway accepts", () => {
    expect(isOrigin("http://localhost:3000")).toBe(true);
    expect(isOrigin("https://example.com")).toBe(true);
    expect(isOrigin("http://[::1]:5173")).toBe(true);
    expect(isOrigin("localhost:3000")).toBe(false);
    expect(isOrigin("http://localhost:3000/app")).toBe(false);
  });
});
