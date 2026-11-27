import { ClipboardList } from "lucide-react";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { QUICK_ACTIONS, type QuickAction } from "../lib/prompt";
import { messages } from "../messages";

/** The copied text, to include or not, and one-click requests about it. */
export function CopiedBar({
  copied,
  included,
  onIncludedChange,
  onAction,
  disabled,
}: {
  copied: string;
  included: boolean;
  onIncludedChange: (included: boolean) => void;
  onAction: (action: QuickAction) => void;
  disabled: boolean;
}) {
  const t = useT(messages);
  return (
    <div className="flex flex-wrap items-center gap-1.5 border-b px-3 py-2">
      <button
        type="button"
        aria-pressed={included}
        title={t("includeCopied")}
        onClick={() => onIncludedChange(!included)}
        className={cn(
          "flex max-w-64 items-center gap-1.5 rounded-md border px-2 py-1 text-xs transition-colors",
          included
            ? "border-primary/50 bg-primary/10 text-foreground"
            : "text-muted-foreground hover:bg-accent",
        )}
      >
        <ClipboardList className="size-3.5 shrink-0" />
        <span className="truncate">{t.plural("copied", copied.length)}</span>
      </button>
      <div className="ml-auto flex flex-wrap gap-1">
        {QUICK_ACTIONS.map((action) => (
          <button
            key={action}
            type="button"
            disabled={disabled}
            onClick={() => onAction(action)}
            className="rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50"
          >
            {t(`action_${action}`)}
          </button>
        ))}
      </div>
    </div>
  );
}
