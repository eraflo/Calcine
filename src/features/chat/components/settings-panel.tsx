import * as Collapsible from "@radix-ui/react-collapsible";
import { useQuery } from "@tanstack/react-query";
import { ChevronRight } from "lucide-react";
import { type ReactNode, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Field, Input, Select, Textarea } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { gatewayQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { LocalModel } from "@/lib/api";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import {
  buildChatRequest,
  type ChatSettings,
  POWER_MODES,
  type PowerMode,
  supportsComputeChoice,
  toCurl,
} from "../lib/request";
import { messages } from "../messages";
import { useChat } from "../store";

const COMPUTE_UNITS = [
  { value: "npu", tone: "text-npu" },
  { value: "gpu", tone: "text-gpu" },
  { value: "cpu", tone: "text-cpu" },
] as const;

/** Generation options, adapted to what the selected model supports. */
export function SettingsPanel({
  model,
  modelId,
}: {
  model: LocalModel | undefined;
  modelId: string | undefined;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  const canChooseCompute = supportsComputeChoice(model);

  return (
    <aside className="flex w-72 shrink-0 flex-col gap-5 overflow-y-auto overscroll-contain border-l p-4">
      <Field label={t("systemPrompt")} htmlFor="system-prompt">
        <Textarea
          id="system-prompt"
          value={settings.systemPrompt}
          onChange={(event) => setSettings({ systemPrompt: event.target.value })}
          placeholder={t("systemPromptPlaceholder")}
          className="min-h-24"
        />
      </Field>

      <div className="flex items-center justify-between gap-3">
        <label htmlFor="thinking" className="flex flex-col">
          <span className="text-xs font-medium text-muted-foreground">{t("thinkingLabel")}</span>
          <span className="text-[11px] text-muted-foreground">{t("thinkingHint")}</span>
        </label>
        <Switch
          id="thinking"
          checked={settings.think}
          onCheckedChange={(think) => setSettings({ think })}
        />
      </div>

      <Field
        label={t("temperature")}
        htmlFor="temperature"
        aside={<Value>{formatNumber(settings.temperature, 1)}</Value>}
      >
        <input
          id="temperature"
          type="range"
          min={0}
          max={2}
          step={0.1}
          value={settings.temperature}
          onChange={(event) => setSettings({ temperature: Number(event.target.value) })}
          className="accent-[var(--primary)]"
        />
      </Field>

      <Field label={t("maxTokens")} htmlFor="max-tokens" hint={t("maxTokensHint")}>
        <Input
          id="max-tokens"
          type="number"
          min={16}
          max={8192}
          step={16}
          value={settings.maxTokens}
          onChange={(event) =>
            setSettings({ maxTokens: clamp(Number(event.target.value), 16, 8192) })
          }
        />
      </Field>

      <Field
        label={t("computeUnit")}
        hint={canChooseCompute ? t("computeHint") : t("computeQairt")}
      >
        <fieldset
          className="grid grid-cols-3 gap-1 rounded-md border p-0.5"
          disabled={!canChooseCompute}
        >
          <legend className="sr-only">{t("computeUnit")}</legend>
          {COMPUTE_UNITS.map(({ value, tone }) => {
            const active = canChooseCompute ? settings.compute === value : value === "npu";
            return (
              <label
                key={value}
                className={cn(
                  "flex h-7 cursor-pointer items-center justify-center rounded-sm text-xs font-medium text-muted-foreground transition-colors hover:text-foreground has-[:disabled]:cursor-not-allowed has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
                  active
                    ? cn("bg-accent", tone)
                    : "has-[:disabled]:opacity-40 has-[:disabled]:hover:text-muted-foreground",
                )}
              >
                <input
                  type="radio"
                  name="compute"
                  value={value}
                  checked={active}
                  onChange={() => setSettings({ compute: value })}
                  className="sr-only"
                />
                {tc(value)}
              </label>
            );
          })}
        </fieldset>
      </Field>

      <Field label={t("powerMode")} htmlFor="power-mode" hint={t("powerModeHint")}>
        <Select
          id="power-mode"
          value={settings.powerMode}
          onChange={(event) => setSettings({ powerMode: event.target.value as PowerMode })}
        >
          {POWER_MODES.map((value) => (
            <option key={value} value={value}>
              {t(`power_${value}`)}
            </option>
          ))}
        </Select>
      </Field>

      {modelId && <RequestPreview model={model} modelId={modelId} settings={settings} />}
    </aside>
  );
}

/** The exact API call Calcine makes, to reuse from another app. */
function RequestPreview({
  model,
  modelId,
  settings,
}: {
  model: LocalModel | undefined;
  modelId: string;
  settings: ChatSettings;
}) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  const { data: status } = useQuery(gatewayQuery);
  const url = `${status?.baseUrl ?? "http://127.0.0.1:18181/v1"}/chat/completions`;
  const curl = toCurl(
    url,
    buildChatRequest(model, modelId, settings, [{ role: "user", content: "Hello!" }]),
  );
  return (
    <Collapsible.Root open={open} onOpenChange={setOpen} className="border-t pt-4">
      <Collapsible.Trigger className="flex items-center gap-1 text-xs font-medium text-muted-foreground hover:text-foreground">
        <ChevronRight className={cn("size-3.5 transition-transform", open && "rotate-90")} />
        {t("equivalentRequest")}
      </Collapsible.Trigger>
      <Collapsible.Content className="relative mt-2">
        <pre className="overflow-x-auto rounded-md border bg-background p-2.5 pr-9 font-mono text-[10.5px] leading-relaxed">
          {curl}
        </pre>
        <CopyButton value={curl} className="absolute top-1 right-1" />
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

function Value({ children }: { children: ReactNode }) {
  return (
    <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{children}</span>
  );
}

function clamp(value: number, min: number, max: number) {
  return Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : min;
}
