import * as Collapsible from "@radix-ui/react-collapsible";
import { useQuery } from "@tanstack/react-query";
import { ChevronRight, Play, Square } from "lucide-react";
import { useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { StatusDot } from "@/components/calcine/feedback/status-dot";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import type { GatewayStatus } from "@/lib/api";
import { gatewayQuery, serverLogsQuery, useStartServer, useStopServer } from "../api";
import { describeStatus } from "../lib/status";

/** Is the API reachable, and is GenieX running behind it? */
export function StatusCard() {
  const { data: status } = useQuery(gatewayQuery);
  const summary = describeStatus(status);
  const start = useStartServer();
  const stop = useStopServer();
  const serverState = status?.server.state;
  const running = serverState === "ready" || serverState === "starting";

  return (
    <Card>
      <CardContent className="flex flex-col gap-4">
        <div className="flex items-start gap-3">
          <StatusDot tone={summary.tone} className="mt-1.5 size-2.5" />
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium">{summary.label}</p>
            <p className="text-xs text-muted-foreground">{summary.detail}</p>
          </div>
          {running ? (
            <Button onClick={() => stop.mutate()} disabled={stop.isPending}>
              <Square />
              Stop GenieX
            </Button>
          ) : (
            <Button
              variant="default"
              onClick={() => start.mutate()}
              disabled={start.isPending || !status?.listening}
            >
              <Play />
              Start GenieX
            </Button>
          )}
        </div>

        {status && <Endpoint status={status} />}
        {(start.isError || stop.isError) && (
          <p className="text-xs text-destructive">{(start.error ?? stop.error)?.message}</p>
        )}
        <ServerLogs />
      </CardContent>
    </Card>
  );
}

function Endpoint({ status }: { status: GatewayStatus }) {
  return (
    <div className="flex items-center gap-2 rounded-md border bg-background px-3 py-2">
      <span className="text-xs text-muted-foreground">Base URL</span>
      <code className="min-w-0 flex-1 truncate font-mono text-[13px]">{status.baseUrl}</code>
      <span className="text-xs text-muted-foreground tabular-nums">
        {status.activeRequests} running · {status.queuedRequests} waiting
      </span>
      <CopyButton value={status.baseUrl} />
    </div>
  );
}

function ServerLogs() {
  const [open, setOpen] = useState(false);
  const { data: lines = [] } = useQuery({ ...serverLogsQuery, enabled: open });
  return (
    <Collapsible.Root open={open} onOpenChange={setOpen}>
      <Collapsible.Trigger className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
        <ChevronRight
          className={
            open ? "size-3.5 rotate-90 transition-transform" : "size-3.5 transition-transform"
          }
        />
        GenieX output
      </Collapsible.Trigger>
      <Collapsible.Content>
        <pre className="mt-2 max-h-56 overflow-auto overscroll-contain rounded-md border bg-background p-3 font-mono text-[11px] leading-relaxed text-muted-foreground">
          {lines.length ? lines.join("\n") : "Nothing yet. GenieX starts on the first request."}
        </pre>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
