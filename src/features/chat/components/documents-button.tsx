import { FileStack, Lock, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import { messages } from "../messages";
import { type Conversation, useChat } from "../store";
import { DocumentChip } from "./document-chip";

/**
 * The conversation's documents: how the model reads them, and a way to
 * remove one or add more. Hidden until a document is attached.
 */
export function DocumentsButton({
  conversation,
  onAdd,
}: {
  conversation: Conversation | undefined;
  /** Pick more documents (they're attached to the next message). */
  onAdd: () => void;
}) {
  const t = useT(messages);
  const removeDocument = useChat((state) => state.removeDocument);
  const documents = conversation?.documents ?? [];
  if (!conversation || documents.length === 0) return null;

  // How the last reply read them.
  const last = conversation.messages.findLast((message) => message.documentMode);
  const mode = last?.documentMode;
  const read = last?.passagesRead;

  return (
    <Popover>
      <Tooltip content={t.plural("documentsCount", documents.length)}>
        <PopoverTrigger asChild>
          <Button
            variant="ghost"
            size="sm"
            className="h-8 gap-1.5 px-2 text-xs"
            aria-label={t.plural("documentsCount", documents.length)}
          >
            <FileStack className="size-4" />
            <span className="tabular-nums">{documents.length}</span>
          </Button>
        </PopoverTrigger>
      </Tooltip>
      <PopoverContent align="end" className="flex w-96 flex-col gap-3 p-4">
        <div className="flex flex-col gap-1">
          <h3 className="text-sm font-semibold">{t("documentsTitle")}</h3>
          <p className="text-xs leading-relaxed text-muted-foreground">
            {mode === "full"
              ? t("documentsModeFull")
              : mode === "passages"
                ? t("documentsModePassages", {
                    read: String(read?.read ?? 0),
                    total: String(read?.total ?? 0),
                  })
                : t("documentsModeNext")}
          </p>
        </div>
        <ul className="flex max-h-72 flex-col gap-1.5 overflow-y-auto">
          {documents.map((document) => (
            <li key={document.id}>
              <DocumentChip
                name={document.name}
                state={{ status: "ready", document }}
                className="max-w-none"
                onRemove={() => removeDocument(conversation.id, document.id)}
              />
            </li>
          ))}
        </ul>
        <div className="flex items-center justify-between gap-3 border-t pt-3">
          <p className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
            <Lock className="size-3 shrink-0" />
            {t("documentsPrivacy")}
          </p>
          <Button
            variant="secondary"
            size="sm"
            className="h-7 shrink-0 gap-1 text-xs"
            onClick={onAdd}
          >
            <Plus className="size-3.5" />
            {t("addDocuments")}
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}
