import { queryOptions } from "@tanstack/react-query";
import { call, commands } from "@/lib/api";

export const appInfoQuery = queryOptions({
  queryKey: ["app-info"],
  queryFn: () => call(commands.appInfo),
  staleTime: Number.POSITIVE_INFINITY,
});
