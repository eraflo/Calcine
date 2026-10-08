import { useQuery } from "@tanstack/react-query";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Page } from "@/components/calcine/layout/page";
import { chipsetQuery, hardwareQuery } from "./api";
import { ComputeUnits } from "./components/compute-units";
import { RuntimeCard } from "./components/runtime-card";
import { StorageCard } from "./components/storage-card";

export function HardwarePage() {
  const chipset = useQuery(chipsetQuery);
  const hardware = useQuery(hardwareQuery);

  return (
    <Page
      title="Hardware"
      description={chipset.data ?? (chipset.isPending ? "Detecting your device…" : "Your device")}
    >
      {hardware.isError ? (
        <ErrorState error={hardware.error} onRetry={() => hardware.refetch()} />
      ) : (
        <>
          <ComputeUnits info={hardware.data} />
          <StorageCard info={hardware.data} />
        </>
      )}
      <RuntimeCard />
    </Page>
  );
}
