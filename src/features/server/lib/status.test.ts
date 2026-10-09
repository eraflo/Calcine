import { describe, expect, it } from "vitest";
import { translator } from "@/i18n";
import type { GatewayStatus } from "@/lib/api";
import { messages } from "../messages";
import { describeStatus } from "./status";

const t = translator(messages, "en");

const status = (overrides: Partial<GatewayStatus> = {}): GatewayStatus => ({
  listening: true,
  baseUrl: "http://127.0.0.1:18181/v1",
  error: null,
  server: { state: "stopped" },
  activeRequests: 0,
  queuedRequests: 0,
  requireApiKey: true,
  ollamaUrl: null,
  ollamaError: null,
  network: null,
  networkError: null,
  ...overrides,
});

describe("describeStatus", () => {
  it("explains a taken port", () => {
    const summary = describeStatus(
      status({ listening: false, error: "Port 18181 is already used" }),
      t,
    );
    expect(summary.tone).toBe("error");
    expect(summary.detail).toContain("18181");
  });

  it("is idle until the first request starts GenieX", () => {
    expect(describeStatus(status(), t).label).toBe("API on");
  });

  it("shows activity and the queue", () => {
    const busy = describeStatus(
      status({
        server: { state: "ready", url: "http://127.0.0.1:5000", startedAtMs: 0 },
        activeRequests: 1,
        queuedRequests: 2,
      }),
      t,
    );
    expect(busy.tone).toBe("busy");
    expect(busy.detail).toContain("2 waiting");
  });

  it("surfaces GenieX failures", () => {
    const failed = describeStatus(
      status({ server: { state: "failed", message: "exited (code 1)" } }),
      t,
    );
    expect(failed).toMatchObject({ tone: "error", detail: "exited (code 1)" });
  });
});
