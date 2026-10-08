import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";
import { modelsQuery } from "@/features/library/api";
import { call, commands, events, type Job, type PullRequest, unwrap } from "@/lib/api";
import { useUi } from "@/stores/ui";
import { upsertJob } from "./format";

export const jobsQuery = queryOptions({
  queryKey: ["jobs"],
  queryFn: () => call(commands.listJobs),
  // Kept fresh by `JobUpdated` events instead of polling.
  staleTime: Number.POSITIVE_INFINITY,
});

/**
 * Mirror backend job events into the query cache, and refresh the library
 * when a download finishes. Mount once, in the app shell.
 */
export function useJobEvents() {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;

    events.jobUpdated
      .listen(({ payload: job }) => {
        queryClient.setQueryData<Job[]>(jobsQuery.queryKey, (jobs = []) => upsertJob(jobs, job));
        if (job.kind.type === "pull" && job.state.state === "succeeded") {
          void queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey });
        }
      })
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [queryClient]);
}

/** Start a download and open the task drawer to show it. */
export function useStartPull() {
  const setTasksOpen = useUi((state) => state.setTasksOpen);
  return useMutation({
    mutationFn: (request: PullRequest) => unwrap(() => commands.pullModel(request)),
    onSuccess: () => setTasksOpen(true),
  });
}

/** Download a model by name, letting GenieX pick the hub and precision. */
export function usePullByName() {
  const startPull = useStartPull();
  return {
    ...startPull,
    pull: async (name: string) => {
      const reference = await unwrap(() => commands.parseModelReference(name));
      return startPull.mutateAsync({ reference, modelType: null });
    },
  };
}

export function useCancelJob() {
  return useMutation({ mutationFn: (id: number) => call(() => commands.cancelJob(id)) });
}

export function useDismissJob() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => call(() => commands.dismissJob(id)),
    onSuccess: (_, id) =>
      queryClient.setQueryData<Job[]>(jobsQuery.queryKey, (jobs = []) =>
        jobs.filter((job) => job.id !== id),
      ),
  });
}
