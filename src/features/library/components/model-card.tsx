import { Link } from "@tanstack/react-router";
import { Check, Copy, Image, MessagesSquare, Trash2, Type, X } from "lucide-react";
import { useState } from "react";
import { RuntimeBadge } from "@/components/calcine/badges/runtime-badge";
import { ConfirmDialog } from "@/components/calcine/feedback/confirm-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Tooltip } from "@/components/ui/tooltip";
import { useChat } from "@/features/chat/store";
import { type MemoryFit, memoryFit } from "@/features/hardware/lib/fit";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { LocalModel, MemoryInfo, ModelKey } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { useRemoveModels, useSetModelType } from "../api";
import { messages } from "../messages";

const FIT_TONE: Record<MemoryFit, "success" | "warning" | "danger"> = {
  fits: "success",
  tight: "warning",
  too_big: "danger",
};

/** `memory` (free and total) rates whether the model fits. */
export function ModelCard({ model, memory }: { model: LocalModel; memory?: MemoryInfo }) {
  const t = useT(messages);
  const tc = useT(common);
  const [pendingRemoval, setPendingRemoval] = useState<ModelKey | null>(null);
  const remove = useRemoveModels();
  const setType = useSetModelType();
  // Removing the only precision removes the whole model.
  const removable = model.precisions.length > 1;

  const confirm = () => {
    if (!pendingRemoval) return;
    remove.mutate([pendingRemoval], { onSuccess: () => setPendingRemoval(null) });
  };

  return (
    <Card className="flex flex-col gap-3 p-4 transition-colors hover:border-muted-foreground/30">
      <div className="flex flex-wrap items-center gap-2">
        <span className="font-mono text-[13px] font-medium">{model.name}</span>
        <RuntimeBadge runtime={model.runtime} />
        {memory && <MemoryBadge model={model} memory={memory} />}
        <span className="ml-auto text-xs text-muted-foreground tabular-nums">
          {formatBytes(model.sizeBytes)}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {model.precisions.map((precision) => (
          <Badge key={precision} tone="outline" className="gap-0.5 pr-0.5 font-mono">
            {precision}
            {removable && (
              <button
                type="button"
                onClick={() => setPendingRemoval({ name: model.name, precision })}
                className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("removePrecision", { precision })}
              >
                <X />
              </button>
            )}
          </Badge>
        ))}
        <Tooltip content={model.modelType === "unknown" ? t("typeUnknownHint") : t("typeHint")}>
          <div>
            <SegmentedControl
              name={`type-${model.name}`}
              label={t("typeLabel")}
              value={model.modelType === "unknown" ? null : model.modelType}
              disabled={setType.isPending}
              onChange={(modelType) => setType.mutate({ name: model.name, modelType })}
              className="[&_label]:h-6 [&_label]:px-2"
              options={[
                { value: "llm", label: t("typeText"), icon: Type },
                { value: "vlm", label: t("typeVision"), icon: Image },
              ]}
            />
          </div>
        </Tooltip>
        <div className="ml-auto flex items-center gap-1.5">
          <Tooltip content={t("removeFromDevice")}>
            <Button
              size="icon"
              variant="ghost"
              className="size-7 hover:text-destructive"
              onClick={() => setPendingRemoval({ name: model.name, precision: null })}
              aria-label={t("removeModel", { name: model.name })}
            >
              <Trash2 />
            </Button>
          </Tooltip>
          <CopyIdButton id={model.name} />
          <Button size="sm" variant="default" asChild>
            <Link
              to="/chat"
              onClick={() => useChat.setState({ lastModelId: model.name, activeId: null })}
            >
              <MessagesSquare />
              {t("chat")}
            </Link>
          </Button>
        </div>
      </div>
      {setType.isError && <p className="text-xs text-destructive">{setType.error.message}</p>}

      <ConfirmDialog
        open={pendingRemoval !== null}
        onOpenChange={(open) => !open && setPendingRemoval(null)}
        title={
          pendingRemoval?.precision
            ? t("removePrecisionTitle", { precision: pendingRemoval.precision })
            : t("removeModelTitle", { name: model.name })
        }
        confirmLabel={tc("remove")}
        destructive
        busy={remove.isPending}
        onConfirm={confirm}
      >
        {pendingRemoval?.precision ? (
          <p>{t("removePrecisionBody", { name: model.name })}</p>
        ) : (
          <p>{t("removeModelBody", { size: formatBytes(model.sizeBytes) })}</p>
        )}
        {remove.isError && <p className="mt-2 text-destructive">{remove.error.message}</p>}
      </ConfirmDialog>
    </Card>
  );
}

/** Whether one precision of the model fits in memory right now. */
function MemoryBadge({ model, memory }: { model: LocalModel; memory: MemoryInfo }) {
  const t = useT(messages);
  const fit = memoryFit(model.sizeBytes / Math.max(1, model.precisions.length), memory);
  return (
    <Tooltip content={t(`fitHint_${fit}`)}>
      <Badge tone={FIT_TONE[fit]} tabIndex={0}>
        {t(`fit_${fit}`)}
      </Badge>
    </Tooltip>
  );
}

function CopyIdButton({ id }: { id: string }) {
  const t = useT(messages);
  const tc = useT(common);
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    await navigator.clipboard.writeText(id);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <Tooltip content={t("apiIdHint")}>
      <Button size="sm" variant="ghost" onClick={copy}>
        {copied ? <Check className="text-success" /> : <Copy />}
        {copied ? tc("copied") : t("apiId")}
      </Button>
    </Tooltip>
  );
}
