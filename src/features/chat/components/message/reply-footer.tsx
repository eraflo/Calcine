import { ChevronLeft, ChevronRight, RotateCcw } from "lucide-react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import { messages } from "../../messages";
import type { ChatMessage } from "../../store";
import type { MessageActions } from "./message";
import { Stats } from "./stats";

/**
 * Under a reply: its versions, how it was written (speed, tokens), and
 * actions shown on hover: copy, regenerate (the last reply only).
 */
export function ReplyFooter({
  message,
  actions,
}: {
  message: ChatMessage;
  actions?: MessageActions;
}) {
  const t = useT(messages);
  const versions = message.versions?.length ?? 0;
  const shown = message.version ?? 0;
  if (!message.stats && !message.stopped && !message.content && versions < 2) return null;

  return (
    <div className="flex min-h-7 items-center gap-2 text-[11px] text-muted-foreground">
      {versions > 1 && (
        <span className="flex items-center gap-0.5 tabular-nums">
          <StepButton
            label={t("previousVersion")}
            disabled={!actions?.onSelectVersion || shown === 0}
            onClick={() => actions?.onSelectVersion?.(shown - 1)}
          >
            <ChevronLeft />
          </StepButton>
          {t("versionOf", { index: String(shown + 1), count: String(versions) })}
          <StepButton
            label={t("nextVersion")}
            disabled={!actions?.onSelectVersion || shown === versions - 1}
            onClick={() => actions?.onSelectVersion?.(shown + 1)}
          >
            <ChevronRight />
          </StepButton>
        </span>
      )}
      {message.stopped && <span>{t("stopped")}</span>}
      {message.stats && <Stats stats={message.stats} />}
      <span className="flex items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
        {message.content && <CopyButton value={message.content} />}
        {actions?.onRegenerate && (
          <Tooltip content={t("regenerate")}>
            <Button
              variant="ghost"
              size="icon"
              className="size-7 text-muted-foreground"
              onClick={actions.onRegenerate}
              aria-label={t("regenerate")}
            >
              <RotateCcw className="size-3.5" />
            </Button>
          </Tooltip>
        )}
      </span>
    </div>
  );
}

function StepButton({
  label,
  disabled,
  onClick,
  children,
}: {
  label: string;
  disabled: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <Button
      variant="ghost"
      size="icon"
      className="size-6 text-muted-foreground [&_svg]:size-3.5"
      disabled={disabled}
      onClick={onClick}
      aria-label={label}
    >
      {children}
    </Button>
  );
}
