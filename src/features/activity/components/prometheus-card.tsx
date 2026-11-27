import { useQuery } from "@tanstack/react-query";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { gatewayQuery } from "@/features/server/api";
import { useT } from "@/i18n";
import { messages } from "../messages";

/** Where Prometheus (and so Grafana) reads the same counters. */
export function PrometheusCard() {
  const t = useT(messages);
  const { data: gateway } = useQuery(gatewayQuery);
  const origin = (gateway?.baseUrl ?? "http://127.0.0.1:18181/v1").replace(/\/v1$/, "");
  const url = `${origin}/metrics`;
  const target = origin.replace(/^https?:\/\//, "");
  const config = [
    "scrape_configs:",
    "  - job_name: calcine",
    "    metrics_path: /metrics",
    "    authorization:",
    "      credentials: <API key>",
    "    static_configs:",
    `      - targets: ["${target}"]`,
  ].join("\n");

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("prometheus")}</CardTitle>
        <CardDescription>{t("prometheusHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <div className="flex items-center gap-2 rounded-md border bg-muted/40 px-3 py-1.5">
          <code className="min-w-0 flex-1 truncate text-xs">{url}</code>
          <CopyButton value={url} />
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-xs font-medium text-muted-foreground">{t("prometheusConfig")}</span>
          <div className="relative rounded-md border bg-muted/40">
            <pre className="overflow-x-auto p-3 text-[11px] leading-relaxed">{config}</pre>
            <CopyButton value={config} className="absolute top-1.5 right-1.5" />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
