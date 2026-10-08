import { queryOptions } from "@tanstack/react-query";
import { commands, unwrap } from "@/lib/api";

export const modelsQuery = queryOptions({
  queryKey: ["models"],
  queryFn: () => unwrap(commands.listModels),
});
