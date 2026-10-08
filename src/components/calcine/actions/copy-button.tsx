import { Check, Copy } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { cn } from "@/lib/utils";

/** Copies `value`; shows a check for a moment. Icon-only when no `label`. */
export function CopyButton({
  value,
  label,
  className,
}: {
  value: string;
  label?: string;
  className?: string;
}) {
  const tc = useT(common);
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <Button
      size={label ? "sm" : "icon"}
      variant="ghost"
      onClick={copy}
      className={cn(!label && "size-7", className)}
      aria-label={label ? undefined : tc("copy")}
    >
      {copied ? <Check className="text-success" /> : <Copy />}
      {label && (copied ? tc("copied") : label)}
    </Button>
  );
}
