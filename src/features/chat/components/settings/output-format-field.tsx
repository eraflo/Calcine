import { Field, Textarea } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { useT } from "@/i18n";
import { type OutputFormat, parseSchema } from "../../lib/request";
import { messages } from "../../messages";
import { useChat } from "../../store";

/** Text, any JSON object, or JSON matching a schema. */
export function OutputFormatField() {
  const t = useT(messages);
  const format = useChat((state) => state.settings.outputFormat);
  const schema = useChat((state) => state.settings.jsonSchema);
  const setSettings = useChat((state) => state.setSettings);
  const parsed = format === "schema" && schema.trim() ? parseSchema(schema) : null;
  return (
    <Field label={t("outputFormat")} hint={format === "text" ? undefined : t("outputFormatHint")}>
      <SegmentedControl<OutputFormat>
        name="output-format"
        label={t("outputFormat")}
        value={format}
        onChange={(outputFormat) => setSettings({ outputFormat })}
        options={(["text", "json", "schema"] as const).map((value) => ({
          value,
          label: t(`format_${value}`),
        }))}
        className="w-full"
      />
      {format === "schema" && (
        <>
          <Textarea
            value={schema}
            onChange={(event) => setSettings({ jsonSchema: event.target.value })}
            placeholder={t("jsonSchemaPlaceholder")}
            aria-label={t("jsonSchema")}
            spellCheck={false}
            className="min-h-32 font-mono text-[11px]"
          />
          {parsed && "error" in parsed && (
            <p className="text-[11px] text-destructive">{t(`schema_${parsed.error}`)}</p>
          )}
        </>
      )}
    </Field>
  );
}
