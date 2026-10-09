import { useMutation } from "@tanstack/react-query";
import { Check, FileDown } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import { commands, unwrap } from "@/lib/api";
import { toMarkdown } from "../lib/export";
import { messages } from "../messages";
import type { Conversation } from "../store";

/** Save the conversation as a Markdown file. */
export function ExportButton({ conversation }: { conversation: Conversation | undefined }) {
  const t = useT(messages);
  const [saved, setSaved] = useState(false);
  const title = conversation?.title || t("newChat");
  const save = useMutation({
    mutationFn: (current: Conversation) => {
      const markdown = toMarkdown(
        current,
        {
          title,
          you: t("exportYou"),
          model: t("model"),
          reasoning: t("reasoning"),
          exported: t("exportedOn"),
          image: t("exportImage"),
          recording: t("recordingName"),
          stopped: t("stopped"),
        },
        new Date(),
      );
      return unwrap(() => commands.saveExport(title, markdown, "markdown"));
    },
    onSuccess: (path) => setSaved(path !== null),
  });

  useEffect(() => {
    if (!saved) return;
    const timer = setTimeout(() => setSaved(false), 2000);
    return () => clearTimeout(timer);
  }, [saved]);

  const empty = !conversation?.messages.some((message) => message.content);
  const label = save.isError ? t("exportFailed", { message: save.error.message }) : t("export");
  return (
    <Tooltip content={saved ? t("exported") : label}>
      <Button
        variant="ghost"
        size="icon"
        className={save.isError ? "text-destructive" : undefined}
        disabled={empty || save.isPending}
        onClick={() => conversation && save.mutate(conversation)}
        aria-label={t("export")}
      >
        {saved ? <Check className="text-success" /> : <FileDown />}
      </Button>
    </Tooltip>
  );
}
