import { useQuery } from "@tanstack/react-query";
import { runtimeQuery } from "@/features/hardware/api";
import { useT } from "@/i18n";
import { messages } from "../messages";
import { Step } from "./step";

export function RuntimeStep() {
  const t = useT(messages);
  const runtime = useQuery(runtimeQuery);
  if (runtime.isPending) {
    return (
      <Step index={1} title={t("runtimeTitle")} status="loading">
        {t("runtimeLooking")}
      </Step>
    );
  }
  if (runtime.isError) {
    return (
      <Step index={1} title={t("runtimeTitle")} status="problem">
        {t("runtimeMissing")}
      </Step>
    );
  }
  return (
    <Step index={1} title={t("runtimeTitle")} status="done">
      {t("runtimeReady", {
        cli: runtime.data.cliVersion,
        qairt: runtime.data.qairtVersion ?? "—",
      })}
    </Step>
  );
}
