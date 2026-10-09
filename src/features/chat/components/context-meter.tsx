import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import type { LocalModel } from "@/lib/api";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import { fetchContextUsage, visibleMessages } from "../context-usage";
import { fillLevel, totalTokens } from "../lib/context";
import { buildChatRequest } from "../lib/request";
import { messages } from "../messages";
import { type Conversation, useChat } from "../store";
import { toTurns } from "../use-send";

const BAR = { ok: "bg-primary", high: "bg-warning", full: "bg-destructive" } as const;
const TEXT = {
  ok: "text-muted-foreground",
  high: "text-warning",
  full: "text-destructive",
} as const;

/** How much of the model's context window the conversation takes. */
export function ContextMeter({
  conversation,
  model,
  modelId,
  streaming,
}: {
  conversation: Conversation | undefined;
  model: LocalModel | undefined;
  modelId: string | undefined;
  streaming: boolean;
}) {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const all = conversation?.messages ?? [];
  const visible = visibleMessages(all, conversation?.contextFrom);
  const forgotten = conversation?.contextFrom
    ? all.findIndex((message) => message.id === conversation.contextFrom)
    : 0;
  const usage = useQuery({
    queryKey: [
      "chat",
      "context",
      modelId,
      conversation?.id,
      visible.length,
      visible.at(-1)?.content.length,
      settings.systemPrompt,
      settings.think,
    ],
    queryFn: () =>
      fetchContextUsage(
        modelId ?? "",
        buildChatRequest(model, modelId ?? "", settings, toTurns(visible)),
      ),
    enabled: Boolean(modelId) && !streaming,
    placeholderData: keepPreviousData,
  });

  const window = usage.data?.window;
  if (!usage.data || !window) return null;
  const used = totalTokens(usage.data.tokens);
  const level = fillLevel(used, window);
  const percent = Math.min(100, (used / window) * 100);
  const details = [
    t("contextDetail", {
      used: formatNumber(used, 0),
      window: formatNumber(window, 0),
      percent: formatNumber(percent, 0),
    }),
    usage.data.tokens.exact ? t("contextExact") : t("contextEstimate"),
    forgotten > 0 ? t.plural("contextForgotten", forgotten) : null,
    level === "ok" ? null : settings.forgetOldest ? t("contextFullForget") : t("contextFullStop"),
  ].filter(Boolean);

  return (
    <Tooltip
      content={
        <span className="flex flex-col gap-1">
          {details.map((line) => (
            <span key={line}>{line}</span>
          ))}
        </span>
      }
    >
      {/* A button so keyboard users reach the details too. */}
      <button
        type="button"
        className="flex items-center gap-2 rounded-md px-1.5 py-1 outline-none focus-visible:ring-2 focus-visible:ring-ring"
        aria-label={details.join(" ")}
      >
        <span className="h-1.5 w-14 overflow-hidden rounded-full bg-muted">
          <span
            className={cn("block h-full rounded-full transition-[width]", BAR[level])}
            style={{ width: `${percent}%` }}
          />
        </span>
        <span className={cn("text-[11px] whitespace-nowrap tabular-nums", TEXT[level])}>
          {formatNumber(used, 0)} / {formatNumber(window, 0)}
        </span>
      </button>
    </Tooltip>
  );
}
