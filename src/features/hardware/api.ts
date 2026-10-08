import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { call, commands, type InstallSource, type ReleaseChannel, unwrap } from "@/lib/api";
import { useUi } from "@/stores/ui";
import { recordUsage } from "./lib/usage-history";

export const runtimeQuery = queryOptions({
  queryKey: ["runtime"],
  queryFn: () => unwrap(commands.runtimeInfo),
  staleTime: 60_000,
});

export const chipsetQuery = queryOptions({
  queryKey: ["chipset"],
  queryFn: () => unwrap(commands.chipset),
  staleTime: Number.POSITIVE_INFINITY,
});

export const chipsetsQuery = queryOptions({
  queryKey: ["chipsets"],
  queryFn: () => unwrap(commands.listChipsets),
  // From Qualcomm AI Hub over the network; it changes with AI Hub releases.
  staleTime: 60 * 60_000,
  retry: 1,
});

export const hardwareQuery = queryOptions({
  queryKey: ["hardware"],
  queryFn: () => unwrap(commands.hardwareInfo),
  staleTime: 30_000,
});

/**
 * Live load, polled every second while a component shows it. Every sample
 * also goes to the history behind the sparklines; observers share one poll.
 */
export const usageQuery = queryOptions({
  queryKey: ["hardware-usage"],
  queryFn: async () => {
    const usage = await unwrap(commands.hardwareUsage);
    recordUsage(usage);
    return usage;
  },
  refetchInterval: 1_000,
  refetchIntervalInBackground: false,
  staleTime: 0,
  gcTime: 0,
  retry: false,
});

/** Pin the chipset (or `null` to detect it), then refresh what depends on it. */
export function useSetChipset() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (chipset: string | null) => unwrap(() => commands.setChipset(chipset)),
    onSettled: async () => {
      await queryClient.invalidateQueries({ queryKey: chipsetQuery.queryKey });
      await queryClient.invalidateQueries({ queryKey: ["catalog"] });
    },
  });
}

/** The newest GenieX on `channel`. Checked over the network, hourly. */
export const runtimeUpdateQuery = (channel: ReleaseChannel) =>
  queryOptions({
    queryKey: ["runtime-update", channel],
    queryFn: () => unwrap(() => commands.checkRuntimeUpdate(channel)),
    staleTime: 60 * 60_000,
    refetchInterval: 6 * 60 * 60_000,
    retry: 1,
  });

/** GenieX installers kept on this PC (for rolling back and repairing). */
export const cachedRuntimesQuery = queryOptions({
  queryKey: ["cached-runtimes"],
  queryFn: () => call(commands.cachedRuntimes),
});

/** Install, update or roll back GenieX; progress shows in the task drawer. */
export function useInstallRuntime() {
  const setTasksOpen = useUi((state) => state.setTasksOpen);
  return useMutation({
    mutationFn: (source: InstallSource) => unwrap(() => commands.installRuntime(source)),
    onSuccess: () => setTasksOpen(true),
  });
}
