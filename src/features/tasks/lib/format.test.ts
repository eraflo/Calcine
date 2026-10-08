import { describe, expect, it } from "vitest";
import { translator } from "@/i18n";
import type { Job } from "@/lib/api";
import { messages } from "../messages";
import { formatDuration, jobTitle, percent, progressLabel, secondsLeft, upsertJob } from "./format";

const t = translator(messages, "en");

const job = (overrides: Partial<Job> = {}): Job => ({
  id: 1,
  kind: { type: "pull", model: "qualcomm/Qwen3-0.6B" },
  state: { state: "running" },
  progress: {
    doneBytes: 190_000_000,
    totalBytes: 760_000_000,
    bytesPerSecond: 19_000_000,
    phase: null,
  },
  startedAtMs: 0,
  finishedAtMs: null,
  ...overrides,
});

describe("job progress", () => {
  it("computes percent and time left", () => {
    expect(percent(job())).toBe(25);
    expect(secondsLeft(job())).toBe(30);
  });

  it("handles unknown totals", () => {
    const unknown = job({
      progress: { doneBytes: 10, totalBytes: null, bytesPerSecond: null, phase: null },
    });
    expect(percent(unknown)).toBeNull();
    expect(secondsLeft(unknown)).toBeNull();
    expect(progressLabel(unknown, t)).toBe("10 B");
  });

  it("describes progress like a download manager", () => {
    expect(progressLabel(job(), t)).toBe("181 MiB of 725 MiB · 18 MiB/s · 30s left");
    expect(progressLabel(job({ progress: null }), t)).toBe("Starting…");
  });
});

describe("formatDuration", () => {
  it("scales units", () => {
    expect(formatDuration(45)).toBe("45s");
    expect(formatDuration(185)).toBe("3m 05s");
    expect(formatDuration(4320)).toBe("1h 12m");
  });
});

describe("upsertJob", () => {
  it("adds new jobs first and replaces known ones", () => {
    const first = job({ id: 1 });
    const second = job({ id: 2 });
    expect(upsertJob([first], second).map((j) => j.id)).toEqual([2, 1]);
    const done = job({ id: 1, state: { state: "succeeded" } });
    expect(upsertJob([second, first], done)[1]?.state).toEqual({ state: "succeeded" });
  });
});

describe("GenieX installs", () => {
  it("are titled by version and describe their phase", () => {
    const install = job({
      kind: { type: "install_runtime", version: "v0.8.1" },
      progress: { doneBytes: 0, totalBytes: null, bytesPerSecond: null, phase: "installing" },
    });
    expect(jobTitle(install, t)).toBe("GenieX v0.8.1");
    expect(progressLabel(install, t)).toBe("Installing… GenieX is unavailable for a moment.");
  });
});
