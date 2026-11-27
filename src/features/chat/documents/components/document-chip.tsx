import { FileWarning, Loader2, X } from "lucide-react";
import { useT } from "@/i18n";
import type { DocumentInfo, DocumentProblem } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";
import { documentMessages as messages } from "../messages";
import { KindIcon } from "./kind-icon";

/** "12 pages · 1.2 MB", "3 slides", "48 KB". */
export function useDocumentMeta() {
  const t = useT(messages);
  return (document: DocumentInfo) => {
    const parts = [
      document.pages !== null &&
        t.plural(document.kind === "slides" ? "slides" : "pages", document.pages),
      formatBytes(document.bytes),
    ].filter(Boolean);
    return parts.join(" · ");
  };
}

type ChipState =
  | { status: "reading" }
  | { status: "ready"; document: DocumentInfo }
  | { status: "failed"; problem: DocumentProblem };

/** A document in the message box, a message or the documents panel. */
export function DocumentChip({
  name,
  state,
  onRemove,
  className,
}: {
  name: string;
  state: ChipState;
  onRemove?: () => void;
  className?: string;
}) {
  const t = useT(messages);
  const meta = useDocumentMeta();
  const failed = state.status === "failed";
  return (
    <div
      className={cn(
        "group/chip relative flex h-12 max-w-64 min-w-40 items-center gap-2.5 rounded-lg border bg-background py-1.5 pr-7 pl-2",
        failed && "border-destructive/40",
        className,
      )}
      title={name}
    >
      {state.status === "ready" ? (
        <KindIcon kind={state.document.kind} />
      ) : (
        <span
          className={cn(
            "flex size-8 shrink-0 items-center justify-center rounded-md",
            failed ? "bg-destructive/12 text-destructive" : "bg-muted text-muted-foreground",
          )}
        >
          {failed ? (
            <FileWarning className="size-4" />
          ) : (
            <Loader2 className="size-4 animate-spin" />
          )}
        </span>
      )}
      <span className="flex min-w-0 flex-col">
        <span className="truncate text-xs font-medium">{name}</span>
        <span
          className={cn(
            "truncate text-[11px]",
            failed ? "text-destructive" : "text-muted-foreground",
          )}
        >
          {state.status === "ready"
            ? meta(state.document)
            : state.status === "reading"
              ? t("readingDocument")
              : t(`problem_${state.problem}`)}
        </span>
      </span>
      {onRemove && (
        <button
          type="button"
          onClick={onRemove}
          className="absolute top-1 right-1 rounded-full p-0.5 text-muted-foreground opacity-60 transition-opacity group-hover/chip:opacity-100 hover:bg-muted hover:text-foreground focus-visible:opacity-100"
          aria-label={t("removeDocument", { name })}
        >
          <X className="size-3" />
        </button>
      )}
    </div>
  );
}
