import { describe, expect, it } from "vitest";
import type { LocalModel } from "@/lib/api";
import { dictationModel, insertDictation, understandsSpeech } from "./dictation";

const model = (name: string, runtime: LocalModel["runtime"] = "llama_cpp"): LocalModel => ({
  name,
  sizeBytes: 1,
  runtime,
  modelType: "vlm",
  precisions: ["Q4_0"],
});

describe("dictation model", () => {
  const qwen = model("qualcomm/Qwen3-4B", "qairt");
  const gemma = model("ggml-org/gemma-4-E4B-it-GGUF");

  it("recognizes models that understand speech", () => {
    expect(understandsSpeech(gemma)).toBe(true);
    expect(understandsSpeech(model("unsloth/gemma-3n-E4B-it-GGUF"))).toBe(true);
    expect(understandsSpeech(qwen)).toBe(false);
    expect(understandsSpeech(model("unsloth/gemma-3-4b-it-GGUF"))).toBe(false);
  });

  it("uses the chosen model, or the first one that listens", () => {
    expect(dictationModel([qwen, gemma], null)).toBe(gemma);
    expect(dictationModel([qwen, gemma], "qualcomm/Qwen3-4B")).toBe(qwen);
    // A chosen model that was removed falls back to one that listens.
    expect(dictationModel([qwen, gemma], "gone/model")).toBe(gemma);
    expect(dictationModel([qwen], null)).toBeNull();
  });
});

describe("inserting dictated text", () => {
  it("replaces the selection and spaces the words", () => {
    expect(insertDictation("", "Bonjour.", 0, 0)).toEqual({ value: "Bonjour.", caret: 8 });
    expect(insertDictation("Dis", "bonjour", 3, 3)).toEqual({ value: "Dis bonjour", caret: 11 });
    expect(insertDictation("Dis  à tous", "bonjour", 4, 4)).toEqual({
      value: "Dis bonjour à tous",
      caret: 11,
    });
    expect(insertDictation("A XX B", "mot", 2, 4)).toEqual({ value: "A mot B", caret: 5 });
    // No space before punctuation that follows.
    expect(insertDictation("Dis.", "bonjour", 3, 3)).toEqual({ value: "Dis bonjour.", caret: 11 });
  });
});
