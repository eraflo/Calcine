import { useT } from "@/i18n";
import { formatNumber } from "@/lib/format";
import type { StreamStats } from "../../lib/sse";
import { messages } from "../../messages";

/** Tokens, speed, time to the first token and accepted guesses. */
export function Stats({ stats }: { stats: StreamStats }) {
  const t = useT(messages);
  const parts = [
    stats.completionTokens !== undefined && t.plural("tokens", stats.completionTokens),
    stats.tokensPerSecond &&
      t("tokensPerSecond", { value: formatNumber(stats.tokensPerSecond, 0) }),
    stats.firstTokenMs !== undefined &&
      t("firstToken", { value: formatNumber(stats.firstTokenMs / 1000, 1) }),
    stats.draftTokens &&
      t("draftsAccepted", {
        accepted: String(stats.draftAccepted ?? 0),
        drafted: String(stats.draftTokens),
        percent: formatNumber(((stats.draftAccepted ?? 0) / stats.draftTokens) * 100, 0),
      }),
  ].filter(Boolean);
  return <span className="tabular-nums">{parts.join(" · ")}</span>;
}
