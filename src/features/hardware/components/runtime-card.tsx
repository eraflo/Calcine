import { useQuery } from "@tanstack/react-query";
import { ArrowUpCircle, CircleCheck, ExternalLink, RotateCcw, ShieldAlert } from "lucide-react";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Skeleton } from "@/components/ui/skeleton";
import { jobsQuery } from "@/features/tasks/api";
import { defineMessages, useT } from "@/i18n";
import { call, commands, type RuntimeRelease } from "@/lib/api";
import { formatBytes, formatRelative } from "@/lib/format";
import { useUpdates } from "@/stores/updates";
import { cachedRuntimesQuery, runtimeQuery, runtimeUpdateQuery, useInstallRuntime } from "../api";

const strings = defineMessages({
  en: {
    title: "GenieX runtime",
    description: "Versions reported by geniex version",
    cli: "GenieX CLI",
    qairt: "QAIRT runtime",
    llamaCpp: "llama.cpp build",
    executable: "Executable",
    updates: "Updates",
    channel: "Channel",
    stable: "Stable",
    prerelease: "Pre-releases",
    prereleaseBadge: "Pre-release",
    checking: "Checking for updates…",
    upToDate: "GenieX is up to date.",
    available: "GenieX {version} is available",
    released: "released {when}",
    update: "Update to {version}",
    install: "Install GenieX {version}",
    notes: "Release notes",
    unsigned:
      "Qualcomm hasn't marked this release as signed. Its checksum is still verified before installing.",
    updateHint:
      "Calcine stops GenieX, installs the new version and starts it again. Apps using the API are paused for a minute.",
    checkFailed: "Couldn't check for updates: {message}",
    installing: "Installing GenieX {version}…",
    otherVersions: "Other versions on this PC",
    bundled: "bundled",
    reinstall: "Reinstall",
    rollBack: "Roll back",
  },
  fr: {
    title: "Runtime GenieX",
    description: "Versions indiquées par geniex version",
    cli: "CLI GenieX",
    qairt: "Runtime QAIRT",
    llamaCpp: "Build llama.cpp",
    executable: "Exécutable",
    updates: "Mises à jour",
    channel: "Canal",
    stable: "Stable",
    prerelease: "Préversions",
    prereleaseBadge: "Préversion",
    checking: "Recherche de mises à jour…",
    upToDate: "GenieX est à jour.",
    available: "GenieX {version} est disponible",
    released: "publiée {when}",
    update: "Mettre à jour vers {version}",
    install: "Installer GenieX {version}",
    notes: "Notes de version",
    unsigned:
      "Qualcomm n'a pas marqué cette version comme signée. Sa somme de contrôle est tout de même vérifiée avant l'installation.",
    updateHint:
      "Calcine arrête GenieX, installe la nouvelle version puis le relance. Les applis qui utilisent l'API sont en pause une minute.",
    checkFailed: "Impossible de vérifier les mises à jour : {message}",
    installing: "Installation de GenieX {version}…",
    otherVersions: "Autres versions sur ce PC",
    bundled: "fournie avec Calcine",
    reinstall: "Réinstaller",
    rollBack: "Revenir à cette version",
  },
});

/** GenieX versions, updates (stable or pre-release) and roll back. */
export function RuntimeCard() {
  const t = useT(strings);
  const { data, error, isPending, refetch } = useQuery(runtimeQuery);

  if (error) {
    return (
      <div className="flex flex-col gap-3">
        <ErrorState error={error} onRetry={() => refetch()} />
        <UpdateSection />
      </div>
    );
  }

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
      <CardContent className="flex flex-col gap-5">
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
        <UpdateSection />
      </CardContent>
    </Card>
  );
}

