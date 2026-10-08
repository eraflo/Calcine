import { useQuery } from "@tanstack/react-query";
import { Download, HardDrive, Loader2, TriangleAlert } from "lucide-react";
import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { hardwareQuery } from "@/features/hardware/api";
import { fitsOnDisk, type MemoryFit, memoryFit } from "@/features/hardware/lib/fit";
import { useStartPull } from "@/features/tasks/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { ModelReference, ModelType, RemotePrecision } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";
import { detailsQuery } from "../api";
import { canListPrecisions } from "../lib/hubs";
import { messages } from "../messages";

type TypeChoice = "auto" | Exclude<ModelType, "unknown">;

const FIT_TONE: Record<MemoryFit, "success" | "warning" | "danger"> = {
  fits: "success",
  tight: "warning",
  too_big: "danger",
};

/**
 * Pick a precision (with its size and whether it fits this device) and the
 * model type, then start the download.
 */
export function DownloadDialog({
  reference,
  onOpenChange,
}: {
  reference: ModelReference | null;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const listable = reference !== null && canListPrecisions(reference);
  const details = useQuery({
    ...detailsQuery(reference ?? { name: "", hub: "auto", precision: null }),
    enabled: listable,
  });
  const { data: hardware } = useQuery(hardwareQuery);
  const startPull = useStartPull();
  const [precision, setPrecision] = useState<string | null>(null);
  const [type, setType] = useState<TypeChoice>("auto");

  const precisions = details.data?.precisions ?? [];
  // Preselect what was typed (`repo:Q8_0`), else the recommended precision.
  useEffect(() => {
    if (!reference) return;
    setType("auto");
    setPrecision(reference.precision);
  }, [reference]);
  useEffect(() => {
    if (precision !== null || precisions.length === 0) return;
    setPrecision(precisions.find((candidate) => candidate.recommended)?.name ?? null);
  }, [precision, precisions]);

  const chosen = precisions.find((candidate) => candidate.name === precision);
  const disk = hardware?.modelsDisk;
  const diskShort = chosen && disk ? !fitsOnDisk(chosen.sizeBytes, disk) : false;
  const detectedType = details.data?.modelType;

  const download = () => {
    if (!reference) return;
    startPull.mutate(
      {
        reference: { ...reference, precision: precision ?? reference.precision },
        modelType: type === "auto" ? null : type,
        localPath: null,
      },
      { onSuccess: () => onOpenChange(false) },
    );
  };

  return (
    <Dialog open={reference !== null} onOpenChange={onOpenChange}>
      <DialogContent className="top-1/2 flex max-h-[85vh] max-w-xl -translate-y-1/2 flex-col">
        <div className="flex flex-col gap-1 border-b px-5 py-4">
          <DialogTitle className="truncate text-base font-semibold">
            {t("dialogTitle", { name: reference?.name ?? "" })}
          </DialogTitle>
          <DialogDescription className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            {reference && <Badge tone="info">{t(`hub_${reference.hub}`)}</Badge>}
            {details.data?.gated && <Badge tone="warning">{t("gated")}</Badge>}
            {detectedType && detectedType !== "unknown" && <Badge>{tc(detectedType)}</Badge>}
          </DialogDescription>
        </div>

        <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto overscroll-contain px-5 py-4">
          {details.data?.gated && (
            <p className="flex items-start gap-2 rounded-md border border-warning/40 bg-warning/10 p-2.5 text-xs">
              <TriangleAlert className="mt-0.5 size-3.5 shrink-0 text-warning" />
              {t("gatedHint")}
            </p>
          )}

          <Field label={t("precision")} hint={listable ? t("precisionHint") : undefined}>
            {!listable ? (
              <p className="text-sm text-muted-foreground">{t("precisionsUnavailable")}</p>
            ) : details.isPending ? (
              <p className="flex items-center gap-2 text-sm text-muted-foreground">
                <Loader2 className="size-4 animate-spin" />
                {t("precisionsLoading")}
              </p>
            ) : details.isError ? (
              <p className="text-sm text-destructive">
                {t("precisionsError", { message: details.error.message })}
              </p>
            ) : (
              <PrecisionList
                precisions={precisions}
                value={precision}
                onChange={setPrecision}
                fitOf={(size) => (hardware ? memoryFit(size, hardware.memory) : null)}
              />
            )}
          </Field>

          <Field label={t("typeLabel")} hint={t("typeHint")}>
            <SegmentedControl
              name="model-type"
              label={t("typeLabel")}
              value={type}
              onChange={setType}
              className="self-start"
              options={[
                { value: "auto", label: t("typeAuto") },
                { value: "llm", label: t("typeLlm") },
                { value: "vlm", label: t("typeVlm") },
              ]}
            />
          </Field>
        </div>

        <div className="flex flex-col gap-3 border-t px-5 py-3">
          {disk && (
            <p
              className={cn(
                "flex items-center gap-2 text-xs",
                diskShort ? "text-destructive" : "text-muted-foreground",
              )}
            >
              <HardDrive className="size-3.5 shrink-0" />
              {diskShort && chosen
                ? t("diskShort", {
                    needed: formatBytes(chosen.sizeBytes),
                    free: formatBytes(disk.availableBytes),
                    path: disk.path,
                  })
                : t("diskOk", { free: formatBytes(disk.availableBytes) })}
            </p>
          )}
          {startPull.isError && (
            <p className="text-xs text-destructive">{startPull.error.message}</p>
          )}
          <div className="flex justify-end gap-2">
            <DialogClose asChild>
              <Button variant="ghost">{tc("cancel")}</Button>
            </DialogClose>
            <Button onClick={download} disabled={!reference || diskShort || startPull.isPending}>
              <Download />
              {t("startDownload")}
              {chosen && (
                <span className="font-mono text-xs opacity-80">
                  {formatBytes(chosen.sizeBytes)}
                </span>
              )}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function PrecisionList({
  precisions,
  value,
  onChange,
  fitOf,
}: {
  precisions: readonly RemotePrecision[];
  value: string | null;
  onChange: (value: string) => void;
  fitOf: (sizeBytes: number) => MemoryFit | null;
}) {
  const t = useT(messages);
  const tc = useT(common);
  return (
    <fieldset className="flex flex-col overflow-hidden rounded-md border">
      <legend className="sr-only">{t("precision")}</legend>
      {precisions.map((option) => {
        const memory = fitOf(option.sizeBytes);
        const selected = value === option.name;
        return (
          <label
            key={option.name}
            className={cn(
              "flex cursor-pointer items-center gap-3 border-b px-3 py-2 text-sm last:border-b-0 has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring has-[:focus-visible]:ring-inset",
              selected ? "bg-accent" : "hover:bg-accent/50",
            )}
          >
            <input
              type="radio"
              name="precision"
              value={option.name}
              checked={selected}
              onChange={() => onChange(option.name)}
              className="accent-[var(--primary)]"
            />
            <span className="w-24 font-mono text-[13px] font-medium">{option.name}</span>
            {option.recommended && <Badge tone="npu">{tc("recommended")}</Badge>}
            <span className="ml-auto flex items-center gap-2">
              {memory && <Badge tone={FIT_TONE[memory]}>{t(`fit_${memory}`)}</Badge>}
              <span className="w-16 text-right font-mono text-xs text-muted-foreground tabular-nums">
                {formatBytes(option.sizeBytes)}
              </span>
            </span>
          </label>
        );
      })}
    </fieldset>
  );
}
