import { queryOptions } from "@tanstack/react-query";
import { commands, unwrap } from "@/lib/api";

export const runtimeQuery = queryOptions({
  queryKey: ["runtime"],
  queryFn: () => unwrap(commands.runtimeInfo),
  staleTime: 60_000,
});

export const chipsetQuery = queryOptions({
  queryKey: ["chipset"],
  queryFn: () => unwrap(commands.chipset),
  staleTime: Number.POSITIVE_INFINITY,
});

export const hardwareQuery = queryOptions({
  queryKey: ["hardware"],
  queryFn: () => unwrap(commands.hardwareInfo),
  staleTime: 30_000,
});
