import { ImageOff, Mic } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";
import { formatDuration } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import type { Attachment } from "../../lib/attachments";
import { messages } from "../../messages";
import { useMediaPayloads } from "../../store";

/** An image or recording sent with a message. Data isn't kept across restarts. */
export function AttachmentPreview({ attachment }: { attachment: Attachment }) {
  const t = useT(messages);
  const available = useMediaPayloads((payloads) => attachment.id in payloads);
  const preview =
    attachment.kind === "image" ? (
      attachment.thumbnail ? (
        <img
          src={attachment.thumbnail}
          alt={attachment.name}
          className={cn("h-24 max-w-48 rounded-md border object-cover", !available && "opacity-60")}
        />
      ) : (
        <span className="flex h-12 items-center gap-1.5 rounded-md border px-3 text-xs text-muted-foreground">
          <ImageOff className="size-3.5" />
          {attachment.name}
        </span>
      )
    ) : (
      <span className="flex h-9 items-center gap-1.5 rounded-md border bg-secondary px-3 text-xs">
        <Mic className="size-3.5 text-primary" />
        {formatDuration(attachment.durationSeconds ?? 0)}
      </span>
    );
  return available ? preview : <Tooltip content={t("attachmentGone")}>{preview}</Tooltip>;
}
