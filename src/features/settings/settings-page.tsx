import { useMutation, useQuery } from "@tanstack/react-query";
import { FolderOpen, Languages, Monitor, Moon, Sun, Trash2 } from "lucide-react";
import { type ReactNode, useState } from "react";
import { ConfirmDialog } from "@/components/calcine/feedback/confirm-dialog";
import { Page } from "@/components/calcine/layout/page";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Switch } from "@/components/ui/switch";
import { hardwareQuery } from "@/features/hardware/api";
import { modelsQuery, useCleanModels } from "@/features/library/api";
import { type LanguagePreference, useLanguage, useT } from "@/i18n";
import { commands, unwrap } from "@/lib/api";
import { formatBytes, totalBytes } from "@/lib/format";
import { useTheme } from "@/stores/theme";
import { appInfoQuery } from "./api";
import { NetworkCard } from "./components/network-card";
import { LocalApiCard, StartupCard } from "./components/system-cards";
import { UpdatesCard } from "./components/updates-card";
import { messages } from "./messages";

export function SettingsPage() {
  const t = useT(messages);
  return (
    <Page title={t("title")}>
      <AppearanceCard />
      <StartupCard />
      <LocalApiCard />
      <NetworkCard />
      <StorageCard />
      <UpdatesCard />
      <AboutCard />
    </Page>
  );
}

function AppearanceCard() {
  const t = useT(messages);
  const { theme, setTheme, mica, setMica } = useTheme();
  const { preference, setPreference } = useLanguage();
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("appearance")}</CardTitle>
        <CardDescription>{t("appearanceHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Row label={t("theme")}>
          <SegmentedControl
            name="theme"
            label={t("theme")}
            value={theme}
            onChange={setTheme}
            options={[
              { value: "dark", label: t("dark"), icon: Moon },
              { value: "light", label: t("light"), icon: Sun },
              { value: "system", label: t("system"), icon: Monitor },
            ]}
          />
        </Row>
        <Row label={t("mica")} hint={t("micaHint")}>
          <Switch checked={mica} onCheckedChange={setMica} aria-label={t("mica")} />
        </Row>
        <Row label={t("language")} hint={t("languageHint")}>
          <SegmentedControl<LanguagePreference>
            name="language"
            label={t("language")}
            value={preference}
            onChange={setPreference}
            options={[
              { value: "en", label: "English" },
              { value: "fr", label: "Français" },
              { value: "system", label: t("system"), icon: Languages },
            ]}
          />
        </Row>
      </CardContent>
    </Card>
  );
}

function StorageCard() {
  const t = useT(messages);
  const { data: hardware } = useQuery(hardwareQuery);
  const { data: models = [] } = useQuery(modelsQuery);
  const openFolder = useMutation({ mutationFn: () => unwrap(commands.openModelsFolder) });
  const clean = useCleanModels();
  const [confirming, setConfirming] = useState(false);
  const [typed, setTyped] = useState("");
  const disk = hardware?.modelsDisk;
  const used = formatBytes(totalBytes(models.map((model) => model.sizeBytes)));

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("storage")}</CardTitle>
        <CardDescription>{t("storageHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Row label={t("modelsFolder")}>
          <div className="flex min-w-0 items-center gap-2">
            <div className="flex min-w-0 flex-col items-end">
              <span className="truncate font-mono text-xs" title={disk?.path}>
                {disk?.path ?? "—"}
              </span>
              <span className="text-[11px] text-muted-foreground">
                {t.plural("modelsUsing", models.length, { size: used })}
                {disk &&
                  ` · ${t("diskFree", {
                    free: formatBytes(disk.availableBytes),
                    total: formatBytes(disk.totalBytes),
                  })}`}
              </span>
            </div>
            <Button size="sm" variant="secondary" onClick={() => openFolder.mutate()}>
              <FolderOpen />
              {t("openFolder")}
            </Button>
          </div>
        </Row>
        {openFolder.isError && (
          <p className="text-xs text-destructive">{openFolder.error.message}</p>
        )}

        <div className="flex flex-col gap-3 rounded-md border border-destructive/40 p-3">
          <p className="text-xs font-medium tracking-wide text-destructive uppercase">
            {t("dangerZone")}
          </p>
          <div className="flex items-center gap-3">
            <p className="min-w-0 flex-1 text-xs text-muted-foreground">
              {t("cleanHint", { size: used })}
            </p>
            <Button
              size="sm"
              variant="destructive"
              disabled={models.length === 0}
              onClick={() => {
                setTyped("");
                setConfirming(true);
              }}
            >
              <Trash2 />
              {t("clean")}
            </Button>
          </div>
        </div>

        <ConfirmDialog
          open={confirming}
          onOpenChange={setConfirming}
          title={t("cleanTitle")}
          confirmLabel={t("cleanConfirm")}
          destructive
          busy={clean.isPending || typed.trim().toLowerCase() !== t("cleanWord")}
          onConfirm={() => clean.mutate(undefined, { onSuccess: () => setConfirming(false) })}
        >
          <p>{t("cleanBody", { count: models.length, size: used })}</p>
          <label htmlFor="clean-confirm" className="mt-3 block text-xs">
            {t("cleanConfirmLabel", { word: t("cleanWord") })}
          </label>
          <Input
            id="clean-confirm"
            className="mt-1.5"
            value={typed}
            onChange={(event) => setTyped(event.target.value)}
            placeholder={t("cleanWord")}
            autoComplete="off"
          />
          {clean.isError && <p className="mt-2 text-destructive">{clean.error.message}</p>}
        </ConfirmDialog>
      </CardContent>
    </Card>
  );
}

function AboutCard() {
  const t = useT(messages);
  const { data } = useQuery(appInfoQuery);
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("about")}</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
          <dt className="text-muted-foreground">{t("version")}</dt>
          <dd className="font-mono text-[13px]">{data?.version ?? "—"}</dd>
          <dt className="text-muted-foreground">{t("backend")}</dt>
          <dd>
            {data?.backend === "mock" ? (
              <Badge tone="warning">{t("mockData")}</Badge>
            ) : (
              <Badge tone="success">GenieX</Badge>
            )}
          </dd>
          <dt className="text-muted-foreground">{t("license")}</dt>
          <dd>{t("licenseValue")}</dd>
          <dt className="text-muted-foreground">{t("source")}</dt>
          <dd className="font-mono text-[13px]">github.com/eraflo/calcine</dd>
        </dl>
      </CardContent>
    </Card>
  );
}

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex flex-col">
        <span className="text-sm">{label}</span>
        {hint && <span className="text-[11px] text-muted-foreground">{hint}</span>}
      </div>
      {children}
    </div>
  );
}
