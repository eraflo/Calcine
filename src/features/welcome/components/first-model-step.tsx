import { useQuery } from "@tanstack/react-query";
import { Check, Download } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { catalogQuery } from "@/features/discover/api";
import { useAvailability } from "@/features/library/availability";
import { usePullByName } from "@/features/tasks/api";
import { percent } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { messages } from "../messages";
import { Step } from "./step";

/** Small models that download quickly and run on the NPU: good first picks. */
const STARTER_MODELS = [
  { name: "qualcomm/Qwen3-0.6B", blurb: "starterQwenTiny" },
  { name: "qualcomm/Llama-v3.2-1B-Instruct", blurb: "starterLlama" },
  { name: "qualcomm/Qwen3-4B", blurb: "starterQwenSmart" },
] as const;

export function FirstModelStep() {
  const t = useT(messages);
  const tc = useT(common);
  const catalog = useQuery(catalogQuery(false));
  const availability = useAvailability();
  const pull = usePullByName();

  const available = new Set(catalog.data?.models.map((model) => model.name));
  const starters = catalog.data
    ? STARTER_MODELS.filter((model) => available.has(model.name))
    : STARTER_MODELS;
  const hasModel = starters.some((model) => availability(model.name).status === "installed");

  return (
    <Step index={3} title={t("firstModelTitle")} status={hasModel ? "done" : "todo"}>
      <p>{t("firstModelHint")}</p>
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
                  <p className="text-xs">{t(model.blurb)}</p>
                </div>
                {state.status === "installed" ? (
                  <Badge tone="success">
                    <Check />
                    {t("ready")}
                  </Badge>
                ) : state.status === "downloading" ? (
                  <span className="text-xs tabular-nums">{percent(state.job) ?? 0}%</span>
                ) : (
                  <Button size="sm" onClick={() => pull.pull(model.name)} disabled={pull.isPending}>
                    <Download />
                    {tc("download")}
                  </Button>
                )}
              </div>
              {state.status === "downloading" && (
                <Progress
                  value={percent(state.job)}
                  label={t("downloadingModel", { name: model.name })}
                />
              )}
            </li>
          );
        })}
      </ul>
    </Step>
  );
}
