import { Dices } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Field, Input } from "@/components/ui/field";
import { useT } from "@/i18n";
import { parseOptional } from "../../lib/inputs";
import { messages } from "../../messages";
import { MAX_SEED } from "./options";

/** Same seed and settings give the same reply (llama.cpp only). */
export function SeedField({
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
