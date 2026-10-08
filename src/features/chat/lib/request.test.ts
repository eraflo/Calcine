import { describe, expect, it } from "vitest";
import type { LocalModel } from "@/lib/api";
import { buildChatRequest, DEFAULT_SETTINGS, toCurl } from "./request";

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
