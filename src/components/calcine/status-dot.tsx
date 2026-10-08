import { cn } from "@/lib/utils";

const TONES = {
  idle: "bg-muted-foreground/50",
  ok: "bg-success",
  busy: "bg-warning animate-pulse",
  error: "bg-destructive",
} as const;

export function StatusDot({ tone, className }: { tone: keyof typeof TONES; className?: string }) {
  return (
    <span
      className={cn("inline-block size-2 shrink-0 rounded-full", TONES[tone], className)}
      aria-hidden="true"
    />
  );
}
