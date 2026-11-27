import { Field, Input } from "@/components/ui/field";
import { useT } from "@/i18n";
import { parseOptional } from "../../lib/inputs";
import { settingsMessages as messages } from "./messages";

/** A number that can be left empty to keep GenieX's default. */
export function OptionalNumber({
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
