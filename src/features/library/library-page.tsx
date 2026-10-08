import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Compass, FolderInput, Library, Plus, RefreshCw } from "lucide-react";
import { useState } from "react";
import { EmptyState } from "@/components/calcine/feedback/empty-state";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Page } from "@/components/calcine/layout/page";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { hardwareQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { formatBytes, totalBytes } from "@/lib/format";
import { cn } from "@/lib/utils";
import { modelsQuery } from "./api";
import { ImportDialog } from "./components/import-dialog";
import { ModelCard } from "./components/model-card";
import { useFileDrop } from "./lib/use-file-drop";
import { messages } from "./messages";

export function LibraryPage() {
  const t = useT(messages);
  const tc = useT(common);
  const { data: models, error, isPending, isFetching, refetch } = useQuery(modelsQuery);
  const { data: hardware } = useQuery(hardwareQuery);
  const [importing, setImporting] = useState<{ path: string | null } | null>(null);
  const dragging = useFileDrop((path) => setImporting({ path }));

  const summary = models
    ? t.plural("summary", models.length, {
        size: formatBytes(totalBytes(models.map((model) => model.sizeBytes))),
      })
    : t("emptyDescription");

  return (
    <Page
      title={t("title")}
      description={summary}
      actions={
        <>
          <Button variant="ghost" size="icon" onClick={() => refetch()} aria-label={tc("refresh")}>
            <RefreshCw className={cn(isFetching && "animate-spin")} />
          </Button>
          <Button variant="secondary" onClick={() => setImporting({ path: null })}>
            <FolderInput />
            {t("import")}
          </Button>
          <Button variant="default" asChild>
            <Link to="/discover">
              <Plus />
              {t("addModel")}
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
          title={t("startTitle")}
          action={
            <div className="flex gap-2">
              <Button variant="default" asChild>
                <Link to="/discover">
                  <Compass />
                  {t("discover")}
                </Link>
              </Button>
              <Button variant="secondary" onClick={() => setImporting({ path: null })}>
                <FolderInput />
                {t("import")}
              </Button>
            </div>
          }
        >
          {t("startHint")}
        </EmptyState>
      ) : (
        <div className="flex flex-col gap-3">
          {models.map((model) => (
            <ModelCard key={model.name} model={model} memory={hardware?.memory} />
          ))}
        </div>
      )}

      {dragging && (
        <div className="pointer-events-none fixed inset-0 z-40 flex items-center justify-center bg-background/70 backdrop-blur-sm">
          <div className="flex flex-col items-center gap-2 rounded-xl border-2 border-dashed border-primary px-12 py-10">
            <FolderInput className="size-8 text-primary" />
            <p className="text-sm font-medium">{t("dropTitle")}</p>
            <p className="text-xs text-muted-foreground">{t("dropHint")}</p>
          </div>
        </div>
      )}

      <ImportDialog
        open={importing !== null}
        initialPath={importing?.path ?? null}
        onOpenChange={(open) => !open && setImporting(null)}
      />
    </Page>
  );
}
