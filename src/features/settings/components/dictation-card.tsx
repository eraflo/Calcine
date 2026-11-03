import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, Select } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { dictationModel, understandsSpeech } from "@/features/chat/lib/dictation";
import { modelsQuery } from "@/features/library/api";
import { defineMessages, useT } from "@/i18n";
import {
  DICTATION_LANGUAGES,
  type DictationLanguage,
  type DictationUnit,
  useDictationSettings,
} from "@/stores/dictation";

const strings = defineMessages({
  en: {
    title: "Dictation",
    description:
      "The microphone button in the chat writes down what you say. It runs on this PC: your voice is never sent anywhere.",
    model: "Model",
    modelHint:
      "A model that understands speech transcribes. Gemma 4 E4B was tested in French and English.",
    automatic: "Automatic ({name})",
    automaticNone: "Automatic",
    noModel:
      "No installed model understands speech. Download ggml-org/gemma-4-E4B-it-GGUF in Discover to dictate.",
    unit: "Compute unit",
    unitHint:
      "GPU: fastest. CPU: works everywhere. NPU: the model on the NPU, your voice decoded on the CPU.",
    language: "Language",
    languageHint: "Telling the language helps with short sentences.",
    lang_auto: "Detect",
    lang_fr: "French",
    lang_en: "English",
    lang_es: "Spanish",
    lang_de: "German",
    lang_it: "Italian",
    lang_pt: "Portuguese",
  },
  fr: {
    title: "Dictée",
    description:
      "Le bouton micro du chat écrit ce que vous dites. Tout se passe sur ce PC : votre voix n'est envoyée nulle part.",
    model: "Modèle",
    modelHint:
      "Un modèle qui comprend la voix fait la transcription. Gemma 4 E4B a été testé en français et en anglais.",
    automatic: "Automatique ({name})",
    automaticNone: "Automatique",
    noModel:
      "Aucun modèle installé ne comprend la voix. Téléchargez ggml-org/gemma-4-E4B-it-GGUF dans Découvrir pour dicter.",
    unit: "Unité de calcul",
    unitHint:
      "GPU : le plus rapide. CPU : marche partout. NPU : le modèle sur le NPU, votre voix décodée sur le CPU.",
    language: "Langue",
    languageHint: "Indiquer la langue aide pour les phrases courtes.",
    lang_auto: "Détecter",
    lang_fr: "Français",
    lang_en: "Anglais",
    lang_es: "Espagnol",
    lang_de: "Allemand",
    lang_it: "Italien",
    lang_pt: "Portugais",
  },
});

const AUTOMATIC = "";

/** Which model writes down dictation, where it runs, and in which language. */
export function DictationCard() {
  const t = useT(strings);
  const { data: models = [] } = useQuery(modelsQuery);
  const { model, unit, language, setModel, setUnit, setLanguage } = useDictationSettings();
  const automatic = dictationModel(models, null);
  const listening = models.filter(understandsSpeech);
  // Any vision model can be tried; the ones known to listen come first.
  const others = models.filter((entry) => entry.modelType === "vlm" && !understandsSpeech(entry));

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Field label={t("model")} htmlFor="dictation-model" hint={t("modelHint")}>
          <Select
            id="dictation-model"
            value={model ?? AUTOMATIC}
            onChange={(event) => setModel(event.target.value || null)}
          >
            <option value={AUTOMATIC}>
              {automatic ? t("automatic", { name: automatic.name }) : t("automaticNone")}
            </option>
            {[...listening, ...others].map((entry) => (
              <option key={entry.name} value={entry.name}>
                {entry.name}
              </option>
            ))}
          </Select>
        </Field>
        {!automatic && model === null && <p className="text-xs text-warning">{t("noModel")}</p>}

        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex flex-col">
            <span className="text-sm">{t("unit")}</span>
            <span className="max-w-md text-[11px] text-muted-foreground">{t("unitHint")}</span>
          </div>
          <SegmentedControl<DictationUnit>
            name="dictation-unit"
            label={t("unit")}
            value={unit}
            onChange={setUnit}
            options={[
              { value: "gpu", label: "GPU" },
              { value: "cpu", label: "CPU" },
              { value: "npu", label: "NPU" },
            ]}
          />
        </div>

        <Field label={t("language")} htmlFor="dictation-language" hint={t("languageHint")}>
          <Select
            id="dictation-language"
            value={language}
            onChange={(event) => setLanguage(event.target.value as DictationLanguage)}
          >
            {DICTATION_LANGUAGES.map((code) => (
              <option key={code} value={code}>
                {t(`lang_${code}`)}
              </option>
            ))}
          </Select>
        </Field>
      </CardContent>
    </Card>
  );
}
