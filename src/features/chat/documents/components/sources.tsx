import { BookOpen } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useT } from "@/i18n";
import type { ContextMode, Source } from "@/lib/api";
import { cn } from "@/lib/utils";
import { groupSources } from "../lib/sources";
import { documentMessages as messages } from "../messages";
import { KindIcon } from "./kind-icon";

/**
 * Where a reply may come from: each document the model read, and for long
 * ones the pages of the passages it was given. A page opens the passage.
 */
export function Sources({
  sources,
  mode,
  passages,
}: {
  sources: Source[];
  mode: ContextMode | undefined;
  passages?: { read: number; total: number };
}) {
  const t = useT(messages);
  const groups = groupSources(sources);
  return (
    <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
      <span className="flex items-center gap-1 pr-0.5">
        <BookOpen className="size-3.5" />
        {t("sources")}
      </span>
      {groups.map(({ name, sources: pages }) => {
        const first = pages[0];
        if (!first) return null;
        const paged = pages.some((source) => source.page !== null);
        return (
          <span
            key={first.documentId}
            className="flex items-center gap-1 rounded-full border bg-card py-0.5 pr-1 pl-0.5"
          >
            {paged ? (
              <span className="flex min-w-0 items-center gap-1 pl-0.5">
                <KindIcon kind={first.kind} className="size-4 rounded-full [&_svg]:size-2.5" />
                <span className="max-w-40 truncate pr-0.5 text-foreground/80">{name}</span>
              </span>
            ) : (
              <Excerpt source={first} label={name} withIcon />
            )}
            {mode === "full" && (
              <span className="pr-1 text-muted-foreground">{t("readWhole")}</span>
            )}
            {paged &&
              mode !== "full" &&
              pages.map((source) => (
                <Excerpt
                  key={source.page}
                  source={source}
                  label={t(source.kind === "slides" ? "sourceSlide" : "sourcePage", {
                    page: String(source.page),
                  })}
                />
              ))}
          </span>
        );
      })}
      {mode === "passages" && passages && (
        <span className="tabular-nums">
          {t("passagesRead", { read: String(passages.read), total: String(passages.total) })}
        </span>
      )}
    </div>
  );
}

function Excerpt({
  source,
  label,
  withIcon = false,
}: {
  source: Source;
  label: string;
  withIcon?: boolean;
}) {
  const t = useT(messages);
  return (
    <Popover>
      <PopoverTrigger
        className={cn(
          "flex min-w-0 items-center gap-1 rounded-full px-1.5 py-0.5 text-foreground/80 transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
          withIcon && "pl-0.5",
        )}
      >
        {withIcon && (
          <KindIcon kind={source.kind} className="size-4 rounded-full [&_svg]:size-2.5" />
        )}
        <span className="max-w-40 truncate">{label}</span>
      </PopoverTrigger>
      <PopoverContent align="start" className="flex w-96 flex-col gap-2">
        <div className="flex items-center gap-2">
          <KindIcon kind={source.kind} className="size-6" />
          <span className="min-w-0 flex-1 truncate text-xs font-medium">{source.name}</span>
          {source.page !== null && (
            <span className="shrink-0 text-[11px] text-muted-foreground">
              {t(source.kind === "slides" ? "sourceSlide" : "sourcePage", {
                page: String(source.page),
              })}
            </span>
          )}
        </div>
        <blockquote className="max-h-60 overflow-y-auto border-l-2 border-primary/40 pl-3 text-xs leading-relaxed whitespace-pre-wrap text-muted-foreground">
          {source.excerpt}
        </blockquote>
      </PopoverContent>
    </Popover>
  );
}
