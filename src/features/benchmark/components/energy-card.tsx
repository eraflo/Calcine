import { useQuery } from "@tanstack/react-query";
import { Leaf, Play, Square, Zap } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, Select } from "@/components/ui/field";
import { Progress } from "@/components/ui/progress";
import { messages as chatMessages } from "@/features/chat/messages";
import { usageQuery } from "@/features/hardware/api";
import { modelsQuery } from "@/features/library/api";
import { useCancelJob } from "@/features/tasks/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { BenchResult, ComputeUnit } from "@/lib/api";
import { formatNumber, formatRelative } from "@/lib/format";
import { cn } from "@/lib/utils";
import { benchHistoryQuery, useRunningJob, useStartEnergyProfile } from "../api";
import {
  benchTargets,
  type EnergyRow,
  energyRows,
  mostEfficient,
  POWER_MODE_ORDER,
  sessions,
} from "../lib/results";
import { messages } from "../messages";

const GENERATED = [128, 256, 512] as const;
const REPETITIONS = [1, 2, 3] as const;
/** Power modes made no difference on the devices measured: one is enough. */
const DEFAULT_MODES = ["burst"];
const ENERGY_UNITS: ComputeUnit[] = ["npu", "gpu", "cpu"];

/** Compare a model's speed and energy in each power mode. */
export function EnergyCard() {
  const t = useT(messages);
  const { data: usage } = useQuery(usageQuery);
  const { data: history = [] } = useQuery(benchHistoryQuery);
  const metered = Boolean(usage?.power);
  const latest = sessions(history).find((session) => session.source === "energy_profile");

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Zap className="size-4 text-warning" />
          {t("energyTitle")}
        </CardTitle>
        <CardDescription>{t("energyHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-5">
        {usage && !metered ? (
          <p className="text-sm text-muted-foreground">{t("energyUnavailable")}</p>
        ) : (
          <EnergyForm />
        )}
        {latest && <EnergyResults results={latest.results} startedAtMs={latest.startedAtMs} />}
      </CardContent>
    </Card>
  );
}

function EnergyForm() {
  const t = useT(messages);
  const tc = useT(common);
  const tchat = useT(chatMessages);
  const { data: models = [] } = useQuery(modelsQuery);
  const targets = benchTargets(models);
  const [targetId, setTargetId] = useState("");
  const target = targets.find((candidate) => candidate.id === targetId) ?? targets[0];
  const llamaCpp = target?.model.runtime === "llama_cpp";
  const [units, setUnits] = useState<ComputeUnit[]>(ENERGY_UNITS);
  const [modes, setModes] = useState<string[]>(DEFAULT_MODES);
  const [generatedTokens, setGeneratedTokens] = useState(256);
  const [repetitions, setRepetitions] = useState(2);
  const start = useStartEnergyProfile();
  const cancel = useCancelJob();
  const running = useRunningJob("energy_profile");
  const busy = Boolean(running);

  if (targets.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("noModels")}</p>;
  }
  const toggleMode = (mode: string) =>
    setModes((current) =>
      current.includes(mode) ? current.filter((other) => other !== mode) : [...current, mode],
    );
  const toggleUnit = (unit: ComputeUnit) =>
    setUnits((current) =>
      current.includes(unit) ? current.filter((other) => other !== unit) : [...current, unit],
    );
  const chosen = POWER_MODE_ORDER.filter((mode) => modes.includes(mode));
  // AI Hub models only run on the NPU.
  const available: ComputeUnit[] = llamaCpp ? ENERGY_UNITS : ["npu"];
  const chosenUnits = available.filter((unit) => units.includes(unit));

  return (
    <div className="flex flex-col gap-4">
      <div className="grid gap-4 sm:grid-cols-[2fr_1fr_1fr]">
        <Field label={t("model")} htmlFor="energy-model">
          <Select
            id="energy-model"
            value={target?.id ?? ""}
            onChange={(event) => setTargetId(event.target.value)}
            className="font-mono"
            disabled={busy}
          >
            {targets.map(({ id }) => (
              <option key={id} value={id}>
                {id}
              </option>
            ))}
          </Select>
        </Field>
        <Field label={t("generatedTokens")} htmlFor="energy-generated">
          <Select
            id="energy-generated"
            value={generatedTokens}
            onChange={(event) => setGeneratedTokens(Number(event.target.value))}
            disabled={busy}
          >
            {GENERATED.map((count) => (
              <option key={count} value={count}>
                {t("tokens", { count: String(count) })}
              </option>
            ))}
          </Select>
        </Field>
        <Field label={t("repetitions")} htmlFor="energy-repetitions">
          <Select
            id="energy-repetitions"
            value={repetitions}
            onChange={(event) => setRepetitions(Number(event.target.value))}
            disabled={busy}
          >
            {REPETITIONS.map((count) => (
              <option key={count} value={count}>
                {t("runs", { count: String(count) })}
              </option>
            ))}
          </Select>
        </Field>
      </div>
      <div className="grid gap-4 sm:grid-cols-[1fr_2fr]">
        <Field label={t("energyUnit")} hint={llamaCpp ? undefined : t("unitsQairt")}>
          <Chips
            options={ENERGY_UNITS.map((unit) => ({
              value: unit,
              label: tc(unit),
              disabled: !available.includes(unit),
            }))}
            selected={chosenUnits}
            onToggle={toggleUnit}
            disabled={busy}
          />
        </Field>
        <Field label={t("energyModes")}>
          <Chips
            options={POWER_MODE_ORDER.map((mode) => ({
              value: mode,
              label: tchat(`power_${mode}`),
            }))}
            selected={chosen}
            onToggle={toggleMode}
            disabled={busy}
          />
        </Field>
      </div>

      {running ? (
        <div className="flex items-center gap-3">
          <div className="flex flex-1 flex-col gap-1.5">
            <Progress
              value={
                running.progress?.step
                  ? ((running.progress.step.current - 0.5) / running.progress.step.total) * 100
                  : null
              }
              label={t("energyTitle")}
              barClassName="bg-warning"
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
          <p
            className={cn(
              "flex-1 text-xs",
              start.isError ? "text-destructive" : "text-muted-foreground",
            )}
          >
            {start.isError ? start.error.message : t("energyQuiet")}
          </p>
          <Button
            variant="default"
            disabled={!target || chosen.length === 0 || chosenUnits.length === 0 || start.isPending}
            onClick={() =>
              target &&
              start.mutate({
                model: target.id,
                runtime: target.model.runtime,
                units: chosenUnits,
                powerModes: chosen,
                generatedTokens,
                repetitions,
              })
            }
          >
            <Play />
            {t("energyStart")}
          </Button>
        </div>
      )}
    </div>
  );
}

