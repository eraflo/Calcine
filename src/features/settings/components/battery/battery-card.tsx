import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, Select } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { POWER_MODES, type PowerMode } from "@/features/chat/lib/request";
import { powerSourceQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { generationMessages } from "@/i18n/generation";
import { commands, unwrap } from "@/lib/api";
import { gatewaySettingsQuery } from "../system-cards";
import { messages } from "./messages";
import { isPowerMode } from "./mode";
import { PowerStatus } from "./power-status";

const SUGGESTED: PowerMode = "power_saver";

/** On battery, run models in a power-saving mode. Hidden on PCs without a battery. */
export function BatteryCard() {
  const t = useT(messages);
  const tg = useT(generationMessages);
  const queryClient = useQueryClient();
  const { data: source } = useQuery(powerSourceQuery);
  const settings = useQuery(gatewaySettingsQuery);
  const saved = settings.data?.batteryPowerMode;
  const enabled = isPowerMode(saved);
  // Kept while the saver is off, so turning it back on restores it.
  const [choice, setChoice] = useState<PowerMode>(SUGGESTED);
  useEffect(() => {
    if (isPowerMode(saved)) setChoice(saved);
  }, [saved]);

  const save = useMutation({
    mutationFn: (mode: PowerMode | null) => unwrap(() => commands.setBatteryPowerMode(mode)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: gatewaySettingsQuery.queryKey }),
  });

  if (!source) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <PowerStatus source={source} saving={enabled} />
        <div className="flex items-center justify-between gap-4">
          <label htmlFor="battery-saver" className="flex flex-col">
            <span className="text-sm">{t("enable")}</span>
            <span className="text-[11px] text-muted-foreground">{t("enableHint")}</span>
          </label>
          <Switch
            id="battery-saver"
            checked={enabled}
            disabled={settings.isPending || save.isPending}
            onCheckedChange={(on) => save.mutate(on ? choice : null)}
          />
        </div>
        <Field label={t("mode")} htmlFor="battery-mode" hint={t("modeHint")}>
          <Select
            id="battery-mode"
            value={choice}
            disabled={!enabled || save.isPending}
            onChange={(event) => {
              const mode = event.target.value as PowerMode;
              setChoice(mode);
              save.mutate(mode);
            }}
          >
            {POWER_MODES.map((mode) => (
              <option key={mode} value={mode}>
                {tg(`power_${mode}`)}
              </option>
            ))}
          </Select>
        </Field>
        {save.isError && <p className="text-xs text-destructive">{save.error.message}</p>}
      </CardContent>
    </Card>
  );
}
