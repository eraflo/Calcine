import { FileSearch } from "lucide-react";
import { useT } from "@/i18n";
import { documentMessages } from "../../documents";
import { messages } from "../../messages";

/** What happens before the reply's first word. */
export function LiveStatus({ status }: { status: "summarizing" | "reading" | null | undefined }) {
  const t = useT(messages);
  const td = useT(documentMessages);
  if (status === "summarizing") {
    return (
      <p className="animate-pulse text-xs text-muted-foreground" role="status">
        {t("summarizing")}
      </p>
    );
  }
  if (status === "reading") {
    return (
      <p
        className="flex animate-pulse items-center gap-1.5 text-xs text-muted-foreground"
        role="status"
      >
        <FileSearch className="size-3.5 text-primary" />
        {td("readingDocuments")}
      </p>
    );
  }
  return <TypingDots />;
}

export function TypingDots() {
  const t = useT(messages);
  return (
    <span className="flex gap-1 py-2" role="status" aria-label={t("generating")}>
      {[0, 150, 300].map((delay) => (
        <span
          key={delay}
          className="size-1.5 animate-pulse rounded-full bg-muted-foreground"
          style={{ animationDelay: `${delay}ms` }}
        />
      ))}
    </span>
  );
}
