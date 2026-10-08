import { queryOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands, type ModelKey, unwrap } from "@/lib/api";

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
