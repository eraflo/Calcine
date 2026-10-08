import { useQuery } from "@tanstack/react-query";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { defineMessages, useT } from "@/i18n";
import { runtimeQuery } from "../api";

const strings = defineMessages({
  en: {
    title: "GenieX runtime",
    description: "Versions reported by geniex version",
    cli: "GenieX CLI",
    qairt: "QAIRT runtime",
    llamaCpp: "llama.cpp build",
    executable: "Executable",
  },
  fr: {
    title: "Runtime GenieX",
    description: "Versions indiquées par geniex version",
    cli: "CLI GenieX",
    qairt: "Runtime QAIRT",
    llamaCpp: "Build llama.cpp",
    executable: "Exécutable",
  },
});

/** GenieX CLI and bundled runtime versions. */
export function RuntimeCard() {
  const t = useT(strings);
  const { data, error, isPending, refetch } = useQuery(runtimeQuery);

  if (error) return <ErrorState error={error} onRetry={() => refetch()} />;

  const rows: [string, string | null | undefined][] = [
    [t("cli"), data?.cliVersion],
    [t("qairt"), data?.qairtVersion],
    [t("llamaCpp"), data?.llamaCppHash],
    [t("executable"), data?.binaryPath],
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
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
