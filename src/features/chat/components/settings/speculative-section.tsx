import { useT } from "@/i18n";
import { generationMessages } from "@/i18n/generation";
import { DEFAULT_SETTINGS } from "../../lib/request";
import { useChat } from "../../store";
import { settingsMessages as messages } from "./messages";
import { isChanged, pick, SPECULATIVE } from "./options";
import { Section } from "./section";
import { SpeculativeFields } from "./speculative-fields";

/** Speculative decoding, for llama.cpp models. */
export function SpeculativeSection({
  modelId,
  llamaCpp,
}: {
  modelId: string | undefined;
  llamaCpp: boolean;
}) {
  const t = useT(messages);
  const tg = useT(generationMessages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  return (
    <Section
      title={tg("speculative")}
      changed={SPECULATIVE.some((key) => isChanged(settings, key))}
      onReset={() => setSettings(pick(DEFAULT_SETTINGS, SPECULATIVE))}
    >
      {llamaCpp ? (
        <SpeculativeFields modelId={modelId} />
      ) : (
        <p className="text-[11px] leading-snug text-muted-foreground">{t("llamaCppOnly")}</p>
      )}
    </Section>
  );
}
