/** The chat's generation settings, their defaults and presets. */

import type { ComputeUnit } from "@/lib/api";

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
  /** Stop generating at any of these strings. GenieX applies `stop` since
   * 0.10; the chat also cuts the reply itself, for older versions. */
  stop: string[];
  /** Layers offloaded to the GPU or NPU, -1 for all (llama.cpp only). */
  gpuLayers: number | null;
  /** Where llama.cpp vision models encode images. GenieX accepts the CPU or
   * an NPU device, not the GPU. */
  visionCompute: VisionCompute | null;
  /** Speculative decoding (llama.cpp only): guess tokens ahead, then check them. */
  specType: SpecType | null;
  /** Small model with the same vocabulary, for the `draft-*` methods. */
  draftModel: string | null;
  /** Most tokens guessed per step (`spec_n_max`). */
  draftMax: number | null;
  /** Fewest tokens guessed per step (`spec_n_min`). */
  draftMin: number | null;
  /** Lowest probability a guessed token may have (`spec_p_min`). */
  draftPMin: number | null;
  /** Free text, any JSON object, or JSON matching `jsonSchema`. Calcine's
   * gateway checks the reply, on every model. */
  outputFormat: OutputFormat;
  /** JSON Schema text, used when `outputFormat` is `schema`. */
  jsonSchema: string;
  /** When the conversation outgrows the model's context window, stop
   * showing it the oldest messages instead of failing. */
  forgetOldest: boolean;
  /** Have the model summarize what it stops seeing, and keep the summary
   * in its system prompt. */
  summarizeForgotten: boolean;
};

export type OutputFormat = "text" | "json" | "schema";

/** GenieX's speculative decoding methods, simplest first. */
export const SPEC_TYPES = [
  "ngram-cache",
  "ngram-simple",
  "ngram-map-k",
  "ngram-map-k4v",
  "ngram-mod",
  "draft-simple",
  "draft-eagle3",
  "draft-mtp",
] as const;

export type SpecType = (typeof SPEC_TYPES)[number];

/** `draft-*` methods guess with a second, smaller model. */
export function needsDraftModel(spec: SpecType | null): boolean {
  return spec?.startsWith("draft-") ?? false;
}

export type VisionCompute = "cpu" | "npu";

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
  specType: null,
  draftModel: null,
  draftMax: null,
  draftMin: null,
  draftPMin: null,
  outputFormat: "text",
  jsonSchema: "",
  forgetOldest: true,
  summarizeForgotten: true,
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
