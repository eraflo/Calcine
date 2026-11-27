import { DocumentChip } from "../../documents";
import type { ChatMessage } from "../../store";
import { AttachmentPreview } from "./attachment-preview";

/** What the user sent: documents, images and recordings, then the text. */
export function UserMessage({ message }: { message: ChatMessage }) {
  const { documents, attachments, content } = message;
  return (
    <div className="flex flex-col items-end gap-1.5">
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
      {content && (
        <div className="max-w-[80%] rounded-lg bg-secondary px-3.5 py-2 text-sm whitespace-pre-wrap">
          {content}
        </div>
      )}
    </div>
  );
}
