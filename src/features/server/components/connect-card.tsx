import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { modelsQuery } from "@/features/library/api";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { gatewayQuery } from "../api";
import { KEY_VARIABLE, SNIPPET_LANGUAGES, type SnippetLanguage, snippet } from "../lib/snippets";
import { messages } from "../messages";

/** Ready-to-paste code for other apps, using a model from the library. */
export function ConnectCard() {
  const t = useT(messages);
  const [language, setLanguage] = useState<SnippetLanguage>("python");
  const { data: status } = useQuery(gatewayQuery);
  const { data: models = [] } = useQuery(modelsQuery);
  const baseUrl = status?.baseUrl ?? "http://127.0.0.1:18181/v1";
  const model = models[0]?.name ?? "qualcomm/Qwen3-0.6B";
  const code = snippet(language, baseUrl, model);

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("connectTitle")}</CardTitle>
        <CardDescription>
          {t.rich(
            "connectHint",
            { code: (text) => <code className="font-mono">{text}</code> },
            { variable: KEY_VARIABLE },
          )}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        <div role="tablist" aria-label={t("snippetLanguage")} className="flex gap-1">
          {SNIPPET_LANGUAGES.map(({ id, label }) => (
            <button
              key={id}
              type="button"
              role="tab"
              aria-selected={language === id}
              onClick={() => setLanguage(id)}
              className={cn(
                "h-7 rounded-md px-2.5 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground",
                language === id && "bg-accent text-foreground",
              )}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="relative">
          <pre className="overflow-x-auto rounded-md border bg-background p-3 pr-10 font-mono text-[12px] leading-relaxed">
            {code}
          </pre>
          <CopyButton value={code} className="absolute top-1.5 right-1.5" />
        </div>
      </CardContent>
    </Card>
  );
}
