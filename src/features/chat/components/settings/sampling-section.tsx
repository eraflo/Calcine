import { useT } from "@/i18n";
import { DEFAULT_SETTINGS } from "../../lib/request";
import { messages } from "../../messages";
import { useChat } from "../../store";
import { OptionalNumber } from "./optional-number";
import { ADVANCED_SAMPLING, isChanged, pick } from "./options";
import { Section } from "./section";
import { SeedField } from "./seed-field";
import { StopField } from "./stop-field";

/** Samplers beyond temperature, folded by default. */
export function SamplingSection({ llamaCpp }: { llamaCpp: boolean }) {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  return (
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
  );
}
