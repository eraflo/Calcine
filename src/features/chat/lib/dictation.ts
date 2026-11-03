import type { LocalModel } from "@/lib/api";
import type { DictationLanguage, DictationUnit } from "@/stores/dictation";

/** Recordings for dictation can run longer than audio attachments: the gateway cuts them. */
export const MAX_DICTATION_SECONDS = 120;

/**
 * Model families that understand speech through GenieX's llama.cpp
 * (with their mmproj). Gemma 4's E2B and E4B were tested; Gemma 3n listens the same way.
 */
const LISTENING_MODEL = /gemma[-_ ]?(4|3n)/i;

export function understandsSpeech(model: LocalModel): boolean {
  return model.runtime === "llama_cpp" && LISTENING_MODEL.test(model.name);
}

/** The chosen model when it's installed, otherwise the first one that understands speech. */
export function dictationModel(
  models: readonly LocalModel[],
  chosen: string | null,
): LocalModel | null {
  const picked = chosen ? models.find((model) => model.name === chosen) : undefined;
  return picked ?? models.find(understandsSpeech) ?? null;
}

/**
 * Transcribe a WAV recording with Calcine's `/v1/audio/transcriptions`.
 * Resolves with the text, empty when nobody spoke; throws with the API's
 * explanation otherwise.
 */
export async function transcribe({
  baseUrl,
  token,
  model,
  unit,
  language,
  wav,
  signal,
}: {
  baseUrl: string;
  token: string;
  model: string;
  unit: DictationUnit;
  language: DictationLanguage;
  wav: Uint8Array;
  signal?: AbortSignal;
}): Promise<string> {
  const form = new FormData();
  form.append("file", new Blob([wav as BlobPart], { type: "audio/wav" }), "dictation.wav");
  form.append("model", model);
  form.append("compute", unit);
  if (language !== "auto") form.append("language", language);
  const response = await fetch(`${baseUrl}/audio/transcriptions`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: form,
    signal,
  });
  const body = (await response.json().catch(() => null)) as {
    text?: string;
    error?: { message?: string };
  } | null;
  if (!response.ok) throw new Error(body?.error?.message ?? `HTTP ${response.status}`);
  return (body?.text ?? "").trim();
}

/**
 * Put dictated text where the selection was, spaced from the words around
 * it. Returns the new text and where the caret goes.
 */
export function insertDictation(
  text: string,
  spoken: string,
  start: number,
  end: number,
): { value: string; caret: number } {
  const before = text.slice(0, start);
  const after = text.slice(end);
  const lead = before && !/\s$/.test(before) ? " " : "";
  const trail = after && !/^[\s.,;:!?)]/.test(after) ? " " : "";
  const inserted = before + lead + spoken;
  return { value: inserted + trail + after, caret: inserted.length };
}
