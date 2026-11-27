import { useT } from "@/i18n";
import { DEFAULT_SETTINGS } from "../../lib/request";
import { messages } from "../../messages";
import { useChat } from "../../store";
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
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  return (
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
  );
}
