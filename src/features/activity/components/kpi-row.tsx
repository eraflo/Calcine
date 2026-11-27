import { useT } from "@/i18n";
import type { UsageSummary } from "@/lib/api";
import { formatCount, formatNumber } from "@/lib/format";
import { change, energyParts, joulesPerThousandTokens } from "../lib/metrics";
import { messages } from "../messages";
import { StatTile } from "./stat-tile";

/** The period's headline figures, each against the previous period. */
export function KpiRow({ summary }: { summary: UsageSummary }) {
  const t = useT(messages);
  const { totals, previous, period } = summary;
  const hasPrevious = previous.requests > 0;
  const trendLabel = hasPrevious ? t("vsPrevious", { period: t(`previous_${period}`) }) : "";
  const trend = (current: number | null, before: number | null) =>
    hasPrevious ? change(current, before) : null;
  const energy = totals.energyJoules === null ? null : energyParts(totals.energyJoules);
  const perThousand = joulesPerThousandTokens(totals);

  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-5">
      <StatTile
        label={t("requests")}
        value={formatCount(totals.requests)}
        detail={
          totals.errors > 0
            ? t.plural("errors", totals.errors, { count: formatCount(totals.errors) })
            : t("noErrors")
        }
        trend={trend(totals.requests, previous.requests)}
        trendLabel={trendLabel}
      />
      <StatTile
        label={t("tokens")}
        value={formatCount(totals.completionTokens)}
        detail={t("promptTokens", { count: formatCount(totals.promptTokens) })}
        trend={trend(totals.completionTokens, previous.completionTokens)}
        trendLabel={trendLabel}
      />
      <StatTile
        label={t("speed")}
        value={totals.averageSpeed === null ? "—" : formatNumber(totals.averageSpeed, 1)}
        unit={totals.averageSpeed === null ? undefined : "tok/s"}
        hint={t("speedHint")}
        trend={trend(totals.averageSpeed, previous.averageSpeed)}
        trendLabel={trendLabel}
      />
      <StatTile
        label={t("firstToken")}
        value={
          totals.averageFirstTokenMs === null
            ? "—"
            : formatNumber(totals.averageFirstTokenMs / 1000, 2)
        }
        unit={totals.averageFirstTokenMs === null ? undefined : "s"}
        hint={t("firstTokenHint")}
        trend={trend(totals.averageFirstTokenMs, previous.averageFirstTokenMs)}
        trendLabel={trendLabel}
      />
      <StatTile
        label={t("energy")}
        value={energy ? formatNumber(energy.value, energy.value < 10 ? 2 : 1) : "—"}
        unit={energy?.unit}
        detail={
          perThousand === null
            ? t("energyUnavailable")
            : t("perThousand", { value: formatNumber(perThousand, 0) })
        }
        trend={trend(totals.energyJoules, previous.energyJoules)}
        trendLabel={trendLabel}
        hint={t("energyHint")}
      />
    </div>
  );
}
