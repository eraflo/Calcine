import { CircleAlert } from "lucide-react";
import { useT } from "@/i18n";
import { messages } from "../../messages";

/** Why a reply failed, with a hint for the common causes. */
export function ErrorNote({ error }: { error: string }) {
  const t = useT(messages);
  return (
    <p className="flex items-start gap-1.5 text-sm text-destructive">
      <CircleAlert className="mt-0.5 size-4 shrink-0" />
      <span>
        {error}
        {error.includes("Model loading failed") && (
          <span className="mt-1 block text-xs text-muted-foreground">{t("loadFailedHint")}</span>
        )}
        {error.includes("context window") && (
          <span className="mt-1 block text-xs text-muted-foreground">{t("contextErrorHint")}</span>
        )}
      </span>
    </p>
  );
}
