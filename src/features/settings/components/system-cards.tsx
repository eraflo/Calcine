import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, X } from "lucide-react";
import { type FormEvent, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { defineMessages, useT } from "@/i18n";
import { call, commands, unwrap } from "@/lib/api";

const strings = defineMessages({
  en: {
    startup: "Startup",
    autostart: "Start with Windows",
    autostartHint: "Calcine opens in the tray so apps can use the API right away.",
    api: "Local API",
    apiHint: "Where other apps reach your models. Calcine only listens on this PC.",
    port: "Port",
    portHint: "18181 is GenieX's default, so existing GenieX clients work unchanged.",
    apply: "Apply",
    portSaved: "The API now listens on port {port}. Update the apps that use it.",
    origins: "Allowed web origins",
    originsHint:
      "Web pages served from these origins (a local web UI, for example) can call the API with a key.",
    originsEmpty: "Only Calcine itself.",
    originPlaceholder: "http://localhost:3000",
    add: "Add",
    removeOrigin: "Remove {origin}",
    invalidOrigin: "{origin} isn't an origin: use the form http://host:port, without a path.",
  },
  fr: {
    startup: "Démarrage",
    autostart: "Lancer avec Windows",
    autostartHint:
      "Calcine s'ouvre dans la zone de notification pour que les applis puissent utiliser l'API tout de suite.",
    api: "API locale",
    apiHint: "Où les autres applis accèdent à vos modèles. Calcine n'écoute que sur ce PC.",
    port: "Port",
    portHint:
      "18181 est le port par défaut de GenieX : les clients GenieX existants fonctionnent sans changement.",
    apply: "Appliquer",
    portSaved:
      "L'API écoute maintenant sur le port {port}. Mettez à jour les applis qui l'utilisent.",
    origins: "Origines web autorisées",
    originsHint:
      "Les pages servies depuis ces origines (une interface web locale, par exemple) peuvent appeler l'API avec une clé.",
    originsEmpty: "Calcine uniquement.",
    originPlaceholder: "http://localhost:3000",
    add: "Ajouter",
    removeOrigin: "Retirer {origin}",
    invalidOrigin:
      "{origin} n'est pas une origine : utilisez la forme http://hôte:port, sans chemin.",
  },
});

const gatewaySettingsQuery = {
  queryKey: ["gateway-settings"],
  queryFn: () => call(commands.gatewaySettings),
};

/** Start Calcine with Windows, in the tray. */
export function StartupCard() {
  const t = useT(strings);
  const queryClient = useQueryClient();
  const enabled = useQuery({
    queryKey: ["autostart"],
    queryFn: () => unwrap(commands.autostartEnabled),
  });
  const toggle = useMutation({
    mutationFn: (value: boolean) => unwrap(() => commands.setAutostart(value)),
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["autostart"] }),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("startup")}</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        <div className="flex items-center justify-between gap-4">
          <label htmlFor="autostart" className="flex flex-col">
            <span className="text-sm">{t("autostart")}</span>
            <span className="text-[11px] text-muted-foreground">{t("autostartHint")}</span>
          </label>
          <Switch
            id="autostart"
            checked={enabled.data ?? false}
            disabled={enabled.isPending || toggle.isPending}
            onCheckedChange={(value) => toggle.mutate(value)}
          />
        </div>
        {(enabled.isError || toggle.isError) && (
          <p className="text-xs text-destructive">{(enabled.error ?? toggle.error)?.message}</p>
        )}
      </CardContent>
    </Card>
  );
}

