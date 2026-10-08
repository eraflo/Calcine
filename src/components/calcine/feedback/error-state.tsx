import { MonitorSmartphone, PackageX, TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { CalcineError } from "@/lib/api";
import { messages } from "../messages";
import { EmptyState } from "./empty-state";

/** Friendly rendering of a failed backend call, with a retry when it can help. */
export function ErrorState({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  const t = useT(messages);
  const tc = useT(common);
  const kind = error instanceof CalcineError ? error.kind : undefined;

  if (kind === "not_in_tauri") {
    return (
      <EmptyState icon={MonitorSmartphone} title={t("desktopOnly")}>
        {t.rich("desktopOnlyHint", {
          code: (text) => <code className="font-mono text-foreground">{text}</code>,
        })}
      </EmptyState>
    );
  }

  if (kind === "runtime_not_found") {
    return (
      <EmptyState icon={PackageX} title={t("geniexMissing")}>
        {t.rich("geniexMissingHint", {
          strong: (text) => <span className="text-foreground">{text}</span>,
        })}
      </EmptyState>
    );
  }

  return (
    <EmptyState
      icon={TriangleAlert}
      title={t("somethingWrong")}
      action={
        onRetry && (
          <Button size="sm" onClick={onRetry}>
            {tc("retry")}
          </Button>
        )
      }
    >
      {error instanceof Error ? error.message : String(error)}
    </EmptyState>
  );
}
