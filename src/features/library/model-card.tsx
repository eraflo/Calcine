import { Link } from "@tanstack/react-router";
import { Check, Copy, MessagesSquare } from "lucide-react";
import { useState } from "react";
import { ModelTypeBadge, RuntimeBadge } from "@/components/calcine/runtime-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Tooltip } from "@/components/ui/tooltip";
import type { LocalModel } from "@/lib/api";
import { formatBytes } from "@/lib/format";

export function ModelCard({ model }: { model: LocalModel }) {
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
          <Badge key={precision} tone="outline" className="font-mono">
            {precision}
          </Badge>
        ))}
        <div className="ml-auto flex items-center gap-1.5">
          <CopyIdButton id={model.name} />
          <Button size="sm" variant="default" asChild>
            <Link to="/chat">
              <MessagesSquare />
              Chat
            </Link>
          </Button>
        </div>
      </div>
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
