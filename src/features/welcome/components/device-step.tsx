import { useQuery } from "@tanstack/react-query";
import { chipsetQuery, hardwareQuery } from "@/features/hardware/api";
import { Step } from "./step";

export function DeviceStep() {
  const chipset = useQuery(chipsetQuery);
  const hardware = useQuery(hardwareQuery);
  const npu = hardware.data?.accelerators.find((accelerator) => accelerator.unit === "npu");

  if (chipset.isPending || hardware.isPending) {
    return (
      <Step index={2} title="Your device" status="loading">
        Detecting your chipset and NPU…
      </Step>
    );
  }
  return (
    <Step index={2} title="Your device" status={npu ? "done" : "problem"}>
      {chipset.data ?? "Unknown chipset"}
      {npu ? ` · ${npu.name}` : " · no NPU detected, models will run on the GPU or CPU"}
    </Step>
  );
}
