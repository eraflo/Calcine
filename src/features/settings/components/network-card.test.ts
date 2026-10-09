import { describe, expect, it } from "vitest";
import { isRange } from "./network-card";

describe("isRange", () => {
  it("accepts addresses and ranges, IPv4 or IPv6", () => {
    for (const value of ["192.168.1.0/24", "10.0.0.7", "fd00::/8", "::1", "0.0.0.0/0"]) {
      expect(isRange(value), value).toBe(true);
    }
  });

  it("refuses everything else", () => {
    for (const value of ["192.168.1.0/33", "300.1.1.1", "example.com", "10.0.0.0/x", "1/2/3", ""]) {
      expect(isRange(value), value).toBe(false);
    }
  });
});
