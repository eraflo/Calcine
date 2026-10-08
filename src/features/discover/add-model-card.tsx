import { useQuery } from "@tanstack/react-query";
import { Download, Link2 } from "lucide-react";
import { type FormEvent, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useStartPull } from "@/features/tasks/api";
import { useDebounced } from "@/hooks/use-debounced";
import { HUB_LABELS, referenceQuery } from "./api";

/** Paste any model name or link; Calcine shows what it understood before downloading. */
export function AddModelCard() {
  const [input, setInput] = useState("");
  const query = useDebounced(input.trim(), 250);
  const preview = useQuery(referenceQuery(query));
  const startPull = useStartPull();

  const reference = query === input.trim() ? preview.data : undefined;
  const error = query && preview.isError ? preview.error.message : null;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!reference) return;
    startPull.mutate({ reference, modelType: null }, { onSuccess: () => setInput("") });
  };

  return (
    <Card className="p-4">
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label htmlFor="model-reference" className="text-sm font-medium">
          Add a model by name or link
        </label>
        <div className="flex gap-2">
          <div className="flex h-9 flex-1 items-center gap-2 rounded-md border bg-background px-3 focus-within:ring-2 focus-within:ring-ring">
            <Link2 className="size-4 shrink-0 text-muted-foreground" />
            <input
              id="model-reference"
              value={input}
              onChange={(event) => setInput(event.target.value)}
              placeholder="Paste a link, or type owner/model"
              autoComplete="off"
              spellCheck={false}
              className="h-full w-full bg-transparent font-mono text-[13px] outline-none placeholder:font-sans placeholder:text-muted-foreground"
            />
          </div>
          <Button
            type="submit"
            variant="default"
            size="lg"
            className="h-9"
            disabled={!reference || startPull.isPending}
          >
            <Download />
            Download
          </Button>
        </div>

        <div className="min-h-5 text-xs text-muted-foreground">
          {error ? (
            <span className="text-destructive">{error}</span>
          ) : reference ? (
            <span className="flex flex-wrap items-center gap-2">
              <span className="font-mono text-foreground">{reference.name}</span>
              <Badge tone="info">{HUB_LABELS[reference.hub]}</Badge>
              <Badge tone="outline" className="font-mono">
                {reference.precision ?? "recommended precision"}
              </Badge>
            </span>
          ) : (
            <span>
              Hugging Face, ModelScope and Docker Hub links work, as do names like{" "}
              <code className="font-mono">qualcomm/Qwen3-4B</code>. Add{" "}
              <code className="font-mono">:Q4_0</code> to pick a GGUF precision (best on the NPU).
            </span>
          )}
        </div>
        {startPull.isError && <p className="text-xs text-destructive">{startPull.error.message}</p>}
      </form>
    </Card>
  );
}