/** Toggle buttons for a multiple choice. */
function Chips<T extends string>({
  options,
  selected,
  onToggle,
  disabled,
}: {
  options: { value: T; label: string; disabled?: boolean }[];
  selected: readonly T[];
  onToggle: (value: T) => void;
  disabled: boolean;
}) {
  return (
    <div className="flex flex-wrap gap-1.5">
      {options.map((option) => {
        const active = selected.includes(option.value);
        return (
          <label
            key={option.value}
            className={cn(
              "flex h-8 cursor-pointer items-center rounded-md border px-3 text-xs font-medium transition-colors has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-40 has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
              active
                ? "border-warning bg-accent text-warning"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            <input
              type="checkbox"
              className="sr-only"
              checked={active}
              onChange={() => onToggle(option.value)}
              disabled={disabled || option.disabled}
            />
            {option.label}
          </label>
        );
      })}
    </div>
  );
}

function EnergyResults({
  results,
  startedAtMs,
}: {
  results: readonly BenchResult[];
  startedAtMs: number;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const tchat = useT(chatMessages);
  const rows = energyRows(results);
  if (rows.length === 0) return null;
  const efficient = mostEfficient(rows);
  const fastest = rows.reduce((best, row) => (row.decodeTps > best.decodeTps ? row : best));
  const top = Math.max(...rows.map((row) => row.tokensPerJoule));
  const idle = rows[0]?.result.measure?.energy?.idleWatts;
  // The unit only shows when several were measured, likewise the mode.
  const severalUnits = new Set(rows.map((row) => row.unit)).size > 1;
  const severalModes = new Set(rows.map((row) => row.mode)).size > 1;
  const mode = (row: EnergyRow) =>
    [
      severalUnits || !severalModes ? tc(row.unit) : null,
      severalModes ? tchat(`power_${row.mode}`) : null,
    ]
      .filter(Boolean)
      .join(" · ");

  return (
    <div className="flex flex-col gap-3 border-t pt-4">
      <div className="flex flex-wrap items-baseline justify-between gap-2 text-xs text-muted-foreground">
        <span className="font-mono">{results[0]?.model}</span>
        <span>
          {formatRelative(startedAtMs)}
          {idle !== undefined && ` · ${t("energyIdle", { value: watts(idle) })}`}
        </span>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-xs text-muted-foreground">
              <th className="py-1.5 pr-3 font-medium">
                {severalModes
                  ? severalUnits
                    ? t("energyUnitMode")
                    : t("energyMode")
                  : t("energyUnit")}
              </th>
              <th className="py-1.5 pr-3 text-right font-medium">{t("energySpeed")}</th>
              <th className="py-1.5 pr-3 text-right font-medium">{t("energyPower")}</th>
              <th className="py-1.5 pr-3 text-right font-medium">{t("energyPerToken")}</th>
              <th className="w-2/5 py-1.5 font-medium">{t("energyEfficiency")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.result.id} className="border-t">
                <td className="py-2 pr-3">
                  <span className="flex items-center gap-2">
                    {mode(row)}
                    {row === efficient && (
                      <Badge variant="success">
                        <Leaf className="size-3" />
                        {t("mostEfficient")}
                      </Badge>
                    )}
                  </span>
                </td>
                <td className="py-2 pr-3 text-right tabular-nums">
                  {t("tps", { value: formatNumber(row.decodeTps, 1) })}
                </td>
                <td className="py-2 pr-3 text-right tabular-nums">
                  {watts(row.activeWatts)}
                  <span className="block text-[11px] text-muted-foreground">
                    {t("energyExtra", { value: watts(row.extraWatts) })}
                  </span>
                </td>
                <td className="py-2 pr-3 text-right tabular-nums">
                  {t("millijoules", { value: formatNumber(row.joulesPerToken * 1000, 0) })}
                  <span className="block text-[11px] text-muted-foreground">
                    {t("energySystem", {
                      value: t("millijoules", {
                        value: formatNumber(row.systemJoulesPerToken * 1000, 0),
                      }),
                    })}
                  </span>
                </td>
                <td className="py-2">
                  <span className="flex items-center gap-2">
                    <span className="h-2 flex-1 overflow-hidden rounded-full bg-muted">
                      <span
                        className={cn(
                          "block h-full rounded-full",
                          row === efficient ? "bg-success" : "bg-warning",
                        )}
                        style={{ width: `${top > 0 ? (row.tokensPerJoule / top) * 100 : 0}%` }}
                      />
                    </span>
                    <span className="w-10 text-right text-xs tabular-nums">
                      {formatNumber(row.tokensPerJoule, 1)}
                    </span>
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {efficient && rows.length > 1 && (
        <p className="text-xs text-muted-foreground">
          {efficient === fastest
            ? t("energySame", { mode: mode(efficient) })
            : t("energySummary", {
                efficient: mode(efficient),
                fastest: mode(fastest),
                ratio: formatNumber(efficient.tokensPerJoule / fastest.tokensPerJoule, 1),
                slower: formatNumber((1 - efficient.decodeTps / fastest.decodeTps) * 100, 0),
              })}
        </p>
      )}
    </div>
  );
}

function watts(value: number): string {
  return `${formatNumber(value, 1)} W`;
}
