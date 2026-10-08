import { describe, expect, it } from "vitest";
import type { BenchResult, RequestEntry } from "@/lib/api";
import { benchTargets, best, leaderboard, sessions, toCsv, usageByModel } from "./results";

const stat = (median: number) => ({ median, min: median, max: median, stdev: 1 });

const result = (
  overrides: Partial<BenchResult> & { decode?: number; ttft?: number },
): BenchResult => {
  const { decode, ttft, ...rest } = overrides;
  return {
    id: "r",
    sessionId: "s1",
    startedAtMs: 1000,
    model: "qualcomm/Qwen3-4B:W4A16",
    runtime: "qairt",
    unit: "npu",
    promptTokens: 512,
    generatedTokens: 128,
    repetitions: 5,
    powerMode: "burst",
    measure:
      decode === undefined
        ? null
        : {
            ttftMs: stat(ttft ?? 100),
            prefillTps: stat(2000),
            decodeTps: stat(decode),
            generatedTokens: 128,
            promptTokens: 512,
            geniexVersion: "v0.8.0",
          },
    error: decode === undefined ? "failed" : null,
    ...rest,
  };
};

describe("sessions", () => {
  it("groups by session, newest first, NPU before CPU", () => {
    const grouped = sessions([
      result({ id: "a", unit: "cpu", decode: 20 }),
      result({ id: "b", unit: "npu", decode: 50 }),
      result({ id: "c", sessionId: "s2", startedAtMs: 5000, decode: 10 }),
    ]);
    expect(grouped.map((session) => session.id)).toEqual(["s2", "s1"]);
    expect(grouped[1]?.results.map((r) => r.unit)).toEqual(["npu", "cpu"]);
  });
});

describe("best", () => {
  it("prefers the fastest decoding and the shortest wait", () => {
    const results = [
      result({ id: "npu", decode: 50, ttft: 80 }),
      result({ id: "gpu", unit: "gpu", decode: 60, ttft: 120 }),
      result({ id: "cpu", unit: "cpu" }),
    ];
    expect(best(results, "decodeTps")?.id).toBe("gpu");
    expect(best(results, "ttftMs")?.id).toBe("npu");
    expect(best([result({})], "decodeTps")).toBeNull();
  });
});

describe("leaderboard", () => {
  it("keeps each model's fastest unit, from its latest run", () => {
    const board = leaderboard([
      result({ id: "old", decode: 90, startedAtMs: 1 }),
      result({ id: "new", decode: 40, startedAtMs: 2 }),
      result({ id: "gpu", unit: "gpu", decode: 45, startedAtMs: 2 }),
      result({ id: "small", model: "qualcomm/Qwen3-0.6B:W4A16", decode: 80 }),
    ]);
    expect(board.map((r) => r.id)).toEqual(["small", "gpu"]);
  });
});

describe("toCsv", () => {
  it("writes one row per unit and quotes errors", () => {
    const csv = toCsv([result({ decode: 50 }), result({ error: 'said "no", twice' })]);
    const lines = csv.trim().split("\n");
    expect(lines[0]).toMatch(/^date,model,runtime,unit/);
    expect(lines[1]).toContain(",50,1,");
    expect(lines[2]).toContain('"said ""no"", twice"');
  });
});

describe("benchTargets", () => {
  it("lists every precision", () => {
    const targets = benchTargets([
      {
        name: "a/b",
        sizeBytes: 1,
        runtime: "llama_cpp",
        modelType: "llm",
        precisions: ["Q4_0", "Q8_0"],
      },
      { name: "c/d", sizeBytes: 1, runtime: "qairt", modelType: "llm", precisions: [] },
    ]);
    expect(targets.map((target) => target.id)).toEqual(["a/b:Q4_0", "a/b:Q8_0", "c/d"]);
  });
});

describe("usageByModel", () => {
  const entry = (overrides: Partial<RequestEntry>): RequestEntry => ({
    id: 1,
    startedAtMs: 0,
    client: "Calcine",
    method: "POST",
    path: "/v1/chat/completions",
    model: "m",
    stream: true,
    status: 200,
    durationMs: 1000,
    firstTokenMs: 100,
    promptTokens: 10,
    completionTokens: 50,
    tokensPerSecond: 20,
    error: null,
    ...overrides,
  });

  it("summarizes successful requests per model", () => {
    const usage = usageByModel([
      entry({ firstTokenMs: 100, tokensPerSecond: 20 }),
      entry({ firstTokenMs: 300, tokensPerSecond: 30 }),
      entry({ status: 500 }),
      entry({ model: "other" }),
    ]);
    expect(usage[0]).toEqual({
      model: "m",
      requests: 2,
      completionTokens: 100,
      firstTokenMs: 200,
      tokensPerSecond: 25,
    });
    expect(usage).toHaveLength(2);
  });
});
