import { useQuery } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import type { RequestEntry } from "@/lib/api";
import { requestsQuery } from "../api";

/** The latest API requests. Prompts and replies are never recorded. */
export function RequestsCard() {
  const { data: requests = [] } = useQuery(requestsQuery);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Recent requests</CardTitle>
        <CardDescription>
          Who called the API and how fast it answered. Prompts aren't recorded.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {requests.length === 0 ? (
          <p className="py-4 text-center text-sm text-muted-foreground">No requests yet.</p>
        ) : (
          <div className="max-h-80 overflow-auto overscroll-contain rounded-md border">
            <table className="w-full text-left text-xs">
              <thead className="sticky top-0 bg-card text-muted-foreground">
                <tr className="border-b">
                  <th className="px-3 py-2 font-medium">Time</th>
                  <th className="px-3 py-2 font-medium">App</th>
                  <th className="px-3 py-2 font-medium">Model</th>
                  <th className="px-3 py-2 font-medium">Status</th>
                  <th className="px-3 py-2 text-right font-medium">Duration</th>
                  <th className="px-3 py-2 text-right font-medium">Tokens</th>
                  <th className="px-3 py-2 text-right font-medium">Speed</th>
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
                      {new Date(request.startedAtMs).toLocaleTimeString()}
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
                        ? `${(request.durationMs / 1000).toFixed(1)}s`
                        : "…"}
                    </td>
                    <td className="px-3 py-2 text-right">{request.completionTokens ?? "—"}</td>
                    <td className="px-3 py-2 text-right">
                      {request.tokensPerSecond
                        ? `${request.tokensPerSecond.toFixed(0)} tok/s`
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
  if (request.status === null) return <Badge tone="info">Running</Badge>;
  if (request.status < 400 && !request.error) return <Badge tone="success">{request.status}</Badge>;
  if (request.status < 400) return <Badge tone="warning">Stopped</Badge>;
  return <Badge tone="danger">{request.status}</Badge>;
}
