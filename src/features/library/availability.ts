import { useQuery } from "@tanstack/react-query";
import { useCallback } from "react";
import { jobsQuery } from "@/features/tasks/api";
import type { Job } from "@/lib/api";
import { modelsQuery } from "./api";

export type Availability =
  | { status: "installed" }
  | { status: "downloading"; job: Job }
  | { status: "available" };

/** Whether a model is in the library, downloading, or ready to download. */
export function useAvailability() {
  const { data: models = [] } = useQuery(modelsQuery);
  const { data: jobs = [] } = useQuery(jobsQuery);

  return useCallback(
    (name: string): Availability => {
      const job = jobs.find(
        (candidate) =>
          candidate.state.state === "running" &&
          candidate.kind.type === "pull" &&
          (candidate.kind.model === name || candidate.kind.model.startsWith(`${name}:`)),
      );
      if (job) return { status: "downloading", job };
      if (models.some((model) => model.name === name)) return { status: "installed" };
      return { status: "available" };
    },
    [models, jobs],
  );
}
