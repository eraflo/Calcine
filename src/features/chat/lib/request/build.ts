/**
 * The `/v1/chat/completions` body for the chat's settings and history:
 * media parts, structured output, speculative decoding, documents.
 */

import type { LocalModel } from "@/lib/api";
import { withDocuments, withPassages } from "../../documents/lib/prompt";
import { withSummary } from "../summary";
import { parseSchema } from "./json";
import {
  type ChatSettings,
  DEFAULT_SETTINGS,
  needsDraftModel,
  type VisionCompute,
} from "./settings";

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

/** GenieX device names for `vit_compute`. */
const VISION_DEVICES: Record<VisionCompute, string> = { cpu: "CPU", npu: "HTP0" };

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

/** `response_format`, enforced by Calcine's gateway. An invalid schema sends none. */
function responseFormat(settings: ChatSettings) {
  if (settings.outputFormat === "json") return { response_format: { type: "json_object" } };
  if (settings.outputFormat !== "schema") return {};
  const parsed = parseSchema(settings.jsonSchema);
  if (!("schema" in parsed)) return {};
  return {
    response_format: {
      type: "json_schema",
      json_schema: { name: "response", schema: parsed.schema },
    },
  };
}

/** Speculative decoding fields. A `draft-*` method without its draft model is left out. */
function speculative(settings: ChatSettings) {
  const { specType, draftModel } = settings;
  if (!specType) return {};
  if (needsDraftModel(specType) && !draftModel) return {};
  return {
    spec_type: specType,
    ...(needsDraftModel(specType) ? { spec_draft_model: draftModel } : {}),
    ...optional({
      spec_n_max: settings.draftMax,
      spec_n_min: settings.draftMin,
      spec_p_min: settings.draftPMin,
    }),
  };
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
  /** What the model no longer sees, summarized: goes in the system prompt. */
  summary?: string,
  /** Attached documents: read whole (`system`), or the passages that may
   * answer the last message (`prefix`). */
  documents: { system?: string; prefix?: string } = {},
) {
  const lastUser = history.findLastIndex((turn) => turn.role === "user");
  const turns = history
    .map((turn, index) =>
      index === lastUser && documents.prefix
        ? { ...turn, content: withPassages(turn.content, documents.prefix) }
        : turn,
    )
    .map(toApiMessage);
  const llamaCpp = supportsComputeChoice(model);
  const stop = settings.stop.filter((sequence) => sequence !== "");
  const system = withSummary(withDocuments(settings.systemPrompt, documents.system), summary);
  const messages = system ? [{ role: "system" as const, content: system }, ...turns] : turns;
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
    ...(llamaCpp ? speculative(settings) : {}),
    ...responseFormat(settings),
    // The gateway forgets what still doesn't fit (the chat trims first).
    ...(settings.forgetOldest ? { truncation: "auto" } : {}),
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
