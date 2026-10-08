import { streamChat } from "@/features/chat/lib/sse";
import type { LocalModel } from "@/lib/api";
import type { GaugeUnit } from "./usage-history";

export const SELF_TEST_PROMPT =
  "Describe a lighthouse on a stormy night in about one hundred words.";

/** Enough tokens for a steady speed, few enough to stay quick. */
const MAX_TOKENS = 160;

export type UnitResult =
  | { status: "waiting" }
  | { status: "running" }
  | { status: "done"; tokensPerSecond: number; firstTokenMs: number }
  | { status: "failed"; message: string };

/** Units a model can be tested on: QAIRT bundles only run on the NPU. */
export function testableUnits(model: LocalModel): GaugeUnit[] {
  return model.runtime === "llama_cpp" ? ["npu", "gpu", "cpu"] : ["npu"];
}

/** Generate a fixed text on `unit` and measure the decoding speed. */
export async function measureUnit({
  baseUrl,
  token,
  model,
  unit,
  signal,
}: {
  baseUrl: string;
  token: string;
  model: LocalModel;
  unit: GaugeUnit;
  signal: AbortSignal;
}): Promise<UnitResult> {
  const result = await streamChat({
    url: `${baseUrl}/chat/completions`,
    token,
    signal,
    onDelta: () => {},
    body: {
      model: model.name,
      messages: [{ role: "user", content: SELF_TEST_PROMPT }],
      stream: true,
      stream_options: { include_usage: true },
      enable_think: false,
      temperature: 0,
      max_tokens: MAX_TOKENS,
      ...(model.runtime === "llama_cpp" ? { compute: unit } : {}),
    },
  });
  const { stats } = result;
  const generating = stats.durationMs - (stats.firstTokenMs ?? 0);
  const tokensPerSecond =
    stats.tokensPerSecond ??
    (stats.completionTokens && generating > 0 ? (stats.completionTokens * 1000) / generating : 0);
  return {
    status: "done",
    tokensPerSecond,
    firstTokenMs: stats.firstTokenMs ?? stats.durationMs,
  };
}

/** The fastest unit among finished results. */
export function fastest(results: Partial<Record<GaugeUnit, UnitResult>>) {
  let best: { unit: GaugeUnit; tokensPerSecond: number } | null = null;
  for (const [unit, result] of Object.entries(results) as [GaugeUnit, UnitResult][]) {
    if (result.status === "done" && result.tokensPerSecond > (best?.tokensPerSecond ?? 0)) {
      best = { unit, tokensPerSecond: result.tokensPerSecond };
    }
  }
  return best;
}