/** The local API's port and allowed browser origins. */
export function LocalApiCard() {
  const t = useT(strings);
  const queryClient = useQueryClient();
  const settings = useQuery(gatewaySettingsQuery);
  const [port, setPort] = useState("");
  const [origin, setOrigin] = useState("");

  useEffect(() => {
    if (settings.data) setPort(String(settings.data.port));
  }, [settings.data]);

  const refresh = () =>
    Promise.all(
      ["gateway-settings", "gateway", "gateway-connection"].map((key) =>
        queryClient.invalidateQueries({ queryKey: [key] }),
      ),
    );
  const savePort = useMutation({
    mutationFn: (value: number) => unwrap(() => commands.setGatewayPort(value)),
    onSettled: refresh,
  });
  const saveOrigins = useMutation({
    mutationFn: (origins: string[]) => unwrap(() => commands.setAllowedOrigins(origins)),
    onSettled: refresh,
  });

  const origins = settings.data?.allowedOrigins ?? [];
  const portNumber = Number(port);
  const portChanged = settings.data !== undefined && portNumber !== settings.data.port;

  const [invalid, setInvalid] = useState<string | null>(null);
  const addOrigin = (event: FormEvent) => {
    event.preventDefault();
    const value = origin.trim();
    if (!value) return;
    if (!isOrigin(value)) {
      setInvalid(t("invalidOrigin", { origin: value }));
      return;
    }
    setInvalid(null);
    saveOrigins.mutate([...origins, value], { onSuccess: () => setOrigin("") });
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("api")}</CardTitle>
        <CardDescription>{t("apiHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-5">
        <form
          className="flex flex-col gap-1.5"
          onSubmit={(event) => {
            event.preventDefault();
            if (portChanged) savePort.mutate(portNumber);
          }}
        >
          <label htmlFor="api-port" className="text-sm">
            {t("port")}
          </label>
          <div className="flex items-center gap-2">
            <Input
              id="api-port"
              type="number"
              min={1024}
              max={65535}
              value={port}
              onChange={(event) => setPort(event.target.value)}
              className="w-32 font-mono"
            />
            <Button type="submit" size="sm" disabled={!portChanged || savePort.isPending}>
              {t("apply")}
            </Button>
          </div>
          <p className="text-[11px] text-muted-foreground">{t("portHint")}</p>
          {savePort.isError && <p className="text-xs text-destructive">{savePort.error.message}</p>}
          {savePort.isSuccess && (
            <p className="text-xs text-success">
              {t("portSaved", { port: String(settings.data?.port ?? port) })}
            </p>
          )}
        </form>

        <div className="flex flex-col gap-1.5">
          <span className="text-sm">{t("origins")}</span>
          <div className="flex flex-wrap gap-1.5">
            {origins.length === 0 ? (
              <span className="text-xs text-muted-foreground">{t("originsEmpty")}</span>
            ) : (
              origins.map((allowed) => (
                <Badge key={allowed} tone="outline" className="gap-0.5 pr-0.5 font-mono">
                  {allowed}
                  <button
                    type="button"
                    onClick={() => saveOrigins.mutate(origins.filter((o) => o !== allowed))}
                    className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                    aria-label={t("removeOrigin", { origin: allowed })}
                  >
                    <X />
                  </button>
                </Badge>
              ))
            )}
          </div>
          <form onSubmit={addOrigin} className="flex items-center gap-2">
            <Input
              value={origin}
              onChange={(event) => setOrigin(event.target.value)}
              placeholder={t("originPlaceholder")}
              aria-label={t("origins")}
              spellCheck={false}
              className="w-72 font-mono"
            />
            <Button type="submit" size="sm" variant="secondary" disabled={!origin.trim()}>
              <Plus />
              {t("add")}
            </Button>
          </form>
          <p className="text-[11px] text-muted-foreground">{t("originsHint")}</p>
          {(invalid || saveOrigins.isError) && (
            <p className="text-xs text-destructive">{invalid ?? saveOrigins.error?.message}</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

/** `http(s)://host[:port]` with nothing after, as the gateway accepts. */
export function isOrigin(value: string): boolean {
  return /^https?:\/\/(\[::1\]|[a-z0-9.-]+)(:\d{1,5})?$/i.test(value.trim());
}
