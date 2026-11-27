import * as Collapsible from "@radix-ui/react-collapsible";
import { useQuery } from "@tanstack/react-query";
import { ChevronRight } from "lucide-react";
import { useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { gatewayQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import type { LocalModel } from "@/lib/api";
import { cn } from "@/lib/utils";
import { buildChatRequest, type ChatSettings, toCurl } from "../../lib/request";
import { messages } from "../../messages";

/** The exact API call Calcine makes, to reuse from another app. */
export function RequestPreview({
  model,
  modelId,
  settings,
}: {
  model: LocalModel | undefined;
  modelId: string;
  settings: ChatSettings;
}) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  const { data: status } = useQuery(gatewayQuery);
  const url = `${status?.baseUrl ?? "http://127.0.0.1:18181/v1"}/chat/completions`;
  const curl = toCurl(
    url,
    buildChatRequest(model, modelId, settings, [{ role: "user", content: "Hello!" }]),
  );
  return (
    <Collapsible.Root open={open} onOpenChange={setOpen} className="border-t pt-4">
      <Collapsible.Trigger className="flex items-center gap-1 text-xs font-medium text-muted-foreground hover:text-foreground">
        <ChevronRight className={cn("size-3.5 transition-transform", open && "rotate-90")} />
        {t("equivalentRequest")}
      </Collapsible.Trigger>
      <Collapsible.Content className="relative mt-2">
        <pre className="overflow-x-auto rounded-md border bg-background p-2.5 pr-9 font-mono text-[10.5px] leading-relaxed">
          {curl}
        </pre>
        <CopyButton value={curl} className="absolute top-1 right-1" />
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
