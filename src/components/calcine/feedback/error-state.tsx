import { MonitorSmartphone, PackageX, TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CalcineError } from "@/lib/api";
import { EmptyState } from "./empty-state";

/** Friendly rendering of a failed backend call, with a retry when it can help. */
export function ErrorState({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  const kind = error instanceof CalcineError ? error.kind : undefined;

  if (kind === "not_in_tauri") {
    return (
      <EmptyState icon={MonitorSmartphone} title="Open Calcine in the desktop app">
        This page needs Calcine's backend, which isn't available in a regular browser. Run{" "}
        <code className="font-mono text-foreground">bun run app</code> or{" "}
        <code className="font-mono text-foreground">bun run app:mock</code>.
      </EmptyState>
    );
  }

  if (kind === "runtime_not_found") {
    return (
      <EmptyState icon={PackageX} title="GenieX isn't installed">
        Calcine's installer sets up GenieX for you. In development, install it from the{" "}
        <span className="text-foreground">GenieX releases</span> page, then refresh.
      </EmptyState>
    );
  }

  return (
    <EmptyState
      icon={TriangleAlert}
      title="Something went wrong"
      action={
        onRetry && (
          <Button size="sm" onClick={onRetry}>
            Try again
          </Button>
        )
      }
    >
      {error instanceof Error ? error.message : String(error)}
    </EmptyState>
  );
}
