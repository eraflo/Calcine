import { useQuery } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useLanguage, useT } from "@/i18n";
import type { RequestEntry } from "@/lib/api";
import { requestsQuery } from "../api";
import { messages } from "../messages";

/** The latest API requests. Prompts and replies are never recorded. */
export function RequestsCard() {
  const t = useT(messages);
  const language = useLanguage((state) => state.language);
  const { data: requests = [] } = useQuery(requestsQuery);

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("requestsTitle")}</CardTitle>
        <CardDescription>{t("requestsDescription")}</CardDescription>
      </CardHeader>
      <CardContent>
        {requests.length === 0 ? (
          <p className="py-4 text-center text-sm text-muted-foreground">{t("noRequests")}</p>
        ) : (
          <div className="max-h-80 overflow-auto overscroll-contain rounded-md border">
            <table className="w-full text-left text-xs">
              <thead className="sticky top-0 bg-card text-muted-foreground">
                <tr className="border-b">
                  <th className="px-3 py-2 font-medium">{t("columnTime")}</th>
                  <th className="px-3 py-2 font-medium">{t("columnApp")}</th>
                  <th className="px-3 py-2 font-medium">{t("columnModel")}</th>
                  <th className="px-3 py-2 font-medium">{t("columnStatus")}</th>
                  <th className="px-3 py-2 text-right font-medium">{t("columnDuration")}</th>
                  <th className="px-3 py-2 text-right font-medium">{t("columnTokens")}</th>
                  <th className="px-3 py-2 text-right font-medium">{t("columnSpeed")}</th>
                </tr>
              </thead>
              <tbody className="tabular-nums">
                {requests.map((request) => (
                  <tr
                    key={request.id}
                    className="border-b last:border-0"
                    title={request.error ?? undefined}
                  >
                    <td className="px-3 py-2 text-muted-foreground">
                      {new Date(request.startedAtMs).toLocaleTimeString(language)}
                    </td>
                    <td className="px-3 py-2">{request.client}</td>
                    <td className="max-w-48 truncate px-3 py-2 font-mono">
                      {request.model ?? request.path}
                    </td>
                    <td className="px-3 py-2">
                      <StatusBadge request={request} />
                    </td>
                    <td className="px-3 py-2 text-right">
                      {request.durationMs !== null
                        ? t("seconds", {
                            value: (request.durationMs / 1000).toLocaleString(language, {
                              minimumFractionDigits: 1,
                              maximumFractionDigits: 1,
                            }),
                          })
                        : "…"}
                    </td>
                    <td className="px-3 py-2 text-right">{request.completionTokens ?? "—"}</td>
                    <td className="px-3 py-2 text-right">
                      {request.tokensPerSecond
                        ? t("tokensPerSecond", { value: Math.round(request.tokensPerSecond) })
                        : "—"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function StatusBadge({ request }: { request: RequestEntry }) {
  const t = useT(messages);
  if (request.status === null) return <Badge tone="info">{t("requestRunning")}</Badge>;
  if (request.status < 400 && !request.error) return <Badge tone="success">{request.status}</Badge>;
  if (request.status < 400) return <Badge tone="warning">{t("requestStopped")}</Badge>;
  return <Badge tone="danger">{request.status}</Badge>;
}
