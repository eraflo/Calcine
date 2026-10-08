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
  // Optional sampler settings: `null` leaves GenieX's default.
  topP: number | null;
  topK: number | null;
  minP: number | null;
  repetitionPenalty: number | null;
  presencePenalty: number | null;
  frequencyPenalty: number | null;
  /** Same seed and settings give the same reply (llama.cpp only). */
  seed: number | null;
  /** Stop generating at any of these strings. GenieX 0.8 ignores `stop` in
   * API requests, so the chat also cuts the reply itself. */
  stop: string[];
  /** Layers offloaded to the GPU or NPU, -1 for all (llama.cpp only). */
  gpuLayers: number | null;
  /** Where llama.cpp vision models encode images. GenieX 0.8 accepts the
   * CPU or an NPU device, not the GPU. */
  visionCompute: VisionCompute | null;
};

export type VisionCompute = "cpu" | "npu";

/** GenieX device names for `vit_compute`. */
const VISION_DEVICES: Record<VisionCompute, string> = { cpu: "CPU", npu: "HTP0" };

export const DEFAULT_SETTINGS: ChatSettings = {
  systemPrompt: "",
  temperature: 0.7,
  maxTokens: 1024,
  think: true,
  compute: "npu",
  powerMode: "burst",
  topP: null,
  topK: null,
  minP: null,
  repetitionPenalty: null,
  presencePenalty: null,
  frequencyPenalty: null,
  seed: null,
  stop: [],
  gpuLayers: null,
  visionCompute: null,
};

/** Sampling presets, from focused to inventive. */
export const PRESETS = {
  precise: { temperature: 0.2, topP: 0.9, topK: null, minP: null },
  balanced: { temperature: 0.7, topP: null, topK: null, minP: null },
  creative: { temperature: 1.1, topP: 0.98, topK: null, minP: null },
} as const satisfies Record<string, Pick<ChatSettings, "temperature" | "topP" | "topK" | "minP">>;

export type Preset = keyof typeof PRESETS;

/** The preset these settings match, if any. */
export function activePreset(settings: ChatSettings): Preset | null {
  for (const [name, values] of Object.entries(PRESETS) as [Preset, (typeof PRESETS)[Preset]][]) {
    if (
      settings.temperature === values.temperature &&
      settings.topP === values.topP &&
      settings.topK === values.topK &&
      settings.minP === values.minP
    ) {
      return name;
    }
  }
  return null;
}

/** Settings saved before a field existed get its default. */
export function withDefaults(settings: Partial<ChatSettings> | undefined): ChatSettings {
  return { ...DEFAULT_SETTINGS, ...settings };
}

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

/** Where the first stop sequence starts in `text`, if any. */
export function stopIndex(text: string, stops: readonly string[]): number | null {
  let first: number | null = null;
  for (const stop of stops) {
    if (!stop) continue;
    const index = text.indexOf(stop);
    if (index !== -1 && (first === null || index < first)) first = index;
  }
  return first;
}

/**
 * The `/v1/chat/completions` body for these settings. Optional samplers are
 * only sent when set, and llama.cpp options (`compute`, `seed`, `ngl`) only
 * to llama.cpp models.
 */
export function buildChatRequest(
  model: LocalModel | undefined,
  modelId: string,
  settings: ChatSettings,
  history: readonly ChatTurn[],
) {
  const turns = history.map(toApiMessage);
  const llamaCpp = supportsComputeChoice(model);
  const stop = settings.stop.filter((sequence) => sequence !== "");
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
    ...optional({
      top_p: settings.topP,
      top_k: settings.topK,
      min_p: settings.minP,
      repetition_penalty: settings.repetitionPenalty,
      presence_penalty: settings.presencePenalty,
      frequency_penalty: settings.frequencyPenalty,
    }),
    ...(stop.length ? { stop } : {}),
    ...(llamaCpp
      ? { compute: settings.compute, ...optional({ seed: settings.seed, ngl: settings.gpuLayers }) }
      : {}),
    ...(llamaCpp && supportsMedia(model) && settings.visionCompute
      ? { vit_compute: VISION_DEVICES[settings.visionCompute] }
      : {}),
    ...(settings.powerMode !== DEFAULT_SETTINGS.powerMode
      ? { power_mode: settings.powerMode }
      : {}),
  };
}

/** The entries that have a value. */
function optional<T extends Record<string, number | null>>(values: T) {
  return Object.fromEntries(Object.entries(values).filter(([, value]) => value !== null)) as {
    [K in keyof T]?: number;
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
