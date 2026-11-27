import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { settingsMessages as messages } from "./messages";

/** A collapsible group of options, flagged when some differ from the defaults. */
export function Section({
  title,
  changed,
  onReset,
  children,
}: {
  title: string;
  changed: boolean;
  onReset: () => void;
  children: ReactNode;
}) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  return (
    <Collapsible.Root open={open} onOpenChange={setOpen} className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-2">
        <Collapsible.Trigger className="flex items-center gap-1 text-xs font-medium text-muted-foreground hover:text-foreground">
          <ChevronRight className={cn("size-3.5 transition-transform", open && "rotate-90")} />
          {title}
          {changed && (
            <span className="ml-1 size-1.5 rounded-full bg-primary" title={t("customized")} />
          )}
        </Collapsible.Trigger>
        {changed && open && (
          <button
            type="button"
            onClick={onReset}
            className="text-[11px] text-muted-foreground hover:text-foreground"
          >
            {t("resetDefaults")}
          </button>
        )}
      </div>
      <Collapsible.Content className="flex flex-col gap-4 border-l pl-3">
        {children}
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
