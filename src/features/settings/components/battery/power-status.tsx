import {
  Battery,
  BatteryFull,
  BatteryLow,
  BatteryMedium,
  type LucideIcon,
  PlugZap,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { useT } from "@/i18n";
import type { PowerSource } from "@/lib/api";
import { messages } from "./messages";

/** The battery's icon for its charge. */
export function batteryIcon(percent: number | null): LucideIcon {
  if (percent === null) return Battery;
  if (percent < 25) return BatteryLow;
  if (percent < 70) return BatteryMedium;
  return BatteryFull;
}

/** Mains or battery, with the charge, and whether power is being saved. */
export function PowerStatus({ source, saving }: { source: PowerSource; saving: boolean }) {
  const t = useT(messages);
  const Icon = source.onBattery ? batteryIcon(source.batteryPercent) : PlugZap;
  let label = t("onMains");
  if (source.onBattery) {
    label =
      source.batteryPercent === null
        ? t("onBatteryUnknown")
        : t("onBattery", { percent: source.batteryPercent });
  }
  return (
    <div className="flex items-center gap-2 rounded-md border bg-muted/30 px-3 py-2 text-sm">
      <Icon className="size-4 text-muted-foreground" />
      <span>{label}</span>
      {source.onBattery && saving && (
        <Badge tone="success" className="ml-auto">
          {t("active")}
        </Badge>
      )}
    </div>
  );
}
