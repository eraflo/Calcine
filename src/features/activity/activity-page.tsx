import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Activity, Lock, Trash2 } from "lucide-react";
import { useState } from "react";
import { ConfirmDialog } from "@/components/calcine/feedback/confirm-dialog";
import { EmptyState } from "@/components/calcine/feedback/empty-state";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Page } from "@/components/calcine/layout/page";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Skeleton } from "@/components/ui/skeleton";
import { Tooltip } from "@/components/ui/tooltip";
import { gatewayQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import type { UsagePeriod, UsageSummary } from "@/lib/api";
import { usageQuery, useClearUsage } from "./api";
import { KpiRow } from "./components/kpi-row";
import { PrometheusCard } from "./components/prometheus-card";
import { ShareList } from "./components/share-list";
import { UsageChart } from "./components/usage-chart";
import type { Metric } from "./lib/metrics";
import { messages } from "./messages";

const PERIODS: UsagePeriod[] = ["day", "week", "month"];

/** Usage over time: requests, tokens, speed and energy, per app and model. */
export function ActivityPage() {
  const t = useT(messages);
  const [period, setPeriod] = useState<UsagePeriod>("day");
  // Kept when the period changes.
  const [metric, setMetric] = useState<Metric>("requests");
  const usage = useQuery(usageQuery(period));
  const summary = usage.data;
  const empty = summary && summary.totals.requests === 0 && summary.previous.requests === 0;

  return (
    <Page
      title={t("title")}
      description={t("description")}
      actions={
        <>
          <InFlight />
          <SegmentedControl<UsagePeriod>
            name="activity-period"
            label={t("period")}
            value={period}
            onChange={setPeriod}
            options={PERIODS.map((value) => ({ value, label: t(`period_${value}`) }))}
          />
          <ClearButton disabled={!summary || empty === true} />
        </>
      }
    >
      {usage.isError ? (
        <ErrorState error={usage.error} onRetry={() => usage.refetch()} />
      ) : !summary ? (
        <Loading />
      ) : empty ? (
        <EmptyState
          icon={Activity}
          title={t("emptyTitle")}
          action={
            <Button asChild variant="secondary" size="sm">
              <Link to="/server">{t("connectApp")}</Link>
            </Button>
          }
        >
          {t("emptyBody")}
        </EmptyState>
      ) : (
        <Overview summary={summary} chosen={metric} onChoose={setMetric} />
      )}
      <PrometheusCard />
      <p className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
        <Lock className="size-3 shrink-0" />
        {t("privacy")}
      </p>
    </Page>
  );
}

function Overview({
  summary,
  chosen,
  onChoose,
}: {
  summary: UsageSummary;
  chosen: Metric;
  onChoose: (metric: Metric) => void;
}) {
  const t = useT(messages);
  const metered = summary.totals.energyJoules !== null;
  // Energy can't be shown where it isn't measured.
  const metric = chosen === "energy" && !metered ? "requests" : chosen;
  const metrics: Metric[] = metered ? ["requests", "tokens", "energy"] : ["requests", "tokens"];
  // The gateway names these two in English.
  const clientLabel = (name: string) =>
    name === "No key" ? t("clientNoKey") : name === "Calcine" ? t("clientChat") : name;

  return (
    <>
      <KpiRow summary={summary} />
      <Card>
        <CardHeader className="flex-row items-center justify-between gap-3">
          <CardTitle>{t("overTime")}</CardTitle>
          <SegmentedControl<Metric>
            name="activity-metric"
            label={t("metric")}
            value={metric}
            onChange={onChoose}
            options={metrics.map((value) => ({ value, label: t(`metric_${value}`) }))}
          />
        </CardHeader>
        <CardContent>
          <UsageChart summary={summary} metric={metric} />
        </CardContent>
      </Card>
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t("byApp")}</CardTitle>
            <CardDescription>{t("byAppHint")}</CardDescription>
          </CardHeader>
          <CardContent>
            <ShareList shares={summary.byClient} metric={metric} label={clientLabel} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t("byModel")}</CardTitle>
          </CardHeader>
          <CardContent>
            <ShareList shares={summary.byModel} metric={metric} monospace />
          </CardContent>
        </Card>
      </div>
    </>
  );
}

/** Requests answering or waiting right now. */
function InFlight() {
  const t = useT(messages);
  const { data: gateway } = useQuery(gatewayQuery);
  const running = (gateway?.activeRequests ?? 0) + (gateway?.queuedRequests ?? 0);
  if (running === 0) return null;
  return (
    <span className="flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs text-muted-foreground">
      <span className="size-1.5 animate-pulse rounded-full bg-chart" />
      {t.plural("inFlight", running)}
    </span>
  );
}

function ClearButton({ disabled }: { disabled: boolean }) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  const clear = useClearUsage();
  return (
    <>
      <Tooltip content={t("clear")}>
        <span>
          <Button
            variant="ghost"
            size="icon"
            disabled={disabled}
            onClick={() => setOpen(true)}
            aria-label={t("clear")}
          >
            <Trash2 />
          </Button>
        </span>
      </Tooltip>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("clearTitle")}
        confirmLabel={t("clearConfirm")}
        destructive
        busy={clear.isPending}
        onConfirm={() => clear.mutate(undefined, { onSettled: () => setOpen(false) })}
      >
        {t("clearBody")}
      </ConfirmDialog>
    </>
  );
}

function Loading() {
  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-5">
        {[0, 1, 2, 3, 4].map((index) => (
          <Skeleton key={index} className="h-28" />
        ))}
      </div>
      <Skeleton className="h-72" />
    </div>
  );
}
