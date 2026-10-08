import { Link } from "@tanstack/react-router";
import { RuntimeBadge } from "@/components/calcine/badges/runtime-badge";
import { Select } from "@/components/ui/field";
import type { LocalModel } from "@/lib/api";

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
  if (models.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        No model yet.{" "}
        <Link to="/discover" className="text-info hover:underline">
          Download one
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
        aria-label="Model"
        className="w-72 font-mono text-[12px]"
      >
        {!selected && <option value="">Pick a model</option>}
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
