import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Compass, Library, Plus, RefreshCw } from "lucide-react";
import { ErrorState } from "@/components/calcine/error-state";
import { EmptyState, Page } from "@/components/calcine/page";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { formatBytes, totalBytes } from "@/lib/format";
import { cn } from "@/lib/utils";
import { modelsQuery } from "./api";
import { ModelCard } from "./model-card";

export function LibraryPage() {
  const { data: models, error, isPending, isFetching, refetch } = useQuery(modelsQuery);

  const summary = models
    ? `${models.length} ${models.length === 1 ? "model" : "models"} · ${formatBytes(
        totalBytes(models.map((model) => model.sizeBytes)),
      )} on disk`
    : "Models downloaded to this device";

  return (
    <Page
      title="Library"
      description={summary}
      actions={
        <>
          <Button variant="ghost" size="icon" onClick={() => refetch()} aria-label="Refresh">
            <RefreshCw className={cn(isFetching && "animate-spin")} />
          </Button>
          <Button variant="default" asChild>
            <Link to="/discover">
              <Plus />
              Add model
            </Link>
          </Button>
        </>
      }
    >
      {isPending ? (
        <div className="flex flex-col gap-3">
          {[0, 1, 2].map((key) => (
            <Skeleton key={key} className="h-[92px]" />
          ))}
        </div>
      ) : error ? (
        <ErrorState error={error} onRetry={() => refetch()} />
      ) : models.length === 0 ? (
        <EmptyState
          icon={Library}
          title="Start your library"
          action={
            <Button variant="default" asChild>
              <Link to="/discover">
                <Compass />
                Discover models
              </Link>
            </Button>
          }
        >
          Download a model from Qualcomm AI Hub or Hugging Face to run it on your NPU.
        </EmptyState>
      ) : (
        <div className="flex flex-col gap-3">
          {models.map((model) => (
            <ModelCard key={model.name} model={model} />
          ))}
        </div>
      )}
    </Page>
  );
}
