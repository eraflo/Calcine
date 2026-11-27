import { Sparkles } from "lucide-react";
import { useT } from "@/i18n";
import { documentMessages } from "../../documents";

/** Quick requests offered once documents are attached. */
const ACTIONS = ["docSummarize", "docKeyPoints", "docFigures"] as const;

/** One click sends a common request about the attached documents. */
export function DocumentActions({
  count,
  disabled,
  onSend,
}: {
  /** Documents attached: the wording follows. */
  count: number;
  disabled: boolean;
  onSend: (message: string) => void;
}) {
  const t = useT(documentMessages);
  return (
    <div className="mb-2 flex flex-wrap items-center gap-1.5 px-1 animate-in fade-in-0 slide-in-from-bottom-1">
      <Sparkles className="size-3.5 text-primary" />
      {ACTIONS.map((key) => {
        const label = t.plural(key, count);
        return (
          <button
            key={key}
            type="button"
            disabled={disabled}
            onClick={() => onSend(label)}
            className="rounded-full border bg-card px-3 py-1 text-xs text-muted-foreground transition-colors hover:border-primary/40 hover:text-foreground disabled:opacity-50"
          >
            {label}
          </button>
        );
      })}
    </div>
  );
}
