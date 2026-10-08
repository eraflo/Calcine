import type * as React from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog";

/** A centered yes/no dialog for destructive or important actions. */
export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  children,
  confirmLabel,
  destructive = false,
  busy = false,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  children: React.ReactNode;
  confirmLabel: string;
  destructive?: boolean;
  busy?: boolean;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="top-1/2 max-w-md -translate-y-1/2 p-5">
        <DialogTitle className="text-base font-semibold">{title}</DialogTitle>
        <DialogDescription asChild>
          <div className="mt-2 text-sm text-muted-foreground">{children}</div>
        </DialogDescription>
        <div className="mt-5 flex justify-end gap-2">
          <DialogClose asChild>
            <Button variant="ghost">Cancel</Button>
          </DialogClose>
          <Button
            variant={destructive ? "destructive" : "default"}
            onClick={onConfirm}
            disabled={busy}
          >
            {confirmLabel}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
