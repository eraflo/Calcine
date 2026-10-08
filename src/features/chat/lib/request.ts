import type { ComputeUnit, LocalModel } from "@/lib/api";

export const POWER_MODES = [
  { value: "burst", label: "Burst (fastest)" },
  { value: "sustained_high_performance", label: "Sustained high performance" },
  { value: "high_performance", label: "High performance" },
  { value: "balanced", label: "Balanced" },
  { value: "low_balanced", label: "Low balanced" },
  { value: "high_power_saver", label: "High power saver" },
  { value: "power_saver", label: "Power saver" },
  { value: "low_power_saver", label: "Low power saver (coolest)" },
] as const;

export type PowerMode = (typeof POWER_MODES)[number]["value"];

export type ChatSettings = {
  systemPrompt: string;
  temperature: number;
  maxTokens: number;
  /** Let reasoning models think before answering. */
  think: boolean;
  /** Only used by llama.cpp models; QAIRT bundles always run on the NPU. */
  compute: Exclude<ComputeUnit, "hybrid">;
  powerMode: PowerMode;
};

export const DEFAULT_SETTINGS: ChatSettings = {
  systemPrompt: "",
  temperature: 0.7,
  maxTokens: 1024,
  think: true,
  compute: "npu",
  powerMode: "burst",
};

export type ChatTurn = { role: "user" | "assistant"; content: string };

/** Whether the model lets you choose where it runs. */
export function supportsComputeChoice(model: LocalModel | undefined): boolean {
  return model?.runtime === "llama_cpp";
}

/**
 * The `/v1/chat/completions` body for these settings. GenieX extensions
 * (`enable_think`, `compute`, `power_mode`) are only sent when they apply.
 */
export function buildChatRequest(
  model: LocalModel | undefined,
  modelId: string,
  settings: ChatSettings,
  history: readonly ChatTurn[],
) {
  const messages = settings.systemPrompt.trim()
    ? [{ role: "system" as const, content: settings.systemPrompt.trim() }, ...history]
    : [...history];
  return {
    model: modelId,
    messages,
    stream: true,
    stream_options: { include_usage: true },
    // Keep the chain of thought out of the answer, in `reasoning_content`.
    reasoning_format: "deepseek",
    enable_think: settings.think,
    temperature: settings.temperature,
    max_tokens: settings.maxTokens,
    ...(supportsComputeChoice(model) ? { compute: settings.compute } : {}),
    ...(settings.powerMode !== DEFAULT_SETTINGS.powerMode
      ? { power_mode: settings.powerMode }
      : {}),
  };
}

/** The same request as a curl command, for reproducing it outside Calcine. */
export function toCurl(url: string, body: unknown): string {
  const json = JSON.stringify(body, null, 2).replaceAll("'", "'\\''");
  return [
    `curl ${url} \\`,
    '  -H "Authorization: Bearer $CALCINE_API_KEY" \\',
    '  -H "Content-Type: application/json" \\',
    `  -d '${json}'`,
  ].join("\n");
}
