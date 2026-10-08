import { useQuery } from "@tanstack/react-query";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { runtimeQuery } from "../api";

/** GenieX CLI and bundled runtime versions. */
export function RuntimeCard() {
  const { data, error, isPending, refetch } = useQuery(runtimeQuery);

  if (error) return <ErrorState error={error} onRetry={() => refetch()} />;

  const rows: [string, string | null | undefined][] = [
    ["GenieX CLI", data?.cliVersion],
    ["QAIRT runtime", data?.qairtVersion],
    ["llama.cpp build", data?.llamaCppHash],
    ["Executable", data?.binaryPath],
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>GenieX runtime</CardTitle>
        <CardDescription>Versions reported by `geniex version`</CardDescription>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
          {rows.map(([label, value]) => (
            <div key={label} className="contents">
              <dt className="text-muted-foreground">{label}</dt>
              <dd className="truncate font-mono text-[13px]">
                {isPending ? <Skeleton className="h-4 w-32" /> : (value ?? "—")}
              </dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}
