import { Zap } from "lucide-react";
import { Sparkline } from "@/components/calcine/charts/sparkline";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useT } from "@/i18n";
import { formatNumber } from "@/lib/format";
import { HISTORY_LENGTH, useUsageHistory } from "../lib/usage-history";
import { messages } from "../messages";

const watts = (value: number | null | undefined) =>
  value === null || value === undefined ? "—" : `${formatNumber(value, 1)} W`;

/** Live power of the whole system, on devices that meter it. */
export function PowerCard() {
  const t = useT(messages);
  const history = useUsageHistory();
  const power = history.latest?.power;
  if (!power) return null;
  // Scaled to the highest recent value, so the line shows the swings.
  const top = Math.max(10, ...history.watts) * 1.1;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Zap className="size-4 text-warning" />
          <CardTitle>{t("power")}</CardTitle>
          <span className="ml-auto font-mono text-sm text-warning tabular-nums">
            {watts(power.systemWatts)}
          </span>
        </div>
        <CardDescription>{t("powerHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        <Sparkline
          values={history.watts.map((value) => (value / top) * 100)}
          length={HISTORY_LENGTH}
          className="text-warning"
        />
        <div className="flex justify-between gap-3 text-xs text-muted-foreground">
          <span>{t("powerSystem")}</span>
          <span className="tabular-nums">
            {t("powerParts", { cpu: watts(power.cpuWatts), gpu: watts(power.gpuWatts) })}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}
