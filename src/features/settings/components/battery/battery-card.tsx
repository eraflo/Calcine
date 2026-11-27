import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, Select } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { POWER_MODES } from "@/features/chat/lib/request";
import { powerSourceQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { generationMessages } from "@/i18n/generation";
import { type BatterySaver, commands, unwrap } from "@/lib/api";
import { gatewaySettingsQuery } from "../system-cards";
import { messages } from "./messages";
import { PowerStatus } from "./power-status";
import { REPLY_LIMITS, saverOf } from "./saver";

const NO_LIMIT = "";

/** On battery, fewer tokens: no thinking, shorter replies. Hidden on PCs without a battery. */
export function BatteryCard() {
  const t = useT(messages);
  const tg = useT(generationMessages);
  const queryClient = useQueryClient();
  const { data: source } = useQuery(powerSourceQuery);
  const settings = useQuery(gatewaySettingsQuery);
  const saver = saverOf(settings.data?.batterySaver);

  const save = useMutation({
    mutationFn: (next: BatterySaver) => unwrap(() => commands.setBatterySaver(next)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: gatewaySettingsQuery.queryKey }),
  });
  const change = (patch: Partial<BatterySaver>) => save.mutate({ ...saver, ...patch });
  const busy = settings.isPending || save.isPending;
  const off = !saver.enabled || busy;

  if (!source) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <PowerStatus source={source} saving={saver.enabled} />
        <Toggle
          id="battery-saver"
          label={t("enable")}
          hint={t("enableHint")}
          checked={saver.enabled}
          disabled={busy}
          onChange={(enabled) => change({ enabled })}
        />
        <div className="flex flex-col gap-4 border-l-2 pl-4">
          <Toggle
            id="battery-no-thinking"
            label={t("noThinking")}
            hint={t("noThinkingHint")}
            checked={saver.noThinking}
            disabled={off}
            onChange={(noThinking) => change({ noThinking })}
          />
          <Field label={t("replyLimit")} htmlFor="battery-reply-limit" hint={t("replyLimitHint")}>
            <Select
              id="battery-reply-limit"
              value={saver.maxReplyTokens === null ? NO_LIMIT : String(saver.maxReplyTokens)}
              disabled={off}
              onChange={(event) =>
                change({
                  maxReplyTokens:
                    event.target.value === NO_LIMIT ? null : Number(event.target.value),
                })
              }
            >
              {REPLY_LIMITS.map((limit) => (
                <option key={limit ?? NO_LIMIT} value={limit === null ? NO_LIMIT : String(limit)}>
                  {limit === null ? t("noLimit") : t("tokens", { count: String(limit) })}
                </option>
              ))}
            </Select>
          </Field>
          <Field label={t("mode")} htmlFor="battery-mode" hint={t("modeHint")}>
            <Select
              id="battery-mode"
              value={saver.powerMode}
              disabled={off}
              onChange={(event) => change({ powerMode: event.target.value })}
            >
              {POWER_MODES.map((mode) => (
                <option key={mode} value={mode}>
                  {tg(`power_${mode}`)}
                </option>
              ))}
            </Select>
          </Field>
        </div>
        {save.isError && <p className="text-xs text-destructive">{save.error.message}</p>}
      </CardContent>
    </Card>
  );
}

function Toggle({
  id,
  label,
  hint,
  checked,
  disabled,
  onChange,
}: {
  id: string;
  label: string;
  hint: ReactNode;
  checked: boolean;
  disabled: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <label htmlFor={id} className="flex flex-col">
        <span className="text-sm">{label}</span>
        <span className="text-[11px] text-muted-foreground">{hint}</span>
      </label>
      <Switch id={id} checked={checked} disabled={disabled} onCheckedChange={onChange} />
    </div>
  );
}
