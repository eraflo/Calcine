import { useQuery } from "@tanstack/react-query";
import { Microchip } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select } from "@/components/ui/field";
import { useT } from "@/i18n";
import type { Chipset } from "@/lib/api";
import { chipsetQuery, chipsetsQuery, useSetChipset } from "../api";
import { messages } from "../messages";

const AUTO = "auto";

/**
 * GenieX reports the reference device when it detects the chipset
 * (`Snapdragon X Elite CRD`) and the saved id when it was set
 * (`qualcomm-snapdragon-x-elite`).
 */
export function findChipset(chipsets: readonly Chipset[], value: string | null | undefined) {
  if (!value) return { chipset: undefined, pinned: false };
  const wanted = value.toLowerCase();
  const byId = chipsets.find(
    (chipset) =>
      chipset.id.toLowerCase() === wanted ||
      chipset.aliases.some((alias) => alias.toLowerCase() === wanted),
  );
  if (byId) return { chipset: byId, pinned: true };
  const byDevice = chipsets.find((chipset) => chipset.device.toLowerCase() === wanted);
  return { chipset: byDevice, pinned: false };
}

/** Which chipset GenieX downloads AI Hub models for (`geniex config set chipset`). */
export function ChipsetCard() {
  const t = useT(messages);
  const current = useQuery(chipsetQuery);
  const chipsets = useQuery(chipsetsQuery);
  const setChipset = useSetChipset();

  const { chipset, pinned } = findChipset(chipsets.data ?? [], current.data);
  const label = chipset?.marketingName ?? chipset?.device ?? current.data ?? "—";

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Microchip className="size-4 text-muted-foreground" />
          <CardTitle>{t("chipset")}</CardTitle>
          {current.data && (
            <Badge tone={pinned ? "warning" : "success"} className="ml-1">
              {pinned ? t("chipsetPinned") : t("chipsetDetected")}
            </Badge>
          )}
        </div>
        <CardDescription>{t("chipsetHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        <div className="flex flex-wrap items-center gap-3">
          <span className="text-sm font-medium">{label}</span>
          <Select
            aria-label={t("chipset")}
            className="ml-auto w-72"
            value={pinned && chipset ? chipset.id : AUTO}
            disabled={!chipsets.data || setChipset.isPending}
            onChange={(event) =>
              setChipset.mutate(event.target.value === AUTO ? null : event.target.value)
            }
          >
            <option value={AUTO}>{t("chipsetAuto")}</option>
            {chipsets.data?.map((option) => (
              <option key={option.id} value={option.id}>
                {option.marketingName ?? option.device} — {option.device}
              </option>
            ))}
          </Select>
        </div>
        {chipsets.isError && (
          <p className="text-xs text-destructive">
            {t("chipsetListError", { message: chipsets.error.message })}
          </p>
        )}
        {setChipset.isError && (
          <p className="text-xs text-destructive">{setChipset.error.message}</p>
        )}
      </CardContent>
    </Card>
  );
}
