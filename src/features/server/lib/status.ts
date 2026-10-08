import type { Translate } from "@/i18n";
import type { GatewayStatus } from "@/lib/api";
import type { messages } from "../messages";

type ServerT = Translate<(typeof messages)["en"]>;

export type StatusTone = "idle" | "ok" | "busy" | "error";

/** One-line summary of the gateway and GenieX, with a tone for the dot. */
export function describeStatus(
  status: GatewayStatus | undefined,
  t: ServerT,
): {
  tone: StatusTone;
  label: string;
  detail: string;
} {
  if (!status) return { tone: "idle", label: t("statusApi"), detail: t("statusChecking") };
  if (!status.listening) {
    return {
      tone: "error",
      label: t("statusOff"),
      detail: status.error ?? t("statusNotListening"),
    };
  }
  const queue =
    status.queuedRequests > 0 ? ` · ${t("statusWaiting", { count: status.queuedRequests })}` : "";
  switch (status.server.state) {
    case "ready":
      return status.activeRequests > 0
        ? { tone: "busy", label: t("statusGenerating"), detail: t("statusAnswering", { queue }) }
        : { tone: "ok", label: t("statusReady"), detail: t("statusReadyDetail") };
    case "starting":
      return {
        tone: "busy",
        label: t("statusStarting"),
        detail: t("statusStartingDetail", { queue }),
      };
    case "failed":
      return { tone: "error", label: t("statusFailed"), detail: status.server.message };
    case "stopped":
      return { tone: "idle", label: t("statusOn"), detail: t("statusStoppedDetail") };
  }
}
