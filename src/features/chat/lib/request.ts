import type { ComputeUnit, LocalModel } from "@/lib/api";

/** GenieX power modes, fastest first. Labels live in the chat messages. */
export const POWER_MODES = [
  "burst",
  "sustained_high_performance",
  "high_performance",
  "balanced",
  "low_balanced",
  "high_power_saver",
  "power_saver",
  "low_power_saver",
] as const;

export type PowerMode = (typeof POWER_MODES)[number];

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

/** An image or a recording sent with a message. */
export type MediaPart = { kind: "image"; dataUrl: string } | { kind: "audio"; base64: string };

export type ChatTurn = { role: "user" | "assistant"; content: string; media?: MediaPart[] };

/** Whether the model lets you choose where it runs. */
export function supportsComputeChoice(model: LocalModel | undefined): boolean {
  return model?.runtime === "llama_cpp";
}

/** Whether the model takes images and audio. */
export function supportsMedia(model: LocalModel | undefined): boolean {
  return model?.modelType === "vlm";
}

/** OpenAI message content: plain text, or parts with the media first. */
function toApiMessage({ role, content, media }: ChatTurn) {
  if (!media?.length) return { role, content };
  return {
    role,
    content: [
      ...media.map((part) =>
        part.kind === "image"
          ? { type: "image_url" as const, image_url: { url: part.dataUrl } }
          : { type: "input_audio" as const, input_audio: { data: part.base64, format: "wav" } },
      ),
      ...(content ? [{ type: "text" as const, text: content }] : []),
    ],
  };
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
  const turns = history.map(toApiMessage);
  const messages = settings.systemPrompt.trim()
    ? [{ role: "system" as const, content: settings.systemPrompt.trim() }, ...turns]
    : turns;
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
