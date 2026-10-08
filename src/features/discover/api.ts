import { queryOptions } from "@tanstack/react-query";
import { commands, type ModelReference, unwrap } from "@/lib/api";

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

/** Hugging Face GGUF models matching `query` (most downloaded when empty). */
export const searchQuery = (query: string) =>
  queryOptions({
    queryKey: ["hub-search", query],
    queryFn: () => unwrap(() => commands.searchModels(query, 30)),
    staleTime: 5 * 60_000,
    retry: 1,
  });

/** Precisions and sizes of a remote model. */
export const detailsQuery = (reference: ModelReference) =>
  queryOptions({
    queryKey: ["hub-details", reference.hub, reference.name],
    queryFn: () => unwrap(() => commands.modelDetails(reference)),
    staleTime: 10 * 60_000,
    retry: false,
  });
