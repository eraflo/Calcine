import { describe, expect, it } from "vitest";
import type { LocalModel } from "@/lib/api";
import {
  activePreset,
  buildChatRequest,
  DEFAULT_SETTINGS,
  PRESETS,
  stopIndex,
  toCurl,
  withDefaults,
} from "./request";

const model = (runtime: LocalModel["runtime"]): LocalModel => ({
  name: "m/x",
  sizeBytes: 1,
  runtime,
  modelType: "llm",
  precisions: [],
});

const hello = [{ role: "user" as const, content: "Hello" }];

describe("buildChatRequest", () => {
  it("streams with reasoning kept apart", () => {
    const body = buildChatRequest(model("qairt"), "m/x", DEFAULT_SETTINGS, hello);
    expect(body).toMatchObject({ stream: true, reasoning_format: "deepseek", enable_think: true });
    expect(body.messages).toEqual(hello);
  });

  it("only lets llama.cpp models choose a compute unit", () => {
    const settings = { ...DEFAULT_SETTINGS, compute: "gpu" as const };
    expect(buildChatRequest(model("qairt"), "m/x", settings, hello)).not.toHaveProperty("compute");
    expect(buildChatRequest(model("llama_cpp"), "m/x", settings, hello)).toHaveProperty(
      "compute",
      "gpu",
    );
  });

  it("adds the system prompt and non-default power mode", () => {
    const settings = {
      ...DEFAULT_SETTINGS,
      systemPrompt: " Be brief. ",
      powerMode: "balanced" as const,
    };
    const body = buildChatRequest(model("qairt"), "m/x", settings, hello);
    expect(body.messages[0]).toEqual({ role: "system", content: "Be brief." });
    expect(body).toHaveProperty("power_mode", "balanced");
    expect(buildChatRequest(model("qairt"), "m/x", DEFAULT_SETTINGS, hello)).not.toHaveProperty(
      "power_mode",
    );
  });
});

describe("optional settings", () => {
  const llama = model("llama_cpp");

  it("leaves GenieX's defaults alone unless set", () => {
    const body = buildChatRequest(llama, "m/x", DEFAULT_SETTINGS, hello);
    for (const key of ["top_p", "top_k", "min_p", "seed", "stop", "ngl"]) {
      expect(body).not.toHaveProperty(key);
    }
  });

  it("sends samplers and stop sequences to every model", () => {
    const settings = {
      ...DEFAULT_SETTINGS,
      topP: 0.9,
      topK: 40,
      minP: 0,
      repetitionPenalty: 1.1,
      presencePenalty: 0.5,
      frequencyPenalty: -0.5,
      stop: ["\n\n", ""],
    };
    expect(buildChatRequest(model("qairt"), "m/x", settings, hello)).toMatchObject({
      top_p: 0.9,
      top_k: 40,
      min_p: 0,
      repetition_penalty: 1.1,
      presence_penalty: 0.5,
      frequency_penalty: -0.5,
      stop: ["\n\n"],
    });
  });

  it("sends the seed and offloaded layers to llama.cpp models only", () => {
    const settings = { ...DEFAULT_SETTINGS, seed: 7, gpuLayers: 0 };
    expect(buildChatRequest(llama, "m/x", settings, hello)).toMatchObject({ seed: 7, ngl: 0 });
    const qairt = buildChatRequest(model("qairt"), "m/x", settings, hello);
    expect(qairt).not.toHaveProperty("seed");
    expect(qairt).not.toHaveProperty("ngl");
  });
});

describe("vision encoder", () => {
  it("is sent to llama.cpp vision models only, as a GenieX device name", () => {
    const settings = { ...DEFAULT_SETTINGS, visionCompute: "npu" as const };
    const vlm = { ...model("llama_cpp"), modelType: "vlm" as const };
    expect(buildChatRequest(vlm, "m/x", settings, hello)).toHaveProperty("vit_compute", "HTP0");
    expect(
      buildChatRequest(vlm, "m/x", { ...settings, visionCompute: "cpu" }, hello),
    ).toHaveProperty("vit_compute", "CPU");
    expect(buildChatRequest(model("llama_cpp"), "m/x", settings, hello)).not.toHaveProperty(
      "vit_compute",
    );
    expect(buildChatRequest(vlm, "m/x", DEFAULT_SETTINGS, hello)).not.toHaveProperty("vit_compute");
  });
});

