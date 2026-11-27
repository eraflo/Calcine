import { Field, Input } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Switch } from "@/components/ui/switch";
import { useT } from "@/i18n";
import { formatNumber } from "@/lib/format";
import { clamp } from "../../lib/inputs";
import { activePreset, PRESETS, type Preset } from "../../lib/request";
import { messages } from "../../messages";
import { useChat } from "../../store";
import { Value } from "./value";

/** Thinking, style, temperature and reply length. */
export function GenerationFields() {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  const preset = activePreset(settings);
  return (
    <>
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
    </>
  );
}
