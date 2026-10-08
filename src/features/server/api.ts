import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";
import {
  call,
  commands,
  events,
  type GatewayStatus,
  type NewApiKey,
  type RequestEntry,
  unwrap,
} from "@/lib/api";

export const gatewayQuery = queryOptions({
  queryKey: ["gateway"],
  queryFn: () => call(commands.gatewayStatus),
  // Kept fresh by `GatewayUpdated` events.
  staleTime: Number.POSITIVE_INFINITY,
});

export const requestsQuery = queryOptions({
  queryKey: ["requests"],
  queryFn: () => call(commands.listRequests),
  staleTime: Number.POSITIVE_INFINITY,
});

export const apiKeysQuery = queryOptions({
  queryKey: ["api-keys"],
  queryFn: () => call(commands.listApiKeys),
});

export const serverLogsQuery = queryOptions({
  queryKey: ["server-logs"],
  queryFn: () => call(commands.serverLogs),
  refetchInterval: 2_000,
});

/** How the app's own UI reaches the gateway (base URL + per-launch token). */
export const connectionQuery = queryOptions({
  queryKey: ["gateway-connection"],
  queryFn: () => call(commands.gatewayConnection),
  staleTime: Number.POSITIVE_INFINITY,
});

const KEPT_REQUESTS = 200;

/** Mirror gateway status and request-log events into the cache. Mount once. */
export function useGatewayEvents() {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    const stops: (() => void)[] = [];
    const keep = (stop: () => void) => (disposed ? stop() : stops.push(stop));

    events.gatewayUpdated
      .listen(({ payload }) =>
        queryClient.setQueryData<GatewayStatus>(gatewayQuery.queryKey, payload),
      )
      .then(keep);
    events.requestLogged
      .listen(({ payload }) =>
        queryClient.setQueryData<RequestEntry[]>(requestsQuery.queryKey, (entries = []) =>
          upsertRequest(entries, payload),
        ),
      )
      .then(keep);

    return () => {
      disposed = true;
      for (const stop of stops) stop();
    };
  }, [queryClient]);
}

export function upsertRequest(
  entries: readonly RequestEntry[],
  entry: RequestEntry,
): RequestEntry[] {
  const index = entries.findIndex((existing) => existing.id === entry.id);
  if (index === -1) return [entry, ...entries].slice(0, KEPT_REQUESTS);
  const next = [...entries];
  next[index] = entry;
  return next;
}

export function useStartServer() {
  return useMutation({ mutationFn: () => unwrap(commands.startServer) });
}

export function useStopServer() {
  return useMutation({ mutationFn: () => unwrap(commands.stopServer) });
}

export function useRequireApiKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (require: boolean) => unwrap(() => commands.setRequireApiKey(require)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: gatewayQuery.queryKey }),
  });
}

export function useCreateApiKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: NewApiKey) => unwrap(() => commands.createApiKey(request)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: apiKeysQuery.queryKey }),
  });
}

export function useRevokeApiKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => unwrap(() => commands.revokeApiKey(id)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: apiKeysQuery.queryKey }),
  });
}
