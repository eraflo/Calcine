import { afterEach, describe, expect, it, vi } from "vitest";
import { createSseParser, streamChat } from "./sse";

describe("createSseParser", () => {
  it("handles lines split across chunks", () => {
    const seen: string[] = [];
    const parser = createSseParser((data) => seen.push(data));
    parser.push('data:{"a":');
    parser.push("1}\n\ndata: [DONE]\r\n");
    expect(seen).toEqual(['{"a":1}', "[DONE]"]);
  });
});

describe("streamChat", () => {
  afterEach(() => vi.unstubAllGlobals());

  const sse = (lines: string[]) =>
    new Response(lines.map((line) => `${line}\n\n`).join(""), {
      headers: { "Content-Type": "text/event-stream" },
    });

  it("throws errors GenieX reports inside a 200 stream", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        sse(['data:{"code":-201201,"error":"SDKError(Multimodal generation failed)"}']),
      ),
    );
    await expect(
      streamChat({
        url: "http://x/v1/chat/completions",
        token: "t",
        body: {},
        signal: new AbortController().signal,
        onDelta: () => {},
      }),
    ).rejects.toThrow("Multimodal generation failed");
  });

  it("collects content, reasoning and stats from a GenieX stream", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        sse([
          'data:{"choices":[{"delta":{"reasoning_content":"Thinking"}}]}',
          'data:{"choices":[{"delta":{"content":"Hello"}}]}',
          'data:{"choices":[{"delta":{"content":" there"}}]}',
          'data:{"choices":[],"usage":{"prompt_tokens":5,"completion_tokens":2},"timings":{"predicted_per_second":78.4}}',
          "data:[DONE]",
        ]),
      ),
    );
    const deltas: string[] = [];
    const result = await streamChat({
      url: "http://127.0.0.1:18181/v1/chat/completions",
      token: "t",
      body: {},
      signal: new AbortController().signal,
      onDelta: (delta) => deltas.push(delta.content ?? delta.reasoning ?? ""),
    });
    expect(result.content).toBe("Hello there");
    expect(result.reasoning).toBe("Thinking");
    expect(result.stats).toMatchObject({
      promptTokens: 5,
      completionTokens: 2,
      tokensPerSecond: 78.4,
    });
    expect(deltas).toEqual(["Thinking", "Hello", " there"]);
  });

  it("surfaces the gateway's error message", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        Response.json({ error: { message: "model x isn't downloaded" } }, { status: 400 }),
      ),
    );
    await expect(
      streamChat({
        url: "u",
        token: "t",
        body: {},
        signal: new AbortController().signal,
        onDelta: () => {},
      }),
    ).rejects.toThrow("model x isn't downloaded");
  });
});
