import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Tooltip } from "@/components/ui/tooltip";
import { powerSourceQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { generationMessages } from "@/i18n/generation";
import { gatewaySettingsQuery } from "../system-cards";
import { messages } from "./messages";
import { isPowerMode } from "./mode";
import { batteryIcon } from "./power-status";

/** In the top bar while the battery saver slows models down. */
export function BatterySaverBadge() {
  const t = useT(messages);
  const tg = useT(generationMessages);
  const { data: source } = useQuery(powerSourceQuery);
  const { data: settings } = useQuery(gatewaySettingsQuery);
  const mode = settings?.batteryPowerMode;
  if (!source?.onBattery || !isPowerMode(mode)) return null;
  const Icon = batteryIcon(source.batteryPercent);
  return (
    <Tooltip content={t("badgeHint", { mode: tg(`power_${mode}`) })}>
      <Link
        to="/settings"
        className="flex h-8 items-center gap-1.5 rounded-md px-2.5 text-xs text-success hover:bg-accent"
      >
        <Icon className="size-4" />
        {t("badge")}
      </Link>
    </Tooltip>
  );
}
