import type * as React from "react";
import { cn } from "@/lib/utils";

export function Kbd({ className, ...props }: React.ComponentProps<"kbd">) {
  return (
    <kbd
      className={cn(
        "inline-flex h-5 items-center rounded-sm border bg-background px-1.5 font-mono text-[10px] text-muted-foreground",
        className,
      )}
      {...props}
    />
  );
}
