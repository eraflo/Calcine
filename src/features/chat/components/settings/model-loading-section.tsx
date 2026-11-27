import { Field } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { LocalModel } from "@/lib/api";
import { DEFAULT_SETTINGS, supportsMedia, type VisionCompute } from "../../lib/request";
import { useChat } from "../../store";
import { settingsMessages as messages } from "./messages";
import { OptionalNumber } from "./optional-number";
import { isChanged, MODEL_LOADING, pick } from "./options";
import { Section } from "./section";

/** How llama.cpp loads the model: GPU layers, vision encoder. */
export function ModelLoadingSection({
  model,
  llamaCpp,
}: {
  model: LocalModel | undefined;
  llamaCpp: boolean;
}) {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  const tc = useT(common);
  return (
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
  );
}
