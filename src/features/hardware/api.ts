import { queryOptions } from "@tanstack/react-query";
import { commands, unwrap } from "@/lib/api";

export const runtimeQuery = queryOptions({
  queryKey: ["runtime"],
  queryFn: () => unwrap(commands.runtimeInfo),
  staleTime: 60_000,
});
