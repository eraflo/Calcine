import { useMutation, useQuery } from "@tanstack/react-query";
import { Check, Loader2, Zap } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { connectionQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import type { LocalModel } from "@/lib/api";
import { buildChatRequest } from "../lib/request";
import { streamChat } from "../lib/sse";
import { messages } from "../messages";
import { useChat } from "../store";

/**
 * Load the model now, with the chat's settings (compute unit, offloaded
 * layers), so the first reply doesn't wait for it. GenieX keeps it loaded
 * for the unload delay set on the Server page.
 */
export function PreloadButton({
  model,
  modelId,
  disabled,
}: {
  model: LocalModel | undefined;
  modelId: string | undefined;
  disabled: boolean;
}) {
  const t = useT(messages);
  const { data: connection } = useQuery(connectionQuery);
  const [loaded, setLoaded] = useState(false);
  const preload = useMutation({
    mutationFn: async ({ id, url, token }: { id: string; url: string; token: string }) => {
      const request = buildChatRequest(model, id, useChat.getState().settings, [
        { role: "user", content: "Hi" },
      ]);
      await streamChat({
        url: `${url}/chat/completions`,
        token,
        body: { ...request, max_tokens: 1, enable_think: false },
        signal: AbortSignal.timeout(5 * 60_000),
        onDelta: () => {},
      });
    },
    onSuccess: () => setLoaded(true),
  });

  useEffect(() => {
    if (!loaded) return;
    const timer = setTimeout(() => setLoaded(false), 2500);
    return () => clearTimeout(timer);
  }, [loaded]);

  const label = preload.isPending
    ? t("preloading")
    : loaded
      ? t("preloaded")
      : preload.isError
        ? t("preloadFailed", { message: preload.error.message })
        : t("preload");
  return (
    <Tooltip content={label}>
      <Button
        variant="ghost"
        size="icon"
        className={preload.isError ? "text-destructive" : undefined}
        disabled={disabled || !modelId || !connection || preload.isPending}
        onClick={() =>
          modelId &&
          connection &&
          preload.mutate({ id: modelId, url: connection.baseUrl, token: connection.token })
        }
        aria-label={t("preload")}
      >
        {preload.isPending ? (
          <Loader2 className="animate-spin" />
        ) : loaded ? (
          <Check className="text-success" />
        ) : (
          <Zap />
        )}
      </Button>
    </Tooltip>
  );
}
