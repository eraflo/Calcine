import { Link } from "@tanstack/react-router";
import { Check, Copy, MessagesSquare, Trash2, X } from "lucide-react";
import { useState } from "react";
import { ModelTypeBadge, RuntimeBadge } from "@/components/calcine/badges/runtime-badge";
import { ConfirmDialog } from "@/components/calcine/feedback/confirm-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Tooltip } from "@/components/ui/tooltip";
import type { LocalModel, ModelKey } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { useRemoveModels } from "../api";

export function ModelCard({ model }: { model: LocalModel }) {
  const [pendingRemoval, setPendingRemoval] = useState<ModelKey | null>(null);
  const remove = useRemoveModels();
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
        <ModelTypeBadge type={model.modelType} />
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
                aria-label={`Remove precision ${precision}`}
              >
                <X />
              </button>
            )}
          </Badge>
        ))}
        <div className="ml-auto flex items-center gap-1.5">
          <Tooltip content="Remove from this device">
            <Button
              size="icon"
              variant="ghost"
              className="size-7 hover:text-destructive"
              onClick={() => setPendingRemoval({ name: model.name, precision: null })}
              aria-label={`Remove ${model.name}`}
            >
              <Trash2 />
            </Button>
          </Tooltip>
          <CopyIdButton id={model.name} />
          <Button size="sm" variant="default" asChild>
            <Link to="/chat">
              <MessagesSquare />
              Chat
            </Link>
          </Button>
        </div>
      </div>

      <ConfirmDialog
        open={pendingRemoval !== null}
        onOpenChange={(open) => !open && setPendingRemoval(null)}
        title={
          pendingRemoval?.precision
            ? `Remove the ${pendingRemoval.precision} precision?`
            : `Remove ${model.name}?`
        }
        confirmLabel="Remove"
        destructive
        busy={remove.isPending}
        onConfirm={confirm}
      >
        {pendingRemoval?.precision ? (
          <p>The other precisions of {model.name} stay in your library.</p>
        ) : (
          <p>This frees {formatBytes(model.sizeBytes)}. You can download it again anytime.</p>
        )}
        {remove.isError && <p className="mt-2 text-destructive">{remove.error.message}</p>}
      </ConfirmDialog>
    </Card>
  );
}

function CopyIdButton({ id }: { id: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    await navigator.clipboard.writeText(id);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <Tooltip content="Copy the model id to use in API requests">
      <Button size="sm" variant="ghost" onClick={copy}>
        {copied ? <Check className="text-success" /> : <Copy />}
        {copied ? "Copied" : "API id"}
      </Button>
    </Tooltip>
  );
}
