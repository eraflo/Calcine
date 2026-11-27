import { Paperclip } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import { documentMessages } from "../../documents";

/** Attach documents, and images for vision models. */
export function AttachButton({
  media,
  disabled,
  onPick,
}: {
  media: boolean;
  disabled: boolean;
  onPick: () => void;
}) {
  const t = useT(documentMessages);
  const label = media ? t("attachDocumentsOrImages") : t("attachDocuments");
  return (
    <Tooltip content={label}>
      <span>
        <Button
          type="button"
          size="icon"
          variant="ghost"
          className="size-9"
          onClick={onPick}
          disabled={disabled}
          aria-label={label}
        >
          <Paperclip />
        </Button>
      </span>
    </Tooltip>
  );
}
