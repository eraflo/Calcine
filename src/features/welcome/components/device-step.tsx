import { useQuery } from "@tanstack/react-query";
import { chipsetQuery, hardwareQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { messages } from "../messages";
import { Step } from "./step";

export function DeviceStep() {
  const t = useT(messages);
  const chipset = useQuery(chipsetQuery);
  const hardware = useQuery(hardwareQuery);
  const npu = hardware.data?.accelerators.find((accelerator) => accelerator.unit === "npu");

  if (chipset.isPending || hardware.isPending) {
    return (
      <Step index={2} title={t("deviceTitle")} status="loading">
        {t("deviceDetecting")}
      </Step>
    );
  }
  return (
    <Step index={2} title={t("deviceTitle")} status={npu ? "done" : "problem"}>
      {chipset.data ?? t("unknownChipset")}
      {npu ? ` · ${npu.name}` : ` · ${t("noNpu")}`}
    </Step>
  );
}
