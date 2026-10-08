import { queryOptions, useMutation } from "@tanstack/react-query";
import { type AppChannel, call, commands, unwrap } from "@/lib/api";
import { useUi } from "@/stores/ui";

export const appInfoQuery = queryOptions({
  queryKey: ["app-info"],
  queryFn: () => call(commands.appInfo),
  staleTime: Number.POSITIVE_INFINITY,
});

/** A newer Calcine on `channel` (`null` when up to date). Checked every few hours. */
export const appUpdateQuery = (channel: AppChannel) =>
  queryOptions({
    queryKey: ["app-update", channel],
    queryFn: () => unwrap(() => commands.checkAppUpdate(channel)),
    staleTime: 60 * 60_000,
    refetchInterval: 6 * 60 * 60_000,
    retry: 1,
  });

/** Download and install the update; Calcine restarts when it's done. */
export function useInstallAppUpdate() {
  const setTasksOpen = useUi((state) => state.setTasksOpen);
  return useMutation({
    mutationFn: (channel: AppChannel) => unwrap(() => commands.installAppUpdate(channel)),
    onSuccess: () => setTasksOpen(true),
  });
}
