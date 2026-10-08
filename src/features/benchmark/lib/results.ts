import type { BenchResult, ComputeUnit, LocalModel, RequestEntry, Runtime } from "@/lib/api";

/** Units in the order they're shown: NPU first, CPU last. */
export const UNIT_ORDER: readonly ComputeUnit[] = ["npu", "hybrid", "gpu", "cpu"];

/** Units `geniex-bench` can run a model on. AI Hub bundles only run on the NPU. */
export function benchUnits(runtime: Runtime): ComputeUnit[] {
  return runtime === "llama_cpp" ? ["npu", "hybrid", "gpu", "cpu"] : ["npu"];
}

/** One entry per downloaded precision, named like GenieX ids. */
export function benchTargets(models: readonly LocalModel[]) {
  return models.flatMap((model) =>
    (model.precisions.length ? model.precisions : [null]).map((precision) => ({
      id: precision ? `${model.name}:${precision}` : model.name,
      model,
    })),
  );
}

/** Results measured together. */
export type Session = {
  id: string;
  startedAtMs: number;
  model: string;
  runtime: Runtime;
  promptTokens: number;
  generatedTokens: number;
  repetitions: number;
  powerMode: string;
  specType: string | null;
  draftModel: string | null;
  results: BenchResult[];
};

/** Group the history by session, newest first, units in display order. */
export function sessions(history: readonly BenchResult[]): Session[] {
  const groups = new Map<string, Session>();
  for (const result of history) {
    const session = groups.get(result.sessionId);
    if (session) {
      session.results.push(result);
      session.startedAtMs = Math.min(session.startedAtMs, result.startedAtMs);
    } else {
      groups.set(result.sessionId, {
        id: result.sessionId,
        startedAtMs: result.startedAtMs,
        model: result.model,
        runtime: result.runtime,
        promptTokens: result.promptTokens,
        generatedTokens: result.generatedTokens,
        repetitions: result.repetitions,
        powerMode: result.powerMode,
        specType: result.specType ?? null,
        draftModel: result.draftModel ?? null,
        results: [result],
      });
    }
  }
  return [...groups.values()]
    .map((session) => ({
      ...session,
      results: session.results.sort(
        (a, b) => UNIT_ORDER.indexOf(a.unit) - UNIT_ORDER.indexOf(b.unit),
      ),
    }))
    .sort((a, b) => b.startedAtMs - a.startedAtMs);
}

export type Metric = "decodeTps" | "prefillTps" | "ttftMs";

/** Median of a metric, `null` if the run failed. */
export function median(result: BenchResult, metric: Metric): number | null {
  return result.measure?.[metric].median ?? null;
}

/** Lower time to first token is better; higher speeds are. */
export const LOWER_IS_BETTER: Record<Metric, boolean> = {
  decodeTps: false,
  prefillTps: false,
  ttftMs: true,
};

/** The best result of a session for a metric. */
export function best(results: readonly BenchResult[], metric: Metric): BenchResult | null {
  let winner: BenchResult | null = null;
  for (const result of results) {
    const value = median(result, metric);
    if (value === null) continue;
    const current = winner ? median(winner, metric) : null;
    if (current === null || (LOWER_IS_BETTER[metric] ? value < current : value > current)) {
      winner = result;
    }
  }
  return winner;
}

/** Each model's fastest decoding, from its latest successful measurement per unit. */
export function leaderboard(history: readonly BenchResult[]) {
  const latest = new Map<string, BenchResult>();
  for (const result of history) {
    if (!result.measure) continue;
    const key = `${result.model}|${result.specType ?? ""}|${result.unit}`;
    const seen = latest.get(key);
    if (!seen || result.startedAtMs > seen.startedAtMs) latest.set(key, result);
  }
  const perModel = new Map<string, BenchResult>();
  for (const result of latest.values()) {
    // With and without speculative decoding are listed apart.
    const key = `${result.model}|${result.specType ?? ""}`;
    const current = perModel.get(key);
    if (!current || (median(result, "decodeTps") ?? 0) > (median(current, "decodeTps") ?? 0)) {
      perModel.set(key, result);
    }
  }
  return [...perModel.values()].sort(
    (a, b) => (median(b, "decodeTps") ?? 0) - (median(a, "decodeTps") ?? 0),
  );
}

/** The whole history as CSV, one row per unit. */
export function toCsv(history: readonly BenchResult[]): string {
  const header = [
    "date",
    "model",
    "runtime",
    "unit",
    "prompt_tokens",
    "generated_tokens",
    "repetitions",
    "power_mode",
    "spec_type",
    "draft_model",
    "ttft_ms_median",
    "ttft_ms_stdev",
    "prefill_tps_median",
    "prefill_tps_stdev",
    "decode_tps_median",
    "decode_tps_stdev",
    "geniex_version",
    "error",
  ];
  const cell = (value: string | number | null | undefined) => {
    const text = value === null || value === undefined ? "" : String(value);
    return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
  };
  const rows = history.map((result) => {
    const measure = result.measure;
    return [
      new Date(result.startedAtMs).toISOString(),
      result.model,
      result.runtime,
      result.unit,
      result.promptTokens,
      result.generatedTokens,
      result.repetitions,
      result.powerMode,
      result.specType,
      result.draftModel,
      measure?.ttftMs.median,
      measure?.ttftMs.stdev,
      measure?.prefillTps.median,
      measure?.prefillTps.stdev,
      measure?.decodeTps.median,
      measure?.decodeTps.stdev,
      measure?.geniexVersion,
      result.error,
    ]
      .map(cell)
      .join(",");
  });
  return `${[header.join(","), ...rows].join("\n")}\n`;
}

/** Everyday speed per model, from the request log (Chat and other apps). */
export type Usage = {
  model: string;
  requests: number;
  completionTokens: number;
  firstTokenMs: number | null;
  tokensPerSecond: number | null;
};

export function usageByModel(entries: readonly RequestEntry[]): Usage[] {
  const byModel = new Map<string, RequestEntry[]>();
  for (const entry of entries) {
    if (entry.status !== 200 || !entry.model || entry.error) continue;
    const list = byModel.get(entry.model) ?? [];
    list.push(entry);
    byModel.set(entry.model, list);
  }
  return [...byModel.entries()]
    .map(([model, list]) => ({
      model,
      requests: list.length,
      completionTokens: list.reduce((sum, entry) => sum + (entry.completionTokens ?? 0), 0),
      firstTokenMs: middle(list.map((entry) => entry.firstTokenMs)),
      tokensPerSecond: middle(list.map((entry) => entry.tokensPerSecond)),
    }))
    .sort((a, b) => b.requests - a.requests);
}

function middle(values: readonly (number | null)[]): number | null {
  const known = values.filter((value): value is number => value !== null).sort((a, b) => a - b);
  if (known.length === 0) return null;
  const half = Math.floor(known.length / 2);
  return known.length % 2
    ? (known[half] ?? null)
    : ((known[half - 1] ?? 0) + (known[half] ?? 0)) / 2;
}
