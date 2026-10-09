/**
 * Minimal server-sent events reader for OpenAI-style streams (`data:{…}`
 * lines, ending with `data:[DONE]`). Chunks may split lines anywhere.
 */
export function createSseParser(onData: (data: string) => void) {
  let pending = "";
  return {
    push(text: string) {
      pending += text;
      let newline = pending.indexOf("\n");
      while (newline !== -1) {
        const line = pending.slice(0, newline).replace(/\r$/, "");
        pending = pending.slice(newline + 1);
        if (line.startsWith("data:")) onData(line.slice(5).trim());
        newline = pending.indexOf("\n");
      }
    },
  };
}

export type StreamStats = {
  promptTokens?: number;
  completionTokens?: number;
  tokensPerSecond?: number;
  firstTokenMs?: number;
  /** Tokens guessed by speculative decoding, and how many were kept. */
  draftTokens?: number;
  draftAccepted?: number;
  durationMs: number;
};

export type StreamResult = {
  content: string;
  reasoning: string;
  stats: StreamStats;
  /** Messages the gateway left out to fit the context window. */
  forgotten: number;
};

type Delta = { content?: string; reasoning?: string };

type Chunk = {
  /** GenieX reports generation failures inside the stream, after a 200. */
  error?: string | { message?: string };
  choices?: { delta?: { content?: string | null; reasoning_content?: string | null } }[];
  usage?: { prompt_tokens?: number; completion_tokens?: number };
  timings?: { predicted_per_second?: number; draft_n?: number; draft_n_accepted?: number };
};

/** POST a streaming chat completion and report text as it arrives. */
export async function streamChat({
  url,
  token,
  body,
  signal,
  onDelta,
}: {
  url: string;
  token: string;
  body: unknown;
  signal: AbortSignal;
  onDelta: (delta: Delta) => void;
}): Promise<StreamResult> {
  const started = performance.now();
  const response = await fetch(url, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    body: JSON.stringify(body),
    signal,
  });
  if (!response.ok || !response.body) {
    throw new Error(await errorMessage(response));
  }

  const result: StreamResult = {
    content: "",
    reasoning: "",
    stats: { durationMs: 0 },
    forgotten: Number(response.headers.get("x-calcine-forgotten-messages") ?? 0) || 0,
  };
  let failure: string | null = null;
  const parser = createSseParser((data) => {
    if (data === "[DONE]" || !data.startsWith("{")) return;
    const chunk = JSON.parse(data) as Chunk;
    if (chunk.error) {
      failure =
        typeof chunk.error === "string"
          ? chunk.error
          : (chunk.error.message ?? "Generation failed");
      return;
    }
    const delta = chunk.choices?.[0]?.delta;
    const content = delta?.content ?? undefined;
    const reasoning = delta?.reasoning_content ?? undefined;
    if (content || reasoning) {
      result.stats.firstTokenMs ??= Math.round(performance.now() - started);
      if (content) result.content += content;
      if (reasoning) result.reasoning += reasoning;
      onDelta({ content, reasoning });
    }
    if (chunk.usage) {
      result.stats.promptTokens = chunk.usage.prompt_tokens;
      result.stats.completionTokens = chunk.usage.completion_tokens;
    }
    if (chunk.timings?.predicted_per_second) {
      result.stats.tokensPerSecond = chunk.timings.predicted_per_second;
    }
    // Only present when speculative decoding guessed something.
    if (chunk.timings?.draft_n) {
      result.stats.draftTokens = chunk.timings.draft_n;
      result.stats.draftAccepted = chunk.timings.draft_n_accepted ?? 0;
    }
  });

  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    parser.push(value);
  }
  if (failure) throw new Error(failure);
  result.stats.durationMs = Math.round(performance.now() - started);
  return result;
}

async function errorMessage(response: Response): Promise<string> {
  const text = await response.text().catch(() => "");
  try {
    const parsed = JSON.parse(text) as { error?: { message?: string } | string };
    const message = typeof parsed.error === "string" ? parsed.error : parsed.error?.message;
    if (message) return message;
  } catch {
    // Not JSON: fall through to the raw text.
  }
  return text || `The local API answered ${response.status}.`;
}
