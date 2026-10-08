import { Link } from "@tanstack/react-router";
import { RuntimeBadge } from "@/components/calcine/badges/runtime-badge";
import { Select } from "@/components/ui/field";
import { useT } from "@/i18n";
import type { LocalModel } from "@/lib/api";
import { messages } from "../messages";

/** Pick one of the downloaded models. */
export function ModelPicker({
  models,
  value,
  onChange,
}: {
  models: LocalModel[];
  value: string | undefined;
  onChange: (modelId: string) => void;
}) {
  const t = useT(messages);
  if (models.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        {t("noModel")}{" "}
        <Link to="/discover" className="text-info hover:underline">
          {t("downloadOne")}
        </Link>
      </p>
    );
  }
  const selected = models.find((model) => model.name === value);
  return (
    <div className="flex items-center gap-2">
      <Select
        value={value ?? ""}
        onChange={(event) => onChange(event.target.value)}
        aria-label={t("model")}
        className="w-72 font-mono text-[12px]"
      >
        {!selected && <option value="">{t("pickModel")}</option>}
        {models.map((model) => (
          <option key={model.name} value={model.name}>
            {model.name}
          </option>
        ))}
      </Select>
      {selected && <RuntimeBadge runtime={selected.runtime} />}
    </div>
  );
}
