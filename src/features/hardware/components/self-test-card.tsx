import { useQuery } from "@tanstack/react-query";
import { Gauge, Loader2, Play, Square, Trophy } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select } from "@/components/ui/field";
import { modelsQuery } from "@/features/library/api";
import { connectionQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import { fastest, measureUnit, testableUnits, type UnitResult } from "../lib/self-test";
import type { GaugeUnit } from "../lib/usage-history";
import { messages } from "../messages";
import { UNIT_STYLE } from "./compute-units";

type Results = Partial<Record<GaugeUnit, UnitResult>>;

/** Measure how fast a downloaded model runs on each compute unit. */
export function SelfTestCard() {
  const t = useT(messages);
  const tc = useT(common);
  const { data: models = [] } = useQuery(modelsQuery);
  const { data: connection } = useQuery(connectionQuery);
  const [modelName, setModelName] = useState<string>("");
  const [results, setResults] = useState<Results>({});
  const [running, setRunning] = useState(false);
  const controller = useRef<AbortController | null>(null);

  const model = models.find((candidate) => candidate.name === modelName) ?? models[0];
  const units = model ? testableUnits(model) : [];
  const best = fastest(results);

  useEffect(() => () => controller.current?.abort(), []);

  const run = async () => {
    if (!model || !connection) return;
    const abort = new AbortController();
    controller.current = abort;
    setRunning(true);
    setResults(Object.fromEntries(units.map((unit) => [unit, { status: "waiting" }])));
    for (const unit of units) {
      if (abort.signal.aborted) break;
      setResults((current) => ({ ...current, [unit]: { status: "running" } }));
      let result: UnitResult;
      try {
        result = await measureUnit({ ...connection, model, unit, signal: abort.signal });
      } catch (error) {
        result = {
          status: "failed",
          message: abort.signal.aborted
            ? tc("cancel")
            : error instanceof Error
              ? error.message
              : String(error),
        };
      }
      setResults((current) => ({ ...current, [unit]: result }));
    }
    controller.current = null;
    setRunning(false);
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Gauge className="size-4 text-muted-foreground" />
          <CardTitle>{t("selfTest")}</CardTitle>
        </div>
        <CardDescription>{t("selfTestHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {models.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t("selfTestNoModel")}</p>
        ) : (
          <div className="flex flex-wrap items-center gap-2">
            <Select
              aria-label={t("selfTestModel")}
              className="w-80 font-mono"
              value={model?.name ?? ""}
              disabled={running}
              onChange={(event) => {
                setModelName(event.target.value);
                setResults({});
              }}
            >
              {models.map((candidate) => (
                <option key={candidate.name} value={candidate.name}>
                  {candidate.name}
                </option>
              ))}
            </Select>
            {running ? (
              <Button variant="secondary" onClick={() => controller.current?.abort()}>
                <Square />
                {tc("cancel")}
              </Button>
            ) : (
              <Button onClick={run} disabled={!connection}>
                <Play />
                {t("selfTestRun")}
              </Button>
            )}
          </div>
        )}

        {model?.runtime === "qairt" && (
          <p className="text-xs text-muted-foreground">{t("selfTestQairt")}</p>
        )}

        {Object.keys(results).length > 0 && (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-xs text-muted-foreground">
                <th className="py-1 font-medium">{t("selfTestUnit")}</th>
                <th className="py-1 font-medium">{t("selfTestSpeed")}</th>
                <th className="py-1 font-medium">{t("selfTestFirstToken")}</th>
              </tr>
            </thead>
            <tbody>
              {units.map((unit) => {
                const result = results[unit];
                const max = best?.tokensPerSecond ?? 1;
                return (
                  <tr key={unit} className="border-t">
                    <td className={cn("py-2 font-medium", UNIT_STYLE[unit].text)}>{tc(unit)}</td>
                    <td className="w-1/2 py-2 pr-4">
                      {result?.status === "done" ? (
                        <div className="flex items-center gap-2">
                          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
                            <div
                              className={cn("h-full rounded-full", UNIT_STYLE[unit].bar)}
                              style={{ width: `${(result.tokensPerSecond / max) * 100}%` }}
                            />
                          </div>
                          <span className="w-20 text-right font-mono text-xs tabular-nums">
                            {t("tokensPerSecond", {
                              value: formatNumber(result.tokensPerSecond, 1),
                            })}
                          </span>
                        </div>
                      ) : result?.status === "running" ? (
                        <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                          <Loader2 className="size-3.5 animate-spin" />
                          {t("selfTestRunning")}
                        </span>
                      ) : result?.status === "failed" ? (
                        <span className="text-xs text-destructive" title={result.message}>
                          {t("selfTestFailed")} · {result.message}
                        </span>
                      ) : (
                        <span className="text-xs text-muted-foreground">
                          {t("selfTestWaiting")}
                        </span>
                      )}
                    </td>
                    <td className="py-2 font-mono text-xs tabular-nums text-muted-foreground">
                      {result?.status === "done"
                        ? `${formatNumber(result.firstTokenMs / 1000, 2)} s`
                        : "—"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}

        {best && !running && units.length > 1 && (
          <p className="flex items-center gap-2 text-sm">
            <Trophy className={cn("size-4", UNIT_STYLE[best.unit].text)} />
            {t("selfTestBest", {
              unit: tc(best.unit),
              speed: formatNumber(best.tokensPerSecond, 1),
            })}
          </p>
        )}
        {Object.keys(results).length > 0 && (
          <p className="text-[11px] text-muted-foreground">{t("selfTestLoadNote")}</p>
        )}
      </CardContent>
    </Card>
  );
}
