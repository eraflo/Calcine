import type { GatewayStatus } from "@/lib/api";

export type StatusTone = "idle" | "ok" | "busy" | "error";

/** One-line summary of the gateway and GenieX, with a tone for the dot. */
export function describeStatus(status: GatewayStatus | undefined): {
  tone: StatusTone;
  label: string;
  detail: string;
} {
  if (!status) return { tone: "idle", label: "API", detail: "Checking the local API…" };
  if (!status.listening) {
    return {
      tone: "error",
      label: "API off",
      detail: status.error ?? "The local API isn't listening.",
    };
  }
  const queue = status.queuedRequests > 0 ? ` · ${status.queuedRequests} waiting` : "";
  switch (status.server.state) {
    case "ready":
      return status.activeRequests > 0
        ? { tone: "busy", label: "Generating", detail: `Answering a request${queue}` }
        : { tone: "ok", label: "API ready", detail: "GenieX is running and ready to answer." };
    case "starting":
      return { tone: "busy", label: "Starting", detail: `GenieX is starting${queue}` };
    case "failed":
      return { tone: "error", label: "GenieX failed", detail: status.server.message };
    case "stopped":
      return {
        tone: "idle",
        label: "API on",
        detail: "Listening. GenieX starts on the first request.",
      };
  }
}
