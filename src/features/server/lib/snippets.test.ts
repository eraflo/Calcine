import { describe, expect, it } from "vitest";
import { SNIPPET_LANGUAGES, snippet } from "./snippets";

const BASE = "http://127.0.0.1:18181/v1";
const MODEL = "qualcomm/Qwen3-0.6B";

describe("snippet", () => {
  it.each(SNIPPET_LANGUAGES.map((language) => language.id))(
    "%s points at the gateway and never embeds a key",
    (language) => {
      const code = snippet(language, BASE, MODEL);
      // Ollama's API lives at the root, not under /v1.
      expect(code).toContain(language === "ollama" ? BASE.replace(/\/v1$/, "") : BASE);
      // Open WebUI lists the models itself.
      if (language !== "openwebui") expect(code).toContain(MODEL);
      expect(code).toContain("CALCINE_API_KEY");
      expect(code).not.toMatch(/calcine_[0-9a-f]{8}/);
    },
  );

  it("builds a valid curl command", () => {
    expect(snippet("curl", BASE, MODEL)).toContain(`curl ${BASE}/chat/completions`);
  });
});
