import { queryOptions } from "@tanstack/react-query";
import { commands, type ModelHub, unwrap } from "@/lib/api";

/** Qualcomm AI Hub catalog, for this device or (`allChipsets`) every device. */
export const catalogQuery = (allChipsets: boolean) =>
  queryOptions({
    queryKey: ["catalog", "aihub", allChipsets],
    queryFn: () => unwrap(() => commands.aihubCatalog(allChipsets)),
    // Fetched from Qualcomm AI Hub over the network; it changes rarely.
    staleTime: 10 * 60_000,
  });

export const referenceQuery = (input: string) =>
  queryOptions({
    queryKey: ["reference", input],
    queryFn: () => unwrap(() => commands.parseModelReference(input)),
    enabled: input.length > 0,
    retry: false,
    staleTime: Number.POSITIVE_INFINITY,
  });

export const HUB_LABELS: Record<ModelHub, string> = {
  auto: "Detected by GenieX",
  ai_hub: "Qualcomm AI Hub",
  hugging_face: "Hugging Face",
  model_scope: "ModelScope",
  docker_hub: "Docker Hub",
};