describe("speculative decoding", () => {
  const llama = model("llama_cpp");

  it("sends n-gram methods alone and draft methods with their model", () => {
    const ngram = { ...DEFAULT_SETTINGS, specType: "ngram-cache" as const, draftMax: 8 };
    expect(buildChatRequest(llama, "m/x", ngram, hello)).toMatchObject({
      spec_type: "ngram-cache",
      spec_n_max: 8,
    });
    expect(buildChatRequest(llama, "m/x", ngram, hello)).not.toHaveProperty("spec_draft_model");

    const draft = {
      ...DEFAULT_SETTINGS,
      specType: "draft-simple" as const,
      draftModel: "a/b:Q4_0",
    };
    expect(buildChatRequest(llama, "m/x", draft, hello)).toMatchObject({
      spec_type: "draft-simple",
      spec_draft_model: "a/b:Q4_0",
    });
  });

  it("leaves it out without a draft model, and for AI Hub models", () => {
    const missing = { ...DEFAULT_SETTINGS, specType: "draft-simple" as const };
    expect(buildChatRequest(llama, "m/x", missing, hello)).not.toHaveProperty("spec_type");
    const ngram = { ...DEFAULT_SETTINGS, specType: "ngram-simple" as const };
    expect(buildChatRequest(model("qairt"), "m/x", ngram, hello)).not.toHaveProperty("spec_type");
  });
});

describe("stopIndex", () => {
  it("finds the earliest stop sequence", () => {
    expect(stopIndex("one. two\n\nthree", ["\n\n", "."])).toBe(3);
    expect(stopIndex("no stop here", ["END", ""])).toBeNull();
    expect(stopIndex("anything", [])).toBeNull();
  });
});

describe("presets", () => {
  it("recognizes the preset the settings match", () => {
    expect(activePreset(DEFAULT_SETTINGS)).toBe("balanced");
    expect(activePreset({ ...DEFAULT_SETTINGS, ...PRESETS.creative })).toBe("creative");
    expect(activePreset({ ...DEFAULT_SETTINGS, temperature: 0.4 })).toBeNull();
  });

  it("fills settings saved before a field existed", () => {
    const old = { systemPrompt: "Hi", temperature: 0.3 } as Partial<typeof DEFAULT_SETTINGS>;
    expect(withDefaults(old)).toEqual({
      ...DEFAULT_SETTINGS,
      systemPrompt: "Hi",
      temperature: 0.3,
    });
  });
});

describe("media", () => {
  it("sends images and recordings before the text, OpenAI style", () => {
    const body = buildChatRequest(model("llama_cpp"), "m/x", DEFAULT_SETTINGS, [
      {
        role: "user",
        content: "What is this?",
        media: [
          { kind: "image", dataUrl: "data:image/jpeg;base64,AAAA" },
          { kind: "audio", base64: "UklGRg" },
        ],
      },
    ]);
    expect(body.messages[0]).toEqual({
      role: "user",
      content: [
        { type: "image_url", image_url: { url: "data:image/jpeg;base64,AAAA" } },
        { type: "input_audio", input_audio: { data: "UklGRg", format: "wav" } },
        { type: "text", text: "What is this?" },
      ],
    });
  });

  it("keeps plain strings without media", () => {
    const body = buildChatRequest(model("llama_cpp"), "m/x", DEFAULT_SETTINGS, [
      { role: "user", content: "Hi", media: [] },
    ]);
    expect(body.messages[0]).toEqual({ role: "user", content: "Hi" });
  });
});

describe("toCurl", () => {
  it("escapes single quotes for the shell", () => {
    const curl = toCurl("http://127.0.0.1:18181/v1/chat/completions", { text: "it's" });
    expect(curl).toContain("it'\\''s");
    expect(curl).toContain("$CALCINE_API_KEY");
  });
});
