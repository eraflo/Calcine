import { X } from "lucide-react";
import { type KeyboardEvent, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Field, Input } from "@/components/ui/field";
import { useT } from "@/i18n";
import { escapeStop, unescapeStop } from "../../lib/inputs";
import { messages } from "../../messages";

/** Sequences that end the reply when the model writes them. */
export function StopField({
  value,
  onChange,
}: {
  value: string[];
  onChange: (value: string[]) => void;
}) {
  const t = useT(messages);
  const [draft, setDraft] = useState("");
  const add = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    const sequence = unescapeStop(draft);
    if (sequence && !value.includes(sequence)) onChange([...value, sequence]);
    setDraft("");
  };
  return (
    <Field label={t("stopSequences")} htmlFor="stop" hint={t("stopHint")}>
      {value.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {value.map((sequence) => (
            <Badge key={sequence} tone="outline" className="gap-0.5 pr-0.5 font-mono">
              {escapeStop(sequence)}
              <button
                type="button"
                onClick={() => onChange(value.filter((other) => other !== sequence))}
                className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("removeStop", { value: escapeStop(sequence) })}
              >
                <X />
              </button>
            </Badge>
          ))}
        </div>
      )}
      <Input
        id="stop"
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={add}
        placeholder={t("stopPlaceholder")}
        spellCheck={false}
        className="font-mono"
      />
    </Field>
  );
}
