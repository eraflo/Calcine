import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowRight, Plus, X } from "lucide-react";
import { type FormEvent, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input, Select } from "@/components/ui/field";
import { modelsQuery } from "@/features/library/api";
import { useT } from "@/i18n";
import { commands, type ModelAlias, unwrap } from "@/lib/api";
import { gatewaySettingsQuery } from "../system-cards";
import { messages } from "./messages";

/** Names apps often have built in, offered when none is used yet. */
const COMMON_NAMES = ["gpt-4o-mini", "gpt-4o", "gpt-3.5-turbo", "llama3.2"];

/** Model names apps ask for, answered by models on this PC. */
export function AliasesCard() {
  const t = useT(messages);
  const queryClient = useQueryClient();
  const settings = useQuery(gatewaySettingsQuery);
  const { data: models = [] } = useQuery(modelsQuery);
  const installed = [...new Set(models.map((model) => model.name))];
  const aliases = settings.data?.modelAliases ?? [];
  const [alias, setAlias] = useState("");
  const [model, setModel] = useState("");

  const save = useMutation({
    mutationFn: (next: ModelAlias[]) => unwrap(() => commands.setModelAliases(next)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: gatewaySettingsQuery.queryKey }),
  });
  const add = (event: FormEvent) => {
    event.preventDefault();
    const target = model || installed[0];
    if (!alias.trim() || !target) return;
    save.mutate([...aliases, { alias: alias.trim(), model: target }], {
      onSuccess: () => setAlias(""),
    });
  };
  const used = new Set(aliases.map((entry) => entry.alias.toLowerCase()));
  const suggestions = COMMON_NAMES.filter((name) => !used.has(name));

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {aliases.length === 0 ? (
          <p className="text-xs text-muted-foreground">{t("empty")}</p>
        ) : (
          <ul className="flex flex-col divide-y rounded-md border">
            {aliases.map((entry) => (
              <li key={entry.alias} className="flex items-center gap-3 px-3 py-2 text-sm">
                <span className="font-mono">{entry.alias}</span>
                <ArrowRight
                  className="size-3.5 shrink-0 text-muted-foreground"
                  aria-label={t("answeredBy")}
                />
                <span className="min-w-0 truncate font-mono text-muted-foreground">
                  {entry.model}
                </span>
                {!installed.includes(entry.model) && <Badge tone="warning">{t("missing")}</Badge>}
                <Button
                  variant="ghost"
                  size="icon"
                  className="ml-auto size-7 text-muted-foreground"
                  disabled={save.isPending}
                  onClick={() => save.mutate(aliases.filter((other) => other !== entry))}
                  aria-label={t("remove", { alias: entry.alias })}
                >
                  <X />
                </Button>
              </li>
            ))}
          </ul>
        )}

        <form onSubmit={add} className="flex flex-col gap-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <Input
              value={alias}
              onChange={(event) => setAlias(event.target.value)}
              placeholder={t("aliasPlaceholder")}
              aria-label={t("alias")}
              spellCheck={false}
              className="w-48 font-mono"
            />
            <ArrowRight className="size-3.5 text-muted-foreground" aria-hidden />
            <Select
              value={model || installed[0] || ""}
              onChange={(event) => setModel(event.target.value)}
              aria-label={t("model")}
              disabled={installed.length === 0}
              className="w-64 min-w-0"
            >
              {installed.length === 0 && <option value="">{t("noModels")}</option>}
              {installed.map((name) => (
                <option key={name} value={name}>
                  {name}
                </option>
              ))}
            </Select>
            <Button
              type="submit"
              size="sm"
              variant="secondary"
              disabled={!alias.trim() || installed.length === 0 || save.isPending}
            >
              <Plus />
              {t("add")}
            </Button>
          </div>
          {suggestions.length > 0 && (
            <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
              {t("suggestions")}
              {suggestions.map((name) => (
                <button
                  key={name}
                  type="button"
                  onClick={() => setAlias(name)}
                  className="rounded-sm border px-1.5 py-0.5 font-mono hover:bg-accent hover:text-foreground"
                >
                  {name}
                </button>
              ))}
            </div>
          )}
          {save.isError && <p className="text-xs text-destructive">{save.error.message}</p>}
        </form>
      </CardContent>
    </Card>
  );
}
