import { X } from "lucide-react";
import { useT } from "@/i18n";
import { type Attachments, DocumentChip } from "../../documents";
import { messages } from "../../messages";

/** What goes with the next message: documents, then images. */
export function AttachmentList({ attachments }: { attachments: Attachments }) {
  const t = useT(messages);
  const { images, documents } = attachments;
  if (images.length === 0 && documents.length === 0) return null;
  return (
    <ul className="flex flex-wrap gap-2 px-1 pt-1">
      {documents.map((item) => (
        <li key={item.key}>
          <DocumentChip
            name={item.name}
            state={item}
            onRemove={() => attachments.removeDocument(item.key)}
          />
        </li>
      ))}
      {images.map(({ attachment }) => (
        <li
          key={attachment.id}
          className="group relative flex h-12 items-center overflow-hidden rounded-lg border bg-background"
        >
          <img
            src={attachment.thumbnail}
            alt={attachment.name}
            className="h-full w-14 object-cover"
          />
          <button
            type="button"
            onClick={() => attachments.removeImage(attachment.id)}
            className="absolute top-0.5 right-0.5 rounded-full bg-background/80 p-0.5 text-muted-foreground hover:text-foreground"
            aria-label={t("removeAttachment", { name: attachment.name })}
          >
            <X className="size-3" />
          </button>
        </li>
      ))}
    </ul>
  );
}
