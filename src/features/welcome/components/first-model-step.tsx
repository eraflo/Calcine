import { useQuery } from "@tanstack/react-query";
import { Check, Download } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { catalogQuery } from "@/features/discover/api";
import { useAvailability } from "@/features/library/availability";
import { usePullByName } from "@/features/tasks/api";
import { percent } from "@/features/tasks/lib/format";
import { Step } from "./step";

/** Small models that download quickly and run on the NPU: good first picks. */
const STARTER_MODELS = [
  { name: "qualcomm/Qwen3-0.6B", blurb: "Tiny and fast. About 725 MiB." },
  { name: "qualcomm/Llama-v3.2-1B-Instruct", blurb: "Meta's compact assistant." },
  { name: "qualcomm/Qwen3-4B", blurb: "Smarter answers. About 3 GiB." },
];

export function FirstModelStep() {
  const catalog = useQuery(catalogQuery(false));
  const availability = useAvailability();
  const pull = usePullByName();

  const available = new Set(catalog.data?.models.map((model) => model.name));
  const starters = catalog.data
    ? STARTER_MODELS.filter((model) => available.has(model.name))
    : STARTER_MODELS;
  const hasModel = starters.some((model) => availability(model.name).status === "installed");

  return (
    <Step index={3} title="Download a first model" status={hasModel ? "done" : "todo"}>
      <p>These are compiled for your NPU. You can add any other model later from Discover.</p>
      <ul className="mt-3 flex flex-col gap-2">
        {starters.map((model) => {
          const state = availability(model.name);
          return (
            <li
              key={model.name}
              className="flex flex-col gap-2 rounded-md border bg-background/40 p-3"
            >
              <div className="flex items-center gap-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate font-mono text-[13px] text-foreground">{model.name}</p>
                  <p className="text-xs">{model.blurb}</p>
                </div>
                {state.status === "installed" ? (
                  <Badge tone="success">
                    <Check />
                    Ready
                  </Badge>
                ) : state.status === "downloading" ? (
                  <span className="text-xs tabular-nums">{percent(state.job) ?? 0}%</span>
                ) : (
                  <Button size="sm" onClick={() => pull.pull(model.name)} disabled={pull.isPending}>
                    <Download />
                    Download
                  </Button>
                )}
              </div>
              {state.status === "downloading" && (
                <Progress value={percent(state.job)} label={`Downloading ${model.name}`} />
              )}
            </li>
          );
        })}
      </ul>
    </Step>
  );
}
