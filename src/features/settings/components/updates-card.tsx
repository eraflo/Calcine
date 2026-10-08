import { useQuery } from "@tanstack/react-query";
import { ArrowUpCircle, CircleCheck, RefreshCw } from "lucide-react";
import Markdown from "react-markdown";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Switch } from "@/components/ui/switch";
import { defineMessages, useT } from "@/i18n";
import { CalcineError } from "@/lib/api";
import { formatRelative } from "@/lib/format";
import { cn } from "@/lib/utils";
import { useUpdates } from "@/stores/updates";
import { appInfoQuery, appUpdateQuery, useInstallAppUpdate } from "../api";

const strings = defineMessages({
  en: {
    title: "Calcine updates",
    description: "Updates are signed and verified before installing.",
    channel: "Channel",
    stable: "Stable",
    beta: "Beta",
    channelHint: "Beta builds come from the development branch: newer, less tested.",
    current: "Calcine {version}",
    checking: "Checking for updates…",
    upToDate: "Calcine is up to date.",
    available: "Calcine {version} is available",
    released: "released {when}",
    install: "Install and restart",
    installHint: "Calcine closes, updates and reopens. Running downloads stop.",
    checkNow: "Check now",
    autoCheck: "Check for updates automatically",
    autoCheckHint:
      "When Calcine starts, ask GitHub for Calcine updates and Qualcomm for GenieX updates. Off: only when you click Check now.",
    checksOff: "Automatic checks are off.",
    checkFailed: "Couldn't check for updates: {message}",
    checkUnavailable:
      "No update found: GitHub is unreachable, or nothing is published on this channel yet.",
  },
  fr: {
    title: "Mises à jour de Calcine",
    description: "Les mises à jour sont signées et vérifiées avant l'installation.",
    channel: "Canal",
    stable: "Stable",
    beta: "Bêta",
    channelHint:
      "Les versions bêta viennent de la branche de développement : plus récentes, moins testées.",
    current: "Calcine {version}",
    checking: "Recherche de mises à jour…",
    upToDate: "Calcine est à jour.",
    available: "Calcine {version} est disponible",
    released: "publiée {when}",
    install: "Installer et redémarrer",
    installHint:
      "Calcine se ferme, se met à jour et se relance. Les téléchargements en cours s'arrêtent.",
    checkNow: "Vérifier maintenant",
    autoCheck: "Vérifier les mises à jour automatiquement",
    autoCheckHint:
      "Au démarrage, interroger GitHub pour Calcine et Qualcomm pour GenieX. Désactivé : seulement quand vous cliquez sur Vérifier maintenant.",
    checksOff: "La vérification automatique est désactivée.",
    checkFailed: "Impossible de vérifier les mises à jour : {message}",
    checkUnavailable:
      "Aucune mise à jour trouvée : GitHub est injoignable, ou rien n'est encore publié sur ce canal.",
  },
});

/** Calcine's own updates: channel, check, install. */
export function UpdatesCard() {
  const t = useT(strings);
  const channel = useUpdates((state) => state.appChannel);
  const setChannel = useUpdates((state) => state.setAppChannel);
  const autoCheck = useUpdates((state) => state.autoCheck);
  const setAutoCheck = useUpdates((state) => state.setAutoCheck);
  const { data: info } = useQuery(appInfoQuery);
  const check = useQuery({ ...appUpdateQuery(channel), enabled: autoCheck });
  const install = useInstallAppUpdate();
  const update = check.data;

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="flex items-center justify-between gap-4">
          <div className="flex flex-col">
            <span className="text-sm">{t("channel")}</span>
            <span className="text-[11px] text-muted-foreground">{t("channelHint")}</span>
          </div>
          <SegmentedControl
            name="app-channel"
            label={t("channel")}
            value={channel}
            onChange={setChannel}
            options={[
              { value: "stable", label: t("stable") },
              { value: "beta", label: t("beta") },
            ]}
          />
        </div>

        <div className="flex items-center justify-between gap-4">
          <label htmlFor="auto-check" className="flex flex-col">
            <span className="text-sm">{t("autoCheck")}</span>
            <span className="text-[11px] text-muted-foreground">{t("autoCheckHint")}</span>
          </label>
          <Switch id="auto-check" checked={autoCheck} onCheckedChange={setAutoCheck} />
        </div>

        <div className="flex items-center gap-3 border-t pt-4">
          {check.isPending && !check.isFetching && !autoCheck ? (
            <p className="text-sm text-muted-foreground">{t("checksOff")}</p>
          ) : check.isPending ? (
            <p className="text-sm text-muted-foreground">{t("checking")}</p>
          ) : check.isError ? (
            <p className="text-sm text-muted-foreground">
              {check.error instanceof CalcineError && check.error.kind === "network"
                ? t("checkUnavailable")
                : t("checkFailed", { message: check.error.message })}
            </p>
          ) : update ? null : (
            <p className="flex items-center gap-2 text-sm text-muted-foreground">
              <CircleCheck className="size-4 text-success" />
              {t("upToDate")}
              {info && <span className="font-mono text-xs">{info.version}</span>}
            </p>
          )}
          <Button
            size="sm"
            variant="ghost"
            className="ml-auto"
            onClick={() => check.refetch()}
            disabled={check.isFetching}
          >
            <RefreshCw className={cn(check.isFetching && "animate-spin")} />
            {t("checkNow")}
          </Button>
        </div>

        {update && (
          <div className="flex flex-col gap-2 rounded-md border border-primary/40 bg-primary/5 p-3">
            <div className="flex flex-wrap items-center gap-2">
              <ArrowUpCircle className="size-4 text-primary" />
              <span className="text-sm font-medium">
                {t("available", { version: update.version })}
              </span>
              {update.version.includes("-") && <Badge tone="warning">{t("beta")}</Badge>}
              {update.date && (
                <span className="text-xs text-muted-foreground">
                  {t("released", { when: formatRelative(Date.parse(update.date)) })}
                </span>
              )}
            </div>
            {update.notes && (
              <div className="markdown max-h-40 overflow-y-auto text-xs text-muted-foreground">
                <Markdown>{update.notes}</Markdown>
              </div>
            )}
            <div className="flex items-center gap-3">
              <p className="min-w-0 flex-1 text-xs text-muted-foreground">{t("installHint")}</p>
              <Button
                size="sm"
                onClick={() => install.mutate(channel)}
                disabled={install.isPending}
              >
                {t("install")}
              </Button>
            </div>
            {install.isError && <p className="text-xs text-destructive">{install.error.message}</p>}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
