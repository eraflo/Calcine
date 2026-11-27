import {
  Braces,
  FileCode2,
  FileText,
  FileWarning,
  Globe,
  Loader2,
  Presentation,
  ScrollText,
  X,
} from "lucide-react";
import type { ComponentType } from "react";
import { useT } from "@/i18n";
import type { DocumentKind, DocumentProblem } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { DocumentInfo } from "../lib/documents";
import { messages } from "../messages";

/** Icon and tint of each kind of document. */
const KINDS: Record<DocumentKind, { icon: ComponentType<{ className?: string }>; tint: string }> = {
  pdf: { icon: FileText, tint: "bg-red-500/12 text-red-600 dark:text-red-400" },
  word: { icon: ScrollText, tint: "bg-blue-500/12 text-blue-600 dark:text-blue-400" },
  slides: { icon: Presentation, tint: "bg-orange-500/12 text-orange-600 dark:text-orange-400" },
  web: { icon: Globe, tint: "bg-sky-500/12 text-sky-600 dark:text-sky-400" },
  data: { icon: Braces, tint: "bg-emerald-500/12 text-emerald-600 dark:text-emerald-400" },
  code: { icon: FileCode2, tint: "bg-violet-500/12 text-violet-600 dark:text-violet-400" },
  text: { icon: FileText, tint: "bg-muted text-muted-foreground" },
};

export function KindIcon({ kind, className }: { kind: DocumentKind; className?: string }) {
  const { icon: Icon, tint } = KINDS[kind];
  return (
    <span
      className={cn("flex size-8 shrink-0 items-center justify-center rounded-md", tint, className)}
    >
      <Icon className="size-4" />
    </span>
  );
}

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
