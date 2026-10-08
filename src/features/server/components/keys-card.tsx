import { useQuery } from "@tanstack/react-query";
import { KeyRound, Plus, ShieldAlert, Trash2 } from "lucide-react";
import { useState } from "react";
import { ConfirmDialog } from "@/components/calcine/feedback/confirm-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import type { ApiKeyInfo } from "@/lib/api";
import { formatRelative } from "@/lib/format";
import { apiKeysQuery, gatewayQuery, useRequireApiKey, useRevokeApiKey } from "../api";
import { CreateKeyDialog } from "./create-key-dialog";

export function KeysCard() {
  const { data: keys = [] } = useQuery(apiKeysQuery);
  const { data: status } = useQuery(gatewayQuery);
  const requireKey = useRequireApiKey();
  const revoke = useRevokeApiKey();
  const [creating, setCreating] = useState(false);
  const [revoking, setRevoking] = useState<ApiKeyInfo | null>(null);
  const required = status?.requireApiKey ?? true;

  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between gap-4">
        <div className="flex flex-col gap-1">
          <CardTitle>API keys</CardTitle>
          <CardDescription>
            One key per app, so you can see who uses the API and revoke access.
          </CardDescription>
        </div>
        <Button variant="default" size="sm" onClick={() => setCreating(true)}>
          <Plus />
          New key
        </Button>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <div className="flex items-start gap-3 rounded-md border bg-background p-3">
          <Switch
            id="require-api-key"
            checked={required}
            onCheckedChange={(checked) => requireKey.mutate(checked)}
            disabled={requireKey.isPending}
          />
          <span className="flex flex-col gap-0.5">
            <label htmlFor="require-api-key" className="text-sm font-medium">
              Require an API key
            </label>
            <span className="text-xs text-muted-foreground">
              {required ? (
                "Requests without a valid key are refused."
              ) : (
                <span className="flex items-center gap-1 text-warning">
                  <ShieldAlert className="size-3.5" />
                  Any program on this PC can use your models. Managing models still needs a key.
                </span>
              )}
            </span>
          </span>
        </div>

        {keys.length === 0 ? (
          <p className="py-4 text-center text-sm text-muted-foreground">
            No keys yet. Create one for each app that should use Calcine.
          </p>
        ) : (
          <ul className="flex flex-col divide-y rounded-md border">
            {keys.map((key) => (
              <li key={key.id} className="flex items-center gap-3 px-3 py-2.5">
                <KeyRound className="size-4 shrink-0 text-muted-foreground" />
                <div className="min-w-0 flex-1">
                  <p className="flex flex-wrap items-center gap-2 text-sm">
                    <span className="font-medium">{key.name}</span>
                    <code className="font-mono text-xs text-muted-foreground">{key.preview}</code>
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Created {formatRelative(key.createdAtMs)} ·{" "}
                    {key.lastUsedAtMs ? `used ${formatRelative(key.lastUsedAtMs)}` : "never used"}
                  </p>
                </div>
                <div className="flex flex-wrap justify-end gap-1">
                  {key.scopes.includes("inference") && <Badge>Run models</Badge>}
                  {key.scopes.includes("manage") && <Badge tone="info">Manage models</Badge>}
                  {key.allowLocalFiles && <Badge tone="warning">Local files</Badge>}
                </div>
                <Button
                  size="icon"
                  variant="ghost"
                  className="size-7 hover:text-destructive"
                  onClick={() => setRevoking(key)}
                  aria-label={`Revoke ${key.name}`}
                >
                  <Trash2 />
                </Button>
              </li>
            ))}
          </ul>
        )}
      </CardContent>

      <CreateKeyDialog open={creating} onOpenChange={setCreating} />
      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
        title={`Revoke ${revoking?.name ?? "this key"}?`}
        confirmLabel="Revoke"
        destructive
        busy={revoke.isPending}
        onConfirm={() =>
          revoking && revoke.mutate(revoking.id, { onSuccess: () => setRevoking(null) })
        }
      >
        <p>Apps using it lose access right away. This can't be undone.</p>
      </ConfirmDialog>
    </Card>
  );
}
