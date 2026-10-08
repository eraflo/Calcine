import { describe, expect, it } from "vitest";
import type { GatewayStatus } from "@/lib/api";
import { describeStatus } from "./status";

const status = (overrides: Partial<GatewayStatus> = {}): GatewayStatus => ({
  listening: true,
  baseUrl: "http://127.0.0.1:18181/v1",
  error: null,
  server: { state: "stopped" },
  activeRequests: 0,
  queuedRequests: 0,
  requireApiKey: true,
  ...overrides,
});

describe("describeStatus", () => {
  it("explains a taken port", () => {
    const summary = describeStatus(
      status({ listening: false, error: "Port 18181 is already used" }),
    );
    expect(summary.tone).toBe("error");
    expect(summary.detail).toContain("18181");
  });

  it("is idle until the first request starts GenieX", () => {
    expect(describeStatus(status()).label).toBe("API on");
  });

  it("shows activity and the queue", () => {
    const busy = describeStatus(
      status({
        server: { state: "ready", url: "http://127.0.0.1:5000", startedAtMs: 0 },
        activeRequests: 1,
        queuedRequests: 2,
      }),
    );
    expect(busy.tone).toBe("busy");
    expect(busy.detail).toContain("2 waiting");
  });

  it("surfaces GenieX failures", () => {
    const failed = describeStatus(
      status({ server: { state: "failed", message: "exited (code 1)" } }),
    );
    expect(failed).toMatchObject({ tone: "error", detail: "exited (code 1)" });
  });
});
