import { Field, Select } from "@/components/ui/field";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { cn } from "@/lib/utils";
import { POWER_MODES, type PowerMode } from "../../lib/request";
import { messages } from "../../messages";
import { useChat } from "../../store";
import { COMPUTE_UNITS } from "./options";

/** Where the model runs, and how hard the NPU works. */
export function ComputeFields({ llamaCpp }: { llamaCpp: boolean }) {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  const tc = useT(common);
  return (
    <>
      <Field label={t("computeUnit")} hint={llamaCpp ? t("computeHint") : t("computeQairt")}>
        <fieldset className="grid grid-cols-3 gap-1 rounded-md border p-0.5" disabled={!llamaCpp}>
          <legend className="sr-only">{t("computeUnit")}</legend>
          {COMPUTE_UNITS.map(({ value, tone }) => {
            const active = llamaCpp ? settings.compute === value : value === "npu";
            return (
              <label
                key={value}
                className={cn(
                  "flex h-7 cursor-pointer items-center justify-center rounded-sm text-xs font-medium text-muted-foreground transition-colors hover:text-foreground has-[:disabled]:cursor-not-allowed has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
                  active
                    ? cn("bg-accent", tone)
                    : "has-[:disabled]:opacity-40 has-[:disabled]:hover:text-muted-foreground",
                )}
              >
                <input
                  type="radio"
                  name="compute"
                  value={value}
                  checked={active}
                  onChange={() => setSettings({ compute: value })}
                  className="sr-only"
                />
                {tc(value)}
              </label>
            );
          })}
        </fieldset>
      </Field>

      <Field label={t("powerMode")} htmlFor="power-mode" hint={t("powerModeHint")}>
        <Select
          id="power-mode"
          value={settings.powerMode}
          onChange={(event) => setSettings({ powerMode: event.target.value as PowerMode })}
        >
          {POWER_MODES.map((value) => (
            <option key={value} value={value}>
              {t(`power_${value}`)}
            </option>
          ))}
        </Select>
      </Field>
    </>
  );
}
