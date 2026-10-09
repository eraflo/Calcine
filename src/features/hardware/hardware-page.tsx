import { useQuery } from "@tanstack/react-query";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Page } from "@/components/calcine/layout/page";
import { useT } from "@/i18n";
import { chipsetQuery, chipsetsQuery, hardwareQuery } from "./api";
import { ChipsetCard, findChipset } from "./components/chipset-card";
import { ComputeUnits } from "./components/compute-units";
import { PowerCard } from "./components/power-card";
import { RuntimeCard } from "./components/runtime-card";
import { SelfTestCard } from "./components/self-test-card";
import { StorageCard } from "./components/storage-card";
import { messages } from "./messages";

export function HardwarePage() {
  const t = useT(messages);
  const chipset = useQuery(chipsetQuery);
  const chipsets = useQuery(chipsetsQuery);
  const hardware = useQuery(hardwareQuery);
  const known = findChipset(chipsets.data ?? [], chipset.data).chipset;

  return (
    <Page
      title={t("title")}
      description={
        known?.marketingName ??
        chipset.data ??
        (chipset.isPending ? t("detecting") : t("yourDevice"))
      }
    >
      {hardware.isError ? (
        <ErrorState error={hardware.error} onRetry={() => hardware.refetch()} />
      ) : (
        <>
          <ComputeUnits info={hardware.data} />
          <PowerCard />
          <StorageCard info={hardware.data} />
        </>
      )}
      <SelfTestCard />
      <ChipsetCard />
      <RuntimeCard />
    </Page>
  );
}
