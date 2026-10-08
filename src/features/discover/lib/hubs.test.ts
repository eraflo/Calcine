import { describe, expect, it } from "vitest";
import { canListPrecisions } from "./hubs";

describe("canListPrecisions", () => {
  it("lists Hugging Face repositories", () => {
    expect(canListPrecisions({ hub: "hugging_face", name: "unsloth/Qwen3-0.6B-GGUF" })).toBe(true);
    expect(canListPrecisions({ hub: "auto", name: "unsloth/Qwen3-0.6B-GGUF" })).toBe(true);
  });

  it("leaves AI Hub, ModelScope and Docker Hub to GenieX", () => {
    expect(canListPrecisions({ hub: "auto", name: "qualcomm/Qwen3-4B" })).toBe(false);
    expect(canListPrecisions({ hub: "ai_hub", name: "qualcomm/Qwen3-4B" })).toBe(false);
    expect(canListPrecisions({ hub: "docker_hub", name: "ai/gemma3" })).toBe(false);
    expect(canListPrecisions({ hub: "model_scope", name: "Qwen/Qwen3-0.6B-GGUF" })).toBe(false);
  });
});
