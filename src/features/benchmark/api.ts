import { queryOptions, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { jobsQuery } from "@/features/tasks/api";
import { type BenchRequest, call, commands, type Job, unwrap } from "@/lib/api";
import { useUi } from "@/stores/ui";

export const benchToolQuery = queryOptions({
  queryKey: ["bench-tool"],
  queryFn: () => unwrap(commands.benchTool),
});

export const benchHistoryQuery = queryOptions({
  queryKey: ["bench-history"],
  queryFn: () => call(commands.benchHistory),
});

/** The running benchmark or tool download, if any. */
export function useRunningJob(type: "benchmark" | "install_bench"): Job | undefined {
  const { data: jobs = [] } = useQuery(jobsQuery);
  return jobs.find((job) => job.kind.type === type && job.state.state === "running");
}

export function useInstallBenchTool() {
  const openTasks = useUi((state) => state.setTasksOpen);
  return useMutation({
    mutationFn: () => unwrap(commands.installBenchTool),
    onSuccess: () => openTasks(true),
  });
}

export function useStartBenchmark() {
  return useMutation({
    mutationFn: (request: BenchRequest) => unwrap(() => commands.startBenchmark(request)),
  });
}

export function useForgetResults() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (ids: string[]) => unwrap(() => commands.forgetBenchResults(ids)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: benchHistoryQuery.queryKey }),
  });
}
