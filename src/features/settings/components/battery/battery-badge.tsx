import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Tooltip } from "@/components/ui/tooltip";
import { powerSourceQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { gatewaySettingsQuery } from "../system-cards";
import { messages } from "./messages";
import { batteryIcon } from "./power-status";
import { saverOf } from "./saver";

/** In the top bar while the battery saver shortens what models write. */
export function BatterySaverBadge() {
  const t = useT(messages);
  const { data: source } = useQuery(powerSourceQuery);
  const { data: settings } = useQuery(gatewaySettingsQuery);
  const saver = saverOf(settings?.batterySaver);
  if (!source?.onBattery || !saver.enabled) return null;
  const changes = [
    saver.noThinking && t("badgeThinking"),
    saver.maxReplyTokens !== null && t("badgeLimit", { count: String(saver.maxReplyTokens) }),
  ].filter(Boolean);
  const Icon = batteryIcon(source.batteryPercent);
  return (
    <Tooltip
      content={changes.length > 0 ? t("badgeHint", { changes: changes.join(", ") }) : t("badge")}
    >
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
