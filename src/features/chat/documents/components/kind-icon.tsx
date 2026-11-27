import { Braces, FileCode2, FileText, Globe, Presentation, ScrollText } from "lucide-react";
import type { ComponentType } from "react";
import type { DocumentKind } from "@/lib/api";
import { cn } from "@/lib/utils";

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
