import * as Collapsible from "@radix-ui/react-collapsible";
import { useQuery } from "@tanstack/react-query";
import { ChevronRight, Dices, X } from "lucide-react";
import { type KeyboardEvent, type ReactNode, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Field, Input, Select } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Switch } from "@/components/ui/switch";
import { modelsQuery } from "@/features/library/api";
import { gatewayQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { LocalModel } from "@/lib/api";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import { clamp, escapeStop, parseOptional, unescapeStop } from "../lib/inputs";
import {
  activePreset,
  buildChatRequest,
  type ChatSettings,
  DEFAULT_SETTINGS,
  needsDraftModel,
  POWER_MODES,
  type PowerMode,
  PRESETS,
  type Preset,
  SPEC_TYPES,
  type SpecType,
  supportsComputeChoice,
  supportsMedia,
  toCurl,
  type VisionCompute,
} from "../lib/request";
import { messages } from "../messages";
import { useChat } from "../store";
import { SystemPromptField } from "./system-prompts";

const COMPUTE_UNITS = [
  { value: "npu", tone: "text-npu" },
  { value: "gpu", tone: "text-gpu" },
  { value: "cpu", tone: "text-cpu" },
] as const;

const ADVANCED_SAMPLING = [
  "topP",
  "topK",
  "minP",
  "repetitionPenalty",
  "presencePenalty",
  "frequencyPenalty",
  "seed",
  "stop",
] as const satisfies readonly (keyof ChatSettings)[];

const MODEL_LOADING = [
  "gpuLayers",
  "visionCompute",
] as const satisfies readonly (keyof ChatSettings)[];

const SPECULATIVE = [
  "specType",
  "draftModel",
  "draftMax",
  "draftMin",
  "draftPMin",
] as const satisfies readonly (keyof ChatSettings)[];

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
  const llamaCpp = supportsComputeChoice(model);
  const preset = activePreset(settings);

  return (
    <aside className="flex w-72 shrink-0 flex-col gap-5 overflow-y-auto overscroll-contain border-l p-4">
      <SystemPromptField />

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

      <Field label={t("style")} hint={t("styleHint")}>
        <SegmentedControl<Preset>
          name="preset"
          label={t("style")}
          value={preset}
          onChange={(name) => setSettings(PRESETS[name])}
          options={(Object.keys(PRESETS) as Preset[]).map((name) => ({
            value: name,
            label: t(`preset_${name}`),
          }))}
          className="w-full"
        />
      </Field>

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

      <Section
        title={t("moreSampling")}
        changed={ADVANCED_SAMPLING.some((key) => isChanged(settings, key))}
        onReset={() => setSettings(pick(DEFAULT_SETTINGS, ADVANCED_SAMPLING))}
      >
        <OptionalNumber
          id="top-p"
          label={t("topP")}
          hint={t("topPHint")}
          value={settings.topP}
          min={0}
          max={1}
          step={0.05}
          onChange={(topP) => setSettings({ topP })}
        />
        <OptionalNumber
          id="top-k"
          label={t("topK")}
          hint={t("topKHint")}
          value={settings.topK}
          min={0}
          max={1000}
          step={1}
          integer
          onChange={(topK) => setSettings({ topK })}
        />
        <OptionalNumber
          id="min-p"
          label={t("minP")}
          hint={t("minPHint")}
          value={settings.minP}
          min={0}
          max={1}
          step={0.01}
          onChange={(minP) => setSettings({ minP })}
        />
        <OptionalNumber
          id="repetition-penalty"
          label={t("repetitionPenalty")}
          hint={t("repetitionPenaltyHint")}
          value={settings.repetitionPenalty}
          min={0}
          max={2}
          step={0.05}
          onChange={(repetitionPenalty) => setSettings({ repetitionPenalty })}
        />
        <OptionalNumber
          id="presence-penalty"
          label={t("presencePenalty")}
          hint={t("presencePenaltyHint")}
          value={settings.presencePenalty}
          min={-2}
          max={2}
          step={0.1}
          onChange={(presencePenalty) => setSettings({ presencePenalty })}
        />
        <OptionalNumber
          id="frequency-penalty"
          label={t("frequencyPenalty")}
          hint={t("frequencyPenaltyHint")}
          value={settings.frequencyPenalty}
          min={-2}
          max={2}
          step={0.1}
          onChange={(frequencyPenalty) => setSettings({ frequencyPenalty })}
        />
        <SeedField
          value={settings.seed}
          disabled={!llamaCpp}
          onChange={(seed) => setSettings({ seed })}
        />
        <StopField value={settings.stop} onChange={(stop) => setSettings({ stop })} />
      </Section>

      <Field label={t("computeUnit")} hint={llamaCpp ? t("computeHint") : t("computeQairt")}>
        <fieldset className="grid grid-cols-3 gap-1 rounded-md border p-0.5" disabled={!llamaCpp}>
          <legend className="sr-only">{t("computeUnit")}</legend>
          {COMPUTE_UNITS.map(({ value, tone }) => {
            const active = llamaCpp ? settings.compute === value : value === "npu";
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

      <Section
        title={t("modelLoading")}
        changed={MODEL_LOADING.some((key) => isChanged(settings, key))}
        onReset={() => setSettings(pick(DEFAULT_SETTINGS, MODEL_LOADING))}
      >
        {llamaCpp ? (
          <OptionalNumber
            id="gpu-layers"
            label={t("gpuLayers")}
            hint={t("gpuLayersHint")}
            value={settings.gpuLayers}
            min={-1}
            max={999}
            step={1}
            integer
            onChange={(gpuLayers) => setSettings({ gpuLayers })}
          />
        ) : null}
        {llamaCpp && supportsMedia(model) ? (
          <Field label={t("visionEncoder")} hint={t("visionEncoderHint")}>
            <SegmentedControl<VisionCompute | "auto">
              name="vision-compute"
              label={t("visionEncoder")}
              value={settings.visionCompute ?? "auto"}
              onChange={(value) => setSettings({ visionCompute: value === "auto" ? null : value })}
              options={[
                { value: "auto", label: t("auto") },
                { value: "cpu", label: tc("cpu"), activeClassName: "text-cpu" },
                { value: "npu", label: tc("npu"), activeClassName: "text-npu" },
              ]}
              className="w-full"
            />
          </Field>
        ) : null}
        {llamaCpp ? null : (
          <p className="text-[11px] leading-snug text-muted-foreground">{t("loadingQairt")}</p>
        )}
        <p className="text-[11px] leading-snug text-muted-foreground">{t("contextOnServer")}</p>
      </Section>

      <Section
        title={t("speculative")}
        changed={SPECULATIVE.some((key) => isChanged(settings, key))}
        onReset={() => setSettings(pick(DEFAULT_SETTINGS, SPECULATIVE))}
      >
        {llamaCpp ? (
          <SpeculativeFields modelId={modelId} />
        ) : (
          <p className="text-[11px] leading-snug text-muted-foreground">{t("llamaCppOnly")}</p>
        )}
      </Section>

      {modelId && <RequestPreview model={model} modelId={modelId} settings={settings} />}
    </aside>
  );
}

/** Method, draft model and draft sizes for speculative decoding. */
function SpeculativeFields({ modelId }: { modelId: string | undefined }) {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  const { data: models = [] } = useQuery(modelsQuery);
  const base = modelId?.split(":")[0];
  // Draft candidates: other llama.cpp models, one entry per precision.
  const drafts = models
    .filter((candidate) => candidate.runtime === "llama_cpp" && candidate.name !== base)
    .flatMap((candidate) =>
      candidate.precisions.length
        ? candidate.precisions.map((precision) => `${candidate.name}:${precision}`)
        : [candidate.name],
    );
  const spec = settings.specType;
  const drafting = needsDraftModel(spec);

  return (
    <>
      <Field
        label={t("specMethod")}
        htmlFor="spec-type"
        hint={spec && !drafting ? t("specNgramHint") : t("speculativeHint")}
      >
        <Select
          id="spec-type"
          value={spec ?? ""}
          onChange={(event) =>
            setSettings({ specType: (event.target.value || null) as SpecType | null })
          }
        >
          <option value="">{t("spec_off")}</option>
          {SPEC_TYPES.map((type) => (
            <option key={type} value={type}>
              {t(`spec_${type}`)}
            </option>
          ))}
        </Select>
      </Field>
      {drafting && (
        <Field
          label={t("draftModel")}
          htmlFor="draft-model"
          hint={drafts.length ? t("draftModelHint") : t("noDraftModels")}
        >
          <Select
            id="draft-model"
            value={settings.draftModel ?? ""}
            disabled={drafts.length === 0}
            onChange={(event) => setSettings({ draftModel: event.target.value || null })}
            className="font-mono"
          >
            <option value="">{t("draftModelNone")}</option>
            {drafts.map((draft) => (
              <option key={draft} value={draft}>
                {draft}
              </option>
            ))}
          </Select>
        </Field>
      )}
      {spec && (
        <>
          <OptionalNumber
            id="draft-max"
            label={t("draftMax")}
            hint={t("draftMaxHint")}
            value={settings.draftMax}
            min={1}
            max={64}
            step={1}
            integer
            onChange={(draftMax) => setSettings({ draftMax })}
          />
          <OptionalNumber
            id="draft-min"
            label={t("draftMin")}
            hint=""
            value={settings.draftMin}
            min={0}
            max={64}
            step={1}
            integer
            onChange={(draftMin) => setSettings({ draftMin })}
          />
          <OptionalNumber
            id="draft-p-min"
            label={t("draftPMin")}
            hint={t("draftPMinHint")}
            value={settings.draftPMin}
            min={0}
            max={1}
            step={0.05}
            onChange={(draftPMin) => setSettings({ draftPMin })}
          />
        </>
      )}
    </>
  );
}

/** A collapsible group of options, flagged when some differ from the defaults. */
function Section({
  title,
  changed,
  onReset,
  children,
}: {
  title: string;
  changed: boolean;
  onReset: () => void;
  children: ReactNode;
}) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  return (
    <Collapsible.Root open={open} onOpenChange={setOpen} className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-2">
        <Collapsible.Trigger className="flex items-center gap-1 text-xs font-medium text-muted-foreground hover:text-foreground">
          <ChevronRight className={cn("size-3.5 transition-transform", open && "rotate-90")} />
          {title}
          {changed && (
            <span className="ml-1 size-1.5 rounded-full bg-primary" title={t("customized")} />
          )}
        </Collapsible.Trigger>
        {changed && open && (
          <button
            type="button"
            onClick={onReset}
            className="text-[11px] text-muted-foreground hover:text-foreground"
          >
            {t("resetDefaults")}
          </button>
        )}
      </div>
      <Collapsible.Content className="flex flex-col gap-4 border-l pl-3">
        {children}
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

/** A number that can be left empty to keep GenieX's default. */
function OptionalNumber({
  id,
  label,
  hint,
  value,
  min,
  max,
  step,
  integer = false,
  onChange,
}: {
  id: string;
  label: string;
  hint: string;
  value: number | null;
  min: number;
  max: number;
  step: number;
  integer?: boolean;
  onChange: (value: number | null) => void;
}) {
  const t = useT(messages);
  return (
    <Field label={label} htmlFor={id} hint={hint}>
      <Input
        id={id}
        type="number"
        inputMode={integer ? "numeric" : "decimal"}
        min={min}
        max={max}
        step={step}
        placeholder={t("auto")}
        value={value ?? ""}
        onChange={(event) => onChange(parseOptional(event.target.value, min, max, integer))}
      />
    </Field>
  );
}

function SeedField({
  value,
  disabled,
  onChange,
}: {
  value: number | null;
  disabled: boolean;
  onChange: (value: number | null) => void;
}) {
  const t = useT(messages);
  return (
    <Field label={t("seed")} htmlFor="seed" hint={disabled ? t("llamaCppOnly") : t("seedHint")}>
      <div className="flex gap-1.5">
        <Input
          id="seed"
          type="number"
          inputMode="numeric"
          min={0}
          max={MAX_SEED}
          step={1}
          placeholder={t("random")}
          disabled={disabled}
          value={value ?? ""}
          onChange={(event) => onChange(parseOptional(event.target.value, 0, MAX_SEED, true))}
        />
        <Button
          type="button"
          variant="secondary"
          size="icon"
          className="size-8 shrink-0"
          disabled={disabled}
          onClick={() => onChange(Math.floor(Math.random() * MAX_SEED))}
          aria-label={t("randomSeed")}
          title={t("randomSeed")}
        >
          <Dices />
        </Button>
      </div>
    </Field>
  );
}

function StopField({ value, onChange }: { value: string[]; onChange: (value: string[]) => void }) {
  const t = useT(messages);
  const [draft, setDraft] = useState("");
  const add = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    const sequence = unescapeStop(draft);
    if (sequence && !value.includes(sequence)) onChange([...value, sequence]);
    setDraft("");
  };
  return (
    <Field label={t("stopSequences")} htmlFor="stop" hint={t("stopHint")}>
      {value.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {value.map((sequence) => (
            <Badge key={sequence} tone="outline" className="gap-0.5 pr-0.5 font-mono">
              {escapeStop(sequence)}
              <button
                type="button"
                onClick={() => onChange(value.filter((other) => other !== sequence))}
                className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("removeStop", { value: escapeStop(sequence) })}
              >
                <X />
              </button>
            </Badge>
          ))}
        </div>
      )}
      <Input
        id="stop"
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={add}
        placeholder={t("stopPlaceholder")}
        spellCheck={false}
        className="font-mono"
      />
    </Field>
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

/** GenieX takes the seed as a 32-bit signed integer. */
const MAX_SEED = 2_147_483_647;

function isChanged(settings: ChatSettings, key: keyof ChatSettings) {
  const value = settings[key];
  return Array.isArray(value) ? value.length > 0 : value !== DEFAULT_SETTINGS[key];
}

function pick<K extends keyof ChatSettings>(settings: ChatSettings, keys: readonly K[]) {
  return Object.fromEntries(keys.map((key) => [key, settings[key]])) as Pick<ChatSettings, K>;
}
