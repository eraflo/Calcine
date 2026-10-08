import { useQuery } from "@tanstack/react-query";
import { Gauge, Play, Square } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, Select } from "@/components/ui/field";
import { Progress } from "@/components/ui/progress";
import { POWER_MODES, type PowerMode } from "@/features/chat/lib/request";
import { messages as chatMessages } from "@/features/chat/messages";
import { modelsQuery } from "@/features/library/api";
import { useCancelJob } from "@/features/tasks/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { ComputeUnit } from "@/lib/api";
import { cn } from "@/lib/utils";
import { benchToolQuery, useRunningJob, useStartBenchmark } from "../api";
import { benchTargets, benchUnits } from "../lib/results";
import { messages } from "../messages";
import { UNIT_COLOR } from "./metric-bars";

const PROMPTS = [128, 512, 1024, 2048] as const;
const GENERATED = [64, 128, 256, 512] as const;
const REPETITIONS = [3, 5, 10] as const;

/** Pick a model, units and sizes, then run geniex-bench on each unit. */
export function RunCard() {
  const t = useT(messages);
  const tc = useT(common);
  const tchat = useT(chatMessages);
  const { data: models = [] } = useQuery(modelsQuery);
  const { data: tool } = useQuery(benchToolQuery);
  const targets = benchTargets(models);
  const [targetId, setTargetId] = useState("");
  const target = targets.find((candidate) => candidate.id === targetId) ?? targets[0];
  const available = target ? benchUnits(target.model.runtime) : [];
  const [units, setUnits] = useState<ComputeUnit[]>(["npu"]);
  const [promptTokens, setPromptTokens] = useState(512);
  const [generatedTokens, setGeneratedTokens] = useState(128);
  const [repetitions, setRepetitions] = useState(5);
  const [powerMode, setPowerMode] = useState<PowerMode>("burst");
  const start = useStartBenchmark();
  const cancel = useCancelJob();
  const running = useRunningJob("benchmark");
  const ready = tool?.installed !== null && tool?.installed === tool?.wanted;

  // Changing model keeps the units it supports, or falls back to all of them.
  useEffect(() => {
    if (!target) return;
    const supported = benchUnits(target.model.runtime);
    setUnits((current) => {
      const kept = current.filter((unit) => supported.includes(unit));
      return kept.length ? kept : supported;
    });
  }, [target]);

  const toggle = (unit: ComputeUnit) =>
    setUnits((current) =>
      current.includes(unit) ? current.filter((u) => u !== unit) : [...current, unit],
    );
  const chosen = available.filter((unit) => units.includes(unit));

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Gauge className="size-4 text-primary" />
          {t("runTitle")}
        </CardTitle>
        <CardDescription>{t("runHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {targets.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t("noModels")}</p>
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-[2fr_3fr]">
              <Field label={t("model")} htmlFor="bench-model">
                <Select
                  id="bench-model"
                  value={target?.id ?? ""}
                  onChange={(event) => setTargetId(event.target.value)}
                  className="font-mono"
                  disabled={Boolean(running)}
                >
                  {targets.map(({ id }) => (
                    <option key={id} value={id}>
                      {id}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field
                label={t("units")}
                hint={target?.model.runtime === "qairt" ? t("unitsQairt") : undefined}
              >
                <div className="flex flex-wrap gap-1.5">
                  {(["npu", "hybrid", "gpu", "cpu"] as const).map((unit) => {
                    const supported = available.includes(unit);
                    const active = supported && units.includes(unit);
                    return (
                      <label
                        key={unit}
                        className={cn(
                          "flex h-8 cursor-pointer items-center rounded-md border px-3 text-xs font-medium transition-colors has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-40 has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
                          active
                            ? cn("border-current bg-accent", UNIT_COLOR[unit].text)
                            : "text-muted-foreground hover:text-foreground",
                        )}
                      >
                        <input
                          type="checkbox"
                          className="sr-only"
                          checked={active}
                          disabled={!supported || Boolean(running)}
                          onChange={() => toggle(unit)}
                        />
                        {tc(unit)}
                      </label>
                    );
                  })}
                </div>
              </Field>
            </div>

            <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
              <Choice
                id="bench-prompt"
                label={t("promptTokens")}
                value={promptTokens}
                options={PROMPTS}
                format={(count) => t("tokens", { count: String(count) })}
                onChange={setPromptTokens}
                disabled={Boolean(running)}
              />
              <Choice
                id="bench-generated"
                label={t("generatedTokens")}
                value={generatedTokens}
                options={GENERATED}
                format={(count) => t("tokens", { count: String(count) })}
                onChange={setGeneratedTokens}
                disabled={Boolean(running)}
              />
              <Choice
                id="bench-repetitions"
                label={t("repetitions")}
                value={repetitions}
                options={REPETITIONS}
                format={(count) => t("runs", { count: String(count) })}
                onChange={setRepetitions}
                disabled={Boolean(running)}
              />
              <Field label={t("powerMode")} htmlFor="bench-power">
                <Select
                  id="bench-power"
                  value={powerMode}
                  disabled={Boolean(running)}
                  onChange={(event) => setPowerMode(event.target.value as PowerMode)}
                >
                  {POWER_MODES.map((mode) => (
                    <option key={mode} value={mode}>
                      {tchat(`power_${mode}`)}
                    </option>
                  ))}
                </Select>
              </Field>
            </div>

            {running ? (
              <div className="flex items-center gap-3">
                <div className="flex flex-1 flex-col gap-1.5">
                  <Progress
                    value={
                      running.progress?.step
                        ? ((running.progress.step.current - 0.5) / running.progress.step.total) *
                          100
                        : null
                    }
                    label={t("runTitle")}
                  />
                  <p className="text-xs text-muted-foreground tabular-nums">
                    {running.progress?.step
                      ? t("running", {
                          current: String(running.progress.step.current),
                          total: String(running.progress.step.total),
                        })
                      : t("startingRun")}
                  </p>
                </div>
                <Button onClick={() => cancel.mutate(running.id)} disabled={cancel.isPending}>
                  <Square />
                  {t("cancel")}
                </Button>
              </div>
            ) : (
              <div className="flex items-center justify-end gap-3">
                {start.isError && (
                  <p className="flex-1 text-xs text-destructive">{start.error.message}</p>
                )}
                <Button
                  variant="default"
                  disabled={!ready || !target || chosen.length === 0 || start.isPending}
                  onClick={() =>
                    target &&
                    start.mutate({
                      model: target.id,
                      runtime: target.model.runtime,
                      units: chosen,
                      promptTokens,
                      generatedTokens,
                      repetitions,
                      powerMode,
                    })
                  }
                >
                  <Play />
                  {t("start")}
                </Button>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}

function Choice<T extends number>({
  id,
  label,
  value,
  options,
  format,
  onChange,
  disabled,
}: {
  id: string;
  label: string;
  value: T;
  options: readonly T[];
  format: (value: T) => string;
  onChange: (value: T) => void;
  disabled: boolean;
}) {
  return (
    <Field label={label} htmlFor={id}>
      <Select
        id={id}
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(Number(event.target.value) as T)}
      >
        {options.map((option) => (
          <option key={option} value={option}>
            {format(option)}
          </option>
        ))}
      </Select>
    </Field>
  );
}