function UpdateSection() {
  const t = useT(strings);
  const channel = useUpdates((state) => state.geniexChannel);
  const setChannel = useUpdates((state) => state.setGeniexChannel);
  const check = useQuery(runtimeUpdateQuery(channel));
  const cached = useQuery(cachedRuntimesQuery);
  const { data: jobs = [] } = useQuery(jobsQuery);
  const install = useInstallRuntime();
  const installing = jobs.find(
    (job) => job.kind.type === "install_runtime" && job.state.state === "running",
  );

  const current = check.data?.current ?? null;
  const latest = check.data?.latest ?? null;
  const others = (cached.data ?? []).filter((installer) => installer.version !== current);

  return (
    <section className="flex flex-col gap-3 border-t pt-4">
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-sm font-medium">{t("updates")}</h3>
        <SegmentedControl
          name="geniex-channel"
          label={t("channel")}
          value={channel}
          onChange={setChannel}
          options={[
            { value: "stable", label: t("stable") },
            { value: "prerelease", label: t("prerelease") },
          ]}
        />
      </div>

      {installing && installing.kind.type === "install_runtime" ? (
        <p className="text-sm text-muted-foreground">
          {t("installing", { version: installing.kind.version })}
        </p>
      ) : check.isPending ? (
        <p className="text-sm text-muted-foreground">{t("checking")}</p>
      ) : check.isError ? (
        <p className="text-sm text-destructive">
          {t("checkFailed", { message: check.error.message })}
        </p>
      ) : check.data.updateAvailable && latest ? (
        <AvailableUpdate
          release={latest}
          missing={current === null}
          signed={check.data.publisherSigned}
          busy={install.isPending}
          onInstall={() => install.mutate({ source: "release", release: latest })}
        />
      ) : (
        <p className="flex items-center gap-2 text-sm text-muted-foreground">
          <CircleCheck className="size-4 text-success" />
          {t("upToDate")}
        </p>
      )}

      {others.length > 0 && !installing && (
        <div className="flex flex-col gap-1.5">
          <p className="text-xs text-muted-foreground">{t("otherVersions")}</p>
          <ul className="flex flex-col gap-1">
            {others.map((installer) => (
              <li key={installer.version} className="flex items-center gap-2 text-sm">
                <span className="font-mono text-[13px]">{installer.version}</span>
                {installer.bundled && <Badge>{t("bundled")}</Badge>}
                <span className="text-xs text-muted-foreground">
                  {formatBytes(installer.sizeBytes)}
                </span>
                <Button
                  size="sm"
                  variant="ghost"
                  className="ml-auto"
                  disabled={install.isPending}
                  onClick={() => install.mutate({ source: "cached", version: installer.version })}
                >
                  <RotateCcw />
                  {current ? t("rollBack") : t("reinstall")}
                </Button>
              </li>
            ))}
          </ul>
        </div>
      )}
      {install.isError && <p className="text-xs text-destructive">{install.error.message}</p>}
    </section>
  );
}

function AvailableUpdate({
  release,
  missing,
  signed,
  busy,
  onInstall,
}: {
  release: RuntimeRelease;
  missing: boolean;
  signed: boolean;
  busy: boolean;
  onInstall: () => void;
}) {
  const t = useT(strings);
  return (
    <div className="flex flex-col gap-2 rounded-md border border-primary/40 bg-primary/5 p-3">
      <div className="flex flex-wrap items-center gap-2">
        <ArrowUpCircle className="size-4 text-primary" />
        <span className="text-sm font-medium">{t("available", { version: release.version })}</span>
        {release.prerelease && <Badge tone="warning">{t("prereleaseBadge")}</Badge>}
        {release.releasedAt && (
          <span className="text-xs text-muted-foreground">
            {t("released", { when: formatRelative(Date.parse(release.releasedAt)) })}
          </span>
        )}
        {release.notesUrl && (
          <Button
            size="sm"
            variant="ghost"
            className="h-6 px-2 text-xs"
            onClick={() => release.notesUrl && call(() => commands.openUrl(release.notesUrl ?? ""))}
          >
            <ExternalLink />
            {t("notes")}
          </Button>
        )}
      </div>
      {!signed && (
        <p className="flex items-start gap-1.5 text-xs text-warning">
          <ShieldAlert className="mt-0.5 size-3.5 shrink-0" />
          {t("unsigned")}
        </p>
      )}
      <div className="flex items-center gap-3">
        <p className="min-w-0 flex-1 text-xs text-muted-foreground">{t("updateHint")}</p>
        <Button size="sm" onClick={onInstall} disabled={busy}>
          {missing
            ? t("install", { version: release.version })
            : t("update", { version: release.version })}
          <span className="font-mono text-xs opacity-80">
            {formatBytes(release.installer.size)}
          </span>
        </Button>
      </div>
    </div>
  );
}
