import { Check, CircleAlert, Loader } from "lucide-react";
import type * as React from "react";
import { Card } from "@/components/ui/card";

export type StepStatus = "todo" | "loading" | "done" | "problem";

/** One numbered item of the welcome checklist. */
export function Step({
  index,
  title,
  status,
  children,
}: {
  index: number;
  title: string;
  status: StepStatus;
  children: React.ReactNode;
}) {
  return (
    <li>
      <Card className="flex gap-4 p-4">
        <div className="flex size-7 shrink-0 items-center justify-center rounded-full border bg-background text-xs font-medium">
          {status === "done" ? (
            <Check className="size-4 text-success" />
          ) : status === "problem" ? (
            <CircleAlert className="size-4 text-warning" />
          ) : status === "loading" ? (
            <Loader className="size-4 animate-spin text-muted-foreground" />
          ) : (
            index
          )}
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <h2 className="text-sm font-medium">{title}</h2>
          <div className="text-sm text-muted-foreground">{children}</div>
        </div>
      </Card>
    </li>
  );
}
