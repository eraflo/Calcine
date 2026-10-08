import { useCallback, useEffect, useRef, useState } from "react";
import { MAX_RECORDING_SECONDS } from "./attachments";

type RecorderState =
  | { status: "idle" }
  | { status: "recording"; seconds: number }
  | { status: "processing" };

/**
 * Record from the microphone with `MediaRecorder`. `stop` resolves with the
 * recording; recordings stop by themselves after a minute.
 */
export function useRecorder() {
  const [state, setState] = useState<RecorderState>({ status: "idle" });
  const recorder = useRef<MediaRecorder | null>(null);
  const finished = useRef<((blob: Blob | null) => void) | null>(null);

  const release = useCallback(() => {
    for (const track of recorder.current?.stream.getTracks() ?? []) track.stop();
    recorder.current = null;
  }, []);

  useEffect(() => release, [release]);

  useEffect(() => {
    if (state.status !== "recording") return;
    const timer = setInterval(() => {
      setState((current) => {
        if (current.status !== "recording") return current;
        const seconds = current.seconds + 1;
        if (seconds >= MAX_RECORDING_SECONDS) recorder.current?.stop();
        return { status: "recording", seconds };
      });
    }, 1_000);
    return () => clearInterval(timer);
  }, [state.status]);

  /** Ask for the microphone and start. Throws if access is refused. */
  const start = useCallback(async (onFinished: (blob: Blob | null) => void) => {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    const media = new MediaRecorder(stream);
    const chunks: Blob[] = [];
    media.ondataavailable = (event) => chunks.push(event.data);
    media.onstop = () => {
      setState({ status: "processing" });
      for (const track of stream.getTracks()) track.stop();
      const blob = chunks.length ? new Blob(chunks, { type: media.mimeType }) : null;
      finished.current?.(blob);
      finished.current = null;
      recorder.current = null;
    };
    finished.current = onFinished;
    recorder.current = media;
    media.start();
    setState({ status: "recording", seconds: 0 });
  }, []);

  const stop = useCallback(() => recorder.current?.stop(), []);
  const done = useCallback(() => setState({ status: "idle" }), []);

  return { state, start, stop, done };
}
