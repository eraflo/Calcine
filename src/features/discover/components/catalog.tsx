import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Check, Download, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { ModelTypeBadge, RuntimeBadge } from "@/components/calcine/badges/runtime-badge";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { type Availability, useAvailability } from "@/features/library/availability";
import { usePullByName } from "@/features/tasks/api";
import { percent } from "@/features/tasks/lib/format";
import type { HubModel } from "@/lib/api";
import { cn } from "@/lib/utils";
import { catalogQuery } from "../api";

/** Qualcomm AI Hub models with a pre-compiled NPU build. */
export function AiHubCatalog() {
  const [showAll, setShowAll] = useState(false);
  const [filter, setFilter] = useState("");
  const compatible = useQuery(catalogQuery(false));
  const everything = useQuery({ ...catalogQuery(true), enabled: showAll });
  const shown = showAll ? everything : compatible;
  const availability = useAvailability();

  const compatibleNames = useMemo(
    () => new Set(compatible.data?.models.map((model) => model.name)),
    [compatible.data],
  );
  const models = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    return (shown.data?.models ?? []).filter((model) => model.name.toLowerCase().includes(needle));
  }, [shown.data, filter]);

  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h2 className="text-sm font-medium">Qualcomm AI Hub</h2>
          <p className="text-xs text-muted-foreground">
            Pre-compiled for the Hexagon NPU
            {compatible.data?.chipset ? ` · showing models for ${compatible.data.chipset}` : ""}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex h-8 items-center gap-2 rounded-md border bg-card px-2.5 focus-within:ring-2 focus-within:ring-ring">
            <Search className="size-3.5 text-muted-foreground" />
            <input
              value={filter}
              onChange={(event) => setFilter(event.target.value)}
              placeholder="Filter models"
              aria-label="Filter models"
              className="w-40 bg-transparent text-[13px] outline-none placeholder:text-muted-foreground"
            />
          </div>
          <label className="flex h-8 cursor-pointer items-center gap-2 rounded-md border bg-card px-2.5 text-xs text-muted-foreground has-[:checked]:text-foreground">
            <input
              type="checkbox"
              checked={showAll}
              onChange={(event) => setShowAll(event.target.checked)}
              className="accent-[var(--primary)]"
            />
            All chipsets
          </label>
        </div>
      </div>

      {shown.isPending ? (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          {Array.from({ length: 6 }, (_, index) => (
            <Skeleton key={String(index)} className="h-[104px]" />
          ))}
        </div>
      ) : shown.isError ? (
        <ErrorState error={shown.error} onRetry={() => shown.refetch()} />
      ) : models.length === 0 ? (
        <p className="py-8 text-center text-sm text-muted-foreground">
          No model matches “{filter}”.
        </p>
      ) : (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          {models.map((model) => (
            <CatalogCard
              key={model.name}
              model={model}
              availability={availability(model.name)}
              otherChipsets={showAll && compatible.isSuccess && !compatibleNames.has(model.name)}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function CatalogCard({
  model,
  availability,
  otherChipsets,
}: {
  model: HubModel;
  availability: Availability;
  otherChipsets: boolean;
}) {
  const [owner, shortName] = splitName(model.name);
  const pull = usePullByName();

  return (
    <Card className={cn("flex flex-col gap-3 p-4", otherChipsets && "opacity-70")}>
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium">{shortName}</p>
          <p className="truncate font-mono text-[11px] text-muted-foreground">{owner}</p>
        </div>
        <CatalogAction
          name={model.name}
          availability={availability}
          onDownload={() => pull.pull(model.name)}
          busy={pull.isPending}
        />
      </div>
      <div className="flex flex-wrap items-center gap-1.5">
        <RuntimeBadge runtime="qairt" />
        <ModelTypeBadge type={model.modelType} />
        {otherChipsets && <Badge tone="warning">Built for other chipsets</Badge>}
      </div>
      {availability.status === "downloading" && (
        <Progress value={percent(availability.job)} label={`Downloading ${model.name}`} />
      )}
    </Card>
  );
}

function CatalogAction({
  name,
  availability,
  onDownload,
  busy,
}: {
  name: string;
  availability: Availability;
  onDownload: () => void;
  busy: boolean;
}) {
  switch (availability.status) {
    case "installed":
      return (
        <Button size="sm" variant="ghost" asChild>
          <Link to="/library">
            <Check className="text-success" />
            In library
          </Link>
        </Button>
      );
    case "downloading":
      return (
        <span className="text-xs text-muted-foreground tabular-nums">
          {percent(availability.job) ?? 0}%
        </span>
      );
    case "available":
      return (
        <Button size="sm" onClick={onDownload} disabled={busy} aria-label={`Download ${name}`}>
          <Download />
          Download
        </Button>
      );
  }
}

function splitName(name: string): [string, string] {
  const slash = name.indexOf("/");
  return slash === -1 ? ["", name] : [name.slice(0, slash), name.slice(slash + 1)];
}
