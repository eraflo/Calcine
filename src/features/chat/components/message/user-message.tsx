import { Pencil } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import { DocumentChip } from "../../documents";
import { messages } from "../../messages";
import type { ChatMessage } from "../../store";
import { AttachmentPreview } from "./attachment-preview";
import type { MessageActions } from "./message";
import { MessageEditor } from "./message-editor";

/** What the user sent: documents, images and recordings, then the text, which can be edited. */
export function UserMessage({
  message,
  actions,
}: {
  message: ChatMessage;
  actions?: MessageActions;
}) {
  const t = useT(messages);
  const [editing, setEditing] = useState(false);
  const { documents, attachments, content } = message;
  const onEdit = actions?.onEdit;
  return (
    <div className="group flex flex-col items-end gap-1.5">
      {documents && documents.length > 0 && (
        <div className="flex max-w-[80%] flex-wrap justify-end gap-1.5">
          {documents.map((document) => (
            <DocumentChip
              key={document.id}
              name={document.name}
              state={{ status: "ready", document }}
              className="pr-2.5"
            />
          ))}
        </div>
      )}
      {attachments && attachments.length > 0 && (
        <div className="flex max-w-[80%] flex-wrap justify-end gap-1.5">
          {attachments.map((attachment) => (
            <AttachmentPreview key={attachment.id} attachment={attachment} />
          ))}
        </div>
      )}
      {editing && onEdit ? (
        <MessageEditor
          initial={content}
          replacesFollowing={actions?.hasFollowing ?? false}
          onCancel={() => setEditing(false)}
          onSend={(text) => {
            setEditing(false);
            onEdit(text);
          }}
        />
      ) : (
        content && (
          <div className="flex max-w-[80%] items-start gap-1.5">
            {onEdit && (
              <Tooltip content={t("editMessage")}>
                <Button
                  variant="ghost"
                  size="icon"
                  className="mt-0.5 size-7 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
                  onClick={() => setEditing(true)}
                  aria-label={t("editMessage")}
                >
                  <Pencil className="size-3.5" />
                </Button>
              </Tooltip>
            )}
            <div className="min-w-0 rounded-lg bg-secondary px-3.5 py-2 text-sm whitespace-pre-wrap">
              {content}
            </div>
          </div>
        )
      )}
    </div>
  );
}
