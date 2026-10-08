import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands, type ModelKey, type ModelType, unwrap } from "@/lib/api";

export const modelsQuery = queryOptions({
  queryKey: ["models"],
  queryFn: () => unwrap(commands.listModels),
});

/** Delete models or single precisions, then refresh the library. */
export function useRemoveModels() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (keys: ModelKey[]) => unwrap(() => commands.removeModels(keys)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey }),
  });
}

/** Delete every downloaded model (`geniex clean`). */
export function useCleanModels() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => unwrap(commands.cleanModels),
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["models"] }),
  });
}

/** Correct whether a model is text-only or takes images (`geniex model set-type`). */
export function useSetModelType() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, modelType }: { name: string; modelType: ModelType }) =>
      unwrap(() => commands.setModelType(name, modelType)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey }),
  });
}

/** Open the system file picker for a model folder or AI Hub `.zip`. */
export const pickImportSource = (source: "folder" | "archive") => commands.pickImportSource(source);
