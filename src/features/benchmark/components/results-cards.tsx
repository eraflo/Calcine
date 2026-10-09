import { useMutation, useQuery } from "@tanstack/react-query";
import { Check, FileDown, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { messages as chatMessages } from "@/features/chat/messages";
import { requestsQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { BenchResult } from "@/lib/api";
import { commands, unwrap } from "@/lib/api";
import { formatCount, formatNumber, formatRelative } from "@/lib/format";
import { cn } from "@/lib/utils";
import { useForgetResults } from "../api";
import { leaderboard, median, type Session, toCsv, usageByModel } from "../lib/results";
import { messages } from "../messages";
import { MetricBars, UNIT_COLOR } from "./metric-bars";

/** One session: generation, prompt reading and first token, per unit. */
export function SessionCard({ session, latest }: { session: Session; latest: boolean }) {
  const t = useT(messages);
  const tchat = useT(chatMessages);
  const failures = session.results.filter((result) => result.error);
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex flex-wrap items-center gap-2">
          {latest ? t("latest") : t("resultsTitle")}
          <span className="font-mono text-xs font-normal text-muted-foreground">
            {session.model}
          </span>
          <span className="text-xs font-normal text-muted-foreground">
            · {formatRelative(session.startedAtMs)}
          </span>
        </CardTitle>
        <CardDescription>
          {t("setup", {
            prompt: String(session.promptTokens),
            generated: String(session.generatedTokens),
            runs: String(session.repetitions),
            power: tchat(`power_${session.powerMode}` as "power_burst"),
          })}
          {session.specType &&
            ` · ${t("withSpec", {
              method: [tchat(`spec_${session.specType}` as "spec_off"), session.draftModel]
                .filter(Boolean)
                .join(" + "),
            })}`}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <MetricBars results={session.results} metric="decodeTps" />
        <MetricBars results={session.results} metric="prefillTps" />
        <MetricBars results={session.results} metric="ttftMs" />
        {failures.length > 0 && (
          <ul className="flex flex-col gap-1 border-t pt-3">
            {failures.map((result) => (
              <li key={result.id} className="text-xs text-muted-foreground">
                <span className={cn("font-medium", UNIT_COLOR[result.unit].text)}>
                  {result.unit.toUpperCase()}
                </span>{" "}
                {result.error}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}

/** Each model's best generation speed, across the history. */
export function LeaderboardCard({ history }: { history: readonly BenchResult[] }) {
  const t = useT(messages);
  const tc = useT(common);
  const board = leaderboard(history);
  if (board.length < 2) return null;
  const top = median(board[0] as BenchResult, "decodeTps") ?? 1;
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("leaderboardTitle")}</CardTitle>
        <CardDescription>{t("leaderboardHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2.5">
        {board.map((result) => {
          const value = median(result, "decodeTps") ?? 0;
          return (
            <div
              key={result.id}
              className="grid grid-cols-[minmax(0,16rem)_1fr_7rem] items-center gap-3"
            >
              <span className="truncate font-mono text-xs" title={result.model}>
                {result.model}
                {result.specType && (
                  <span className="ml-1.5 font-sans text-[10px] text-info">{result.specType}</span>
                )}
              </span>
              <div className="h-2.5 overflow-hidden rounded-full bg-muted">
                <div
                  className={cn("h-full rounded-full", UNIT_COLOR[result.unit].bar)}
                  style={{ width: `${Math.max(2, (value / top) * 100)}%` }}
                />
              </div>
              <span className="text-right text-xs tabular-nums">
                <span className="font-medium">{formatNumber(value, value < 100 ? 1 : 0)}</span>{" "}
                <span className={UNIT_COLOR[result.unit].text}>{tc(result.unit)}</span>
              </span>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}

/** Past sessions: show one, delete it, or export everything as CSV. */
export function HistoryCard({
  sessions,
  history,
  selected,
  onSelect,
}: {
  sessions: readonly Session[];
  history: readonly BenchResult[];
  selected: string | null;
  onSelect: (id: string) => void;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const forget = useForgetResults();
  const [saved, setSaved] = useState(false);
  const exportCsv = useMutation({
    mutationFn: () =>
      unwrap(() => commands.saveExport("Calcine benchmarks", toCsv(history), "csv")),
    onSuccess: (path) => setSaved(path !== null),
  });
  useEffect(() => {
    if (!saved) return;
    const timer = setTimeout(() => setSaved(false), 2000);
    return () => clearTimeout(timer);
  }, [saved]);

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle>{t("historyTitle")}</CardTitle>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => exportCsv.mutate()}
          disabled={exportCsv.isPending}
        >
          {saved ? <Check className="text-success" /> : <FileDown />}
          {saved ? t("exported") : t("exportCsv")}
        </Button>
      </CardHeader>
      <CardContent>
        <ul className="flex flex-col divide-y">
          {sessions.map((session) => (
            <li
              key={session.id}
              className={cn(
                "group flex items-center gap-3 py-2",
                session.id === selected && "font-medium",
              )}
            >
              <button
                type="button"
                onClick={() => onSelect(session.id)}
                className="flex min-w-0 flex-1 items-center gap-3 text-left"
              >
                <span className="w-24 shrink-0 text-xs text-muted-foreground">
                  {formatRelative(session.startedAtMs)}
                </span>
                <span className="min-w-0 flex-1 truncate font-mono text-xs">{session.model}</span>
                <span className="flex shrink-0 gap-1">
                  {session.results.map((result) => {
                    const decode = median(result, "decodeTps");
                    return (
                      <Badge
                        key={result.id}
                        tone={
                          result.measure
                            ? result.unit === "hybrid"
                              ? "info"
                              : result.unit
                            : "danger"
                        }
                        className="tabular-nums"
                      >
                        {tc(result.unit)} {decode === null ? "×" : formatNumber(decode, 0)}
                      </Badge>
                    );
                  })}
                </span>
              </button>
              <button
                type="button"
                onClick={() => forget.mutate(session.results.map((result) => result.id))}
                className="rounded-sm p-1 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:text-destructive focus-visible:opacity-100"
                aria-label={t("deleteSession")}
              >
                <Trash2 className="size-3.5" />
              </button>
            </li>
          ))}
        </ul>
        {exportCsv.isError && (
          <p className="mt-2 text-xs text-destructive">{exportCsv.error.message}</p>
        )}
      </CardContent>
    </Card>
  );
}

/** Speed with real prompts, from the request log since Calcine started. */
export function UsageCard() {
  const t = useT(messages);
  const { data: entries = [] } = useQuery(requestsQuery);
  const usage = usageByModel(entries);
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("usageTitle")}</CardTitle>
        <CardDescription>{t("usageHint")}</CardDescription>
      </CardHeader>
      <CardContent>
        {usage.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t("usageEmpty")}</p>
        ) : (
          <ul className="flex flex-col divide-y">
            {usage.map((row) => (
              <li key={row.model} className="flex items-center gap-3 py-2 text-xs">
                <span className="min-w-0 flex-1 truncate font-mono">{row.model}</span>
                <span className="text-muted-foreground">
                  {t.plural("requests", row.requests, { count: formatCount(row.requests) })}
                </span>
                {row.firstTokenMs !== null && (
                  <span className="w-36 text-right text-muted-foreground tabular-nums">
                    {t("firstTokenShort", {
                      value: t("ms", { value: formatNumber(row.firstTokenMs, 0) }),
                    })}
                  </span>
                )}
                {row.tokensPerSecond !== null && (
                  <span className="w-20 text-right font-medium tabular-nums">
                    {t("speedShort", { value: formatNumber(row.tokensPerSecond, 1) })}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
