import * as Collapsible from "@radix-ui/react-collapsible";
import { Brain, ChevronRight } from "lucide-react";
import { useState } from "react";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { messages } from "../../messages";

/** The model's chain of thought, folded once the reply starts. */
export function Reasoning({ text, active }: { text: string; active: boolean }) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  return (
    <Collapsible.Root open={open || active} onOpenChange={setOpen}>
      <Collapsible.Trigger className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground">
        <Brain className={cn("size-3.5", active && "animate-pulse text-primary")} />
        {active ? t("thinking") : t("reasoning")}
        <ChevronRight
          className={cn("size-3.5 transition-transform", (open || active) && "rotate-90")}
        />
      </Collapsible.Trigger>
      <Collapsible.Content>
        <p className="mt-1.5 border-l-2 pl-3 text-xs leading-relaxed whitespace-pre-wrap text-muted-foreground">
          {text.trim()}
        </p>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
