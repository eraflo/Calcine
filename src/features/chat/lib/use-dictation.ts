import { useQuery } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import { modelsQuery } from "@/features/library/api";
import { connectionQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import { useDictationSettings } from "@/stores/dictation";
import { messages } from "../messages";
import { recordingToWavBytes } from "./attachments";
import { dictationModel, MAX_DICTATION_SECONDS, transcribe } from "./dictation";
import { useRecorder } from "./use-recorder";

export type DictationState =
  | { status: "idle" }
  | { status: "recording"; seconds: number }
  | { status: "transcribing" };

/**
 * Dictation: record, then have the dictation model write it down through
 * Calcine's API. `onText` receives the words; problems go to `onProblem`.
 */
export function useDictation({
  onText,
  onProblem,
}: {
  onText: (text: string) => void;
  onProblem: (message: string | null) => void;
}) {
  const t = useT(messages);
  const { data: connection } = useQuery(connectionQuery);
  const { data: models } = useQuery(modelsQuery);
  const { model: chosen, unit, language } = useDictationSettings();
  const model = dictationModel(models ?? [], chosen);
  const recorder = useRecorder(MAX_DICTATION_SECONDS);
  const [transcribing, setTranscribing] = useState(false);
  const controller = useRef<AbortController | null>(null);

  // Leaving the chat drops a transcription still running.
  useEffect(() => () => controller.current?.abort(), []);

  const state: DictationState = transcribing
    ? { status: "transcribing" }
    : recorder.state.status === "recording"
      ? { status: "recording", seconds: recorder.state.seconds }
      : recorder.state.status === "processing"
        ? { status: "transcribing" }
        : { status: "idle" };

  const toggle = useCallback(async () => {
    if (recorder.state.status === "recording") {
      recorder.stop();
      return;
    }
    if (!model || !connection) return;
    onProblem(null);
    try {
      await recorder.start(async (blob) => {
        setTranscribing(true);
        const abort = new AbortController();
        controller.current = abort;
        try {
          if (!blob) return;
          const { bytes } = await recordingToWavBytes(blob);
          const text = await transcribe({
            baseUrl: connection.baseUrl,
            token: connection.token,
            model: model.name,
            unit,
            language,
            wav: bytes,
            signal: abort.signal,
          });
          if (text) onText(text);
          else onProblem(t("dictationEmpty"));
        } catch (error) {
          if (!abort.signal.aborted) {
            onProblem(t("dictationFailed", { message: describe(error) }));
          }
        } finally {
          controller.current = null;
          setTranscribing(false);
          recorder.done();
        }
      });
    } catch (error) {
      onProblem(t("micDenied", { message: describe(error) }));
    }
  }, [recorder, model, connection, unit, language, onText, onProblem, t]);

  return { state, toggle, model, available: Boolean(model && connection) };
}

function describe(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
