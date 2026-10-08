import { useQuery } from "@tanstack/react-query";
import { Download, Link2 } from "lucide-react";
import { type FormEvent, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useDebounced } from "@/hooks/use-debounced";
import { useT } from "@/i18n";
import type { ModelReference } from "@/lib/api";
import { referenceQuery } from "../api";
import { messages } from "../messages";

/**
 * Paste any model name or link; Calcine shows what it understood, then
 * `onDownload` lets the user pick a precision.
 */
export function AddModelCard({ onDownload }: { onDownload: (reference: ModelReference) => void }) {
  const t = useT(messages);
  const [input, setInput] = useState("");
  const query = useDebounced(input.trim(), 250);
  const preview = useQuery(referenceQuery(query));

  const reference = query === input.trim() ? preview.data : undefined;
  const error = query && preview.isError ? preview.error.message : null;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!reference) return;
    onDownload(reference);
    setInput("");
  };

  return (
    <Card className="p-4">
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label htmlFor="model-reference" className="text-sm font-medium">
          {t("addTitle")}
        </label>
        <div className="flex gap-2">
          <div className="flex h-9 flex-1 items-center gap-2 rounded-md border bg-background px-3 focus-within:ring-2 focus-within:ring-ring">
            <Link2 className="size-4 shrink-0 text-muted-foreground" />
            <input
              id="model-reference"
              value={input}
              onChange={(event) => setInput(event.target.value)}
              placeholder={t("addPlaceholder")}
              autoComplete="off"
              spellCheck={false}
              className="h-full w-full bg-transparent font-mono text-[13px] outline-none placeholder:font-sans placeholder:text-muted-foreground"
            />
          </div>
          <Button type="submit" variant="default" size="lg" className="h-9" disabled={!reference}>
            <Download />
            {t("addContinue")}
          </Button>
        </div>

        <div className="min-h-5 text-xs text-muted-foreground">
          {error ? (
            <span className="text-destructive">{error}</span>
          ) : reference ? (
            <span className="flex flex-wrap items-center gap-2">
              <span className="font-mono text-foreground">{reference.name}</span>
              <Badge tone="info">{t(`hub_${reference.hub}`)}</Badge>
              <Badge tone="outline" className="font-mono">
                {reference.precision ?? t("recommendedPrecision")}
              </Badge>
            </span>
          ) : (
            <span>
              {t.rich("addHint", {
                code: (text) => <code className="font-mono">{text}</code>,
              })}
            </span>
          )}
        </div>
      </form>
    </Card>
  );
}
