import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";
import { modelsQuery } from "@/features/library/api";
import {
  call,
  commands,
  events,
  type Job,
  type ModelType,
  type PullRequest,
  unwrap,
} from "@/lib/api";
import { useUi } from "@/stores/ui";
import { upsertJob } from "./lib/format";

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
        // A benchmark records its results even when some units fail.
        if (job.kind.type === "benchmark" && job.state.state !== "running") {
          void queryClient.invalidateQueries({ queryKey: ["bench-history"] });
        }
        if (job.state.state === "succeeded") {
          if (job.kind.type === "install_bench") {
            void queryClient.invalidateQueries({ queryKey: ["bench-tool"] });
          } else if (job.kind.type === "benchmark") {
            // Handled above.
          } else if (job.kind.type === "install_runtime") {
            for (const key of ["runtime", "runtime-update", "cached-runtimes", "models"]) {
              void queryClient.invalidateQueries({ queryKey: [key] });
            }
          } else {
            void queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey });
          }
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
      return startPull.mutateAsync({ reference, modelType: null, localPath: null });
    },
  };
}

export type ImportRequest = { name: string; path: string; modelType: ModelType | null };

/** Copy a model from this PC into the cache, and show the task drawer. */
export function useImportModel() {
  const setTasksOpen = useUi((state) => state.setTasksOpen);
  return useMutation({
    mutationFn: ({ name, path, modelType }: ImportRequest) =>
      unwrap(() => commands.importModel(name, path, modelType)),
    onSuccess: () => setTasksOpen(true),
  });
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
