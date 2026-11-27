import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { call, commands, type UsagePeriod } from "@/lib/api";
import { utcOffsetMinutes } from "./lib/time";

/** Every usage summary: invalidated together when a request finishes. */
export const USAGE_KEY = ["usage"] as const;

export const usageQuery = (period: UsagePeriod) =>
  queryOptions({
    queryKey: [...USAGE_KEY, period],
    queryFn: () => call(() => commands.usageSummary(period, utcOffsetMinutes())),
    // Hours roll over even without requests.
    refetchInterval: 60_000,
  });

export function useClearUsage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => call(() => commands.clearUsage()),
    onSettled: () => queryClient.invalidateQueries({ queryKey: USAGE_KEY }),
  });
}
