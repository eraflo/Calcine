import { type ChatSettings, DEFAULT_SETTINGS } from "../../lib/request";

export const COMPUTE_UNITS = [
  { value: "npu", tone: "text-npu" },
  { value: "gpu", tone: "text-gpu" },
  { value: "cpu", tone: "text-cpu" },
] as const;

export const ADVANCED_SAMPLING = [
  "topP",
  "topK",
  "minP",
  "repetitionPenalty",
  "presencePenalty",
  "frequencyPenalty",
  "seed",
  "stop",
] as const satisfies readonly (keyof ChatSettings)[];

export const MODEL_LOADING = [
  "gpuLayers",
  "visionCompute",
] as const satisfies readonly (keyof ChatSettings)[];

export const SPECULATIVE = [
  "specType",
  "draftModel",
  "draftMax",
  "draftMin",
  "draftPMin",
] as const satisfies readonly (keyof ChatSettings)[];

/** GenieX takes the seed as a 32-bit signed integer. */
export const MAX_SEED = 2_147_483_647;

export function isChanged(settings: ChatSettings, key: keyof ChatSettings) {
  const value = settings[key];
  return Array.isArray(value) ? value.length > 0 : value !== DEFAULT_SETTINGS[key];
}

export function pick<K extends keyof ChatSettings>(settings: ChatSettings, keys: readonly K[]) {
  return Object.fromEntries(keys.map((key) => [key, settings[key]])) as Pick<ChatSettings, K>;
}
