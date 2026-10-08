import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Check, Download, Heart, Lock, Search } from "lucide-react";
import { useState } from "react";
import { ModelTypeBadge, RuntimeBadge } from "@/components/calcine/badges/runtime-badge";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { type Availability, useAvailability } from "@/features/library/availability";
import { percent } from "@/features/tasks/lib/format";
import { useDebounced } from "@/hooks/use-debounced";
import { useT } from "@/i18n";
import type { ModelReference, RemoteModel } from "@/lib/api";
import { formatCount, formatRelative } from "@/lib/format";
import { searchQuery } from "../api";
import { messages } from "../messages";

/** Search Hugging Face for GGUF models; downloading opens the precision picker. */
export function HuggingFaceSearch({
  onDownload,
}: {
  onDownload: (reference: ModelReference) => void;
}) {
  const t = useT(messages);
  const [input, setInput] = useState("");
  const query = useDebounced(input.trim(), 350);
  const results = useQuery(searchQuery(query));
  const availability = useAvailability();

  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <p className="text-xs text-muted-foreground">{t("hfDescription")}</p>
        <div className="flex h-8 w-80 items-center gap-2 rounded-md border bg-card px-2.5 focus-within:ring-2 focus-within:ring-ring">
          <Search className="size-3.5 text-muted-foreground" />
          <input
            value={input}
            onChange={(event) => setInput(event.target.value)}
            placeholder={t("hfPlaceholder")}
            aria-label={t("hfSearch")}
            spellCheck={false}
            className="w-full bg-transparent text-[13px] outline-none placeholder:text-muted-foreground"
          />
        </div>
      </div>

      {results.isPending ? (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          {Array.from({ length: 6 }, (_, index) => (
            <Skeleton key={String(index)} className="h-[104px]" />
          ))}
        </div>
      ) : results.isError ? (
        <ErrorState error={results.error} onRetry={() => results.refetch()} />
      ) : results.data.length === 0 ? (
        <p className="py-8 text-center text-sm text-muted-foreground">
          {t("hfNoResults", { query })}
        </p>
      ) : (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          {results.data.map((model) => (
            <ResultCard
              key={model.name}
              model={model}
              availability={availability(model.name)}
              onDownload={() =>
                onDownload({ name: model.name, hub: "hugging_face", precision: null })
              }
            />
          ))}
        </div>
      )}
    </section>
  );
}

function ResultCard({
  model,
  availability,
  onDownload,
}: {
  model: RemoteModel;
  availability: Availability;
  onDownload: () => void;
}) {
  const t = useT(messages);
  const slash = model.name.indexOf("/");
  const owner = model.name.slice(0, slash);
  const shortName = model.name.slice(slash + 1);

  return (
    <Card className="flex flex-col gap-3 p-4">
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium" title={model.name}>
            {shortName}
          </p>
          <p className="truncate font-mono text-[11px] text-muted-foreground">{owner}</p>
        </div>
        {availability.status === "installed" ? (
          <Button size="sm" variant="ghost" asChild>
            <Link to="/library">
              <Check className="text-success" />
              {t("inLibrary")}
            </Link>
          </Button>
        ) : availability.status === "downloading" ? (
          <span className="text-xs text-muted-foreground tabular-nums">
            {percent(availability.job) ?? 0}%
          </span>
        ) : (
          <Button
            size="sm"
            onClick={onDownload}
            aria-label={t("downloadModel", { name: model.name })}
          >
            <Download />
            {t("choose")}
          </Button>
        )}
      </div>
      <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
        <RuntimeBadge runtime="llama_cpp" />
        {model.modelType !== "unknown" && <ModelTypeBadge type={model.modelType} />}
        {model.gated && (
          <Badge tone="warning">
            <Lock />
            {t("gated")}
          </Badge>
        )}
        <span className="ml-auto flex items-center gap-2 tabular-nums">
          <span title={t("downloads", { count: model.downloads })}>
            <Download className="mr-0.5 inline size-3" />
            {formatCount(model.downloads)}
          </span>
          <span title={t("likes", { count: model.likes })}>
            <Heart className="mr-0.5 inline size-3" />
            {formatCount(model.likes)}
          </span>
          {model.updatedAt && (
            <span>{t("updated", { when: formatRelative(Date.parse(model.updatedAt)) })}</span>
          )}
        </span>
      </div>
      {availability.status === "downloading" && (
        <Progress value={percent(availability.job)} label={model.name} />
      )}
    </Card>
  );
}
