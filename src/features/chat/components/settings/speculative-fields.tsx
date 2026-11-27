import { useQuery } from "@tanstack/react-query";
import { Field, Select } from "@/components/ui/field";
import { modelsQuery } from "@/features/library/api";
import { useT } from "@/i18n";
import { generationMessages } from "@/i18n/generation";
import { needsDraftModel, SPEC_TYPES, type SpecType } from "../../lib/request";
import { useChat } from "../../store";
import { settingsMessages as messages } from "./messages";
import { OptionalNumber } from "./optional-number";

/** Method, draft model and draft sizes for speculative decoding. */
export function SpeculativeFields({ modelId }: { modelId: string | undefined }) {
  const t = useT(messages);
  const tg = useT(generationMessages);
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
          <option value="">{tg("spec_off")}</option>
          {SPEC_TYPES.map((type) => (
            <option key={type} value={type}>
              {tg(`spec_${type}`)}
            </option>
          ))}
        </Select>
      </Field>
      {drafting && (
        <Field
          label={tg("draftModel")}
          htmlFor="draft-model"
          hint={drafts.length ? tg("draftModelHint") : tg("noDraftModels")}
        >
          <Select
            id="draft-model"
            value={settings.draftModel ?? ""}
            disabled={drafts.length === 0}
            onChange={(event) => setSettings({ draftModel: event.target.value || null })}
            className="font-mono"
          >
            <option value="">{tg("draftModelNone")}</option>
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
