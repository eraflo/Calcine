import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FolderOpen, Plus, RefreshCw, ShieldCheck, Wifi, X } from "lucide-react";
import { type FormEvent, useEffect, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { ConfirmDialog } from "@/components/calcine/feedback/confirm-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { gatewayQuery } from "@/features/server/api";
import { defineMessages, useT } from "@/i18n";
import { commands, unwrap } from "@/lib/api";
import { gatewaySettingsQuery } from "./system-cards";

const strings = defineMessages({
  en: {
    title: "Local network",
    hint: "Let your other devices use your models, over HTTPS. Only keys allowed for other devices work there, and they can't read files on this PC.",
    enable: "Answer other devices",
    enableHint:
      "Off by default. Windows may ask whether Calcine can use the network: allow private networks.",
    url: "Address for other devices",
    hostName: "Or by name: {url}",
    fingerprint: "Certificate fingerprint (SHA-256)",
    fingerprintHint:
      "The certificate is made on this PC, so other devices don't trust it by default. Import certificate.pem there, or check that the fingerprint they see is this one.",
    showCertificate: "Show certificate.pem",
    renew: "New certificate",
    renewTitle: "Make a new certificate?",
    renewBody:
      "Devices that trust the current certificate will refuse to connect until they trust the new one.",
    steps:
      "To connect a device: create a key with “From other devices” on the Server page, then use this address and that key there.",
    port: "Port",
    apply: "Apply",
    allowed: "Allowed addresses",
    allowedEmpty: "Private networks: home, office, Tailscale.",
    allowedPlaceholder: "192.168.1.0/24",
    allowedHint: "An address or a range. Leave empty to allow private networks.",
    add: "Add",
    remove: "Remove {range}",
    invalidRange: "{range} isn't an address or a range like 192.168.1.0/24.",
  },
  fr: {
    title: "Réseau local",
    hint: "Laissez vos autres appareils utiliser vos modèles, en HTTPS. Seules les clés autorisées pour les autres appareils y fonctionnent, et elles ne peuvent pas lire les fichiers de ce PC.",
    enable: "Répondre aux autres appareils",
    enableHint:
      "Désactivé par défaut. Windows peut demander si Calcine peut utiliser le réseau : autorisez les réseaux privés.",
    url: "Adresse pour les autres appareils",
    hostName: "Ou par nom : {url}",
    fingerprint: "Empreinte du certificat (SHA-256)",
    fingerprintHint:
      "Le certificat est créé sur ce PC : les autres appareils ne lui font pas confiance d'office. Importez-y certificate.pem, ou vérifiez que l'empreinte qu'ils affichent est bien celle-ci.",
    showCertificate: "Afficher certificate.pem",
    renew: "Nouveau certificat",
    renewTitle: "Créer un nouveau certificat ?",
    renewBody:
      "Les appareils qui font confiance au certificat actuel refuseront de se connecter tant qu'ils ne feront pas confiance au nouveau.",
    steps:
      "Pour connecter un appareil : créez une clé avec « Depuis d'autres appareils » sur la page Serveur, puis utilisez-y cette adresse et cette clé.",
    port: "Port",
    apply: "Appliquer",
    allowed: "Adresses autorisées",
    allowedEmpty: "Réseaux privés : maison, bureau, Tailscale.",
    allowedPlaceholder: "192.168.1.0/24",
    allowedHint: "Une adresse ou une plage. Laissez vide pour autoriser les réseaux privés.",
    add: "Ajouter",
    remove: "Retirer {range}",
    invalidRange: "{range} n'est ni une adresse ni une plage comme 192.168.1.0/24.",
  },
});

/** An IPv4 or IPv6 address, optionally with a prefix length. */
export function isRange(value: string): boolean {
  const [address = "", prefix, extra] = value.trim().split("/");
  if (extra !== undefined) return false;
  const ipv4 = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/.exec(address);
  const ipv6 = !ipv4 && /^[0-9a-f:]+$/i.test(address) && address.includes(":");
  if (ipv4?.slice(1).some((part) => Number(part) > 255)) return false;
  if (!ipv4 && !ipv6) return false;
  if (prefix === undefined) return true;
  const bits = Number(prefix);
  return /^\d+$/.test(prefix) && bits <= (ipv4 ? 32 : 128);
}

/** Answer other devices on the local network, over HTTPS. */
export function NetworkCard() {
  const t = useT(strings);
  const queryClient = useQueryClient();
  const settings = useQuery(gatewaySettingsQuery);
  const { data: status } = useQuery(gatewayQuery);
  const [port, setPort] = useState("");
  const [range, setRange] = useState("");
  const [invalid, setInvalid] = useState<string | null>(null);
  const [renewing, setRenewing] = useState(false);

  useEffect(() => {
    if (settings.data) setPort(String(settings.data.networkPort));
  }, [settings.data]);

  const refresh = () =>
    Promise.all(
      ["gateway-settings", "gateway"].map((key) =>
        queryClient.invalidateQueries({ queryKey: [key] }),
      ),
    );
  const save = useMutation({
    mutationFn: (next: { enabled: boolean; port: number; allowed: string[] }) =>
      unwrap(() => commands.setNetwork(next.enabled, next.port, next.allowed)),
    onSettled: refresh,
  });
  const renew = useMutation({
    mutationFn: () => unwrap(commands.renewNetworkCertificate),
    onSettled: refresh,
  });
  const show = useMutation({ mutationFn: () => unwrap(commands.showNetworkCertificate) });

  const current = settings.data;
  const enabled = current?.networkEnabled ?? false;
  const allowed = current?.networkAllowed ?? [];
  const portNumber = Number(port);
  const portChanged = current !== undefined && portNumber !== current.networkPort;
  const network = status?.network;
  const apply = (patch: Partial<{ enabled: boolean; port: number; allowed: string[] }>) =>
    current &&
    save.mutate({
      enabled: current.networkEnabled,
      port: current.networkPort,
      allowed: current.networkAllowed,
      ...patch,
    });

  const addRange = (event: FormEvent) => {
    event.preventDefault();
    const value = range.trim();
    if (!value) return;
    if (!isRange(value)) {
      setInvalid(t("invalidRange", { range: value }));
      return;
    }
    setInvalid(null);
    apply({ allowed: [...allowed, value] });
    setRange("");
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Wifi className="size-4 text-primary" />
          {t("title")}
        </CardTitle>
        <CardDescription>{t("hint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-5">
        <div className="flex items-center justify-between gap-4">
          <label htmlFor="network-enabled" className="flex flex-col">
            <span className="text-sm">{t("enable")}</span>
            <span className="text-[11px] text-muted-foreground">{t("enableHint")}</span>
          </label>
          <Switch
            id="network-enabled"
            checked={enabled}
            disabled={!current || save.isPending}
            onCheckedChange={(next) => apply({ enabled: next })}
          />
        </div>

        {enabled && status?.networkError && (
          <p className="text-xs text-warning">{status.networkError}</p>
        )}
        {save.isError && <p className="text-xs text-destructive">{save.error.message}</p>}

        {enabled && network && (
          <div className="flex flex-col gap-4 rounded-md border bg-muted/30 p-3">
            <div className="flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">{t("url")}</span>
              <div className="flex items-center gap-2">
                <code className="font-mono text-sm">{network.baseUrl}</code>
                <CopyButton value={network.baseUrl} />
              </div>
              {network.hostName && (
                <span className="text-[11px] text-muted-foreground">
                  {t("hostName", {
                    url: network.baseUrl.replace(/\/\/[^:/]+/, `//${network.hostName}`),
                  })}
                </span>
              )}
            </div>
            <div className="flex flex-col gap-1">
              <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <ShieldCheck className="size-3.5 text-success" />
                {t("fingerprint")}
              </span>
              <div className="flex items-start gap-2">
                <code className="font-mono text-[11px] leading-relaxed break-all">
                  {network.fingerprint}
                </code>
                <CopyButton value={network.fingerprint} />
              </div>
              <span className="text-[11px] text-muted-foreground">{t("fingerprintHint")}</span>
              <div className="mt-1 flex flex-wrap gap-2">
                <Button size="sm" variant="secondary" onClick={() => show.mutate()}>
                  <FolderOpen />
                  {t("showCertificate")}
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setRenewing(true)}>
                  <RefreshCw />
                  {t("renew")}
                </Button>
              </div>
            </div>
            <p className="text-[11px] text-muted-foreground">{t("steps")}</p>
          </div>
        )}

        <form
          className="flex flex-col gap-1.5"
          onSubmit={(event) => {
            event.preventDefault();
            if (portChanged) apply({ port: portNumber });
          }}
        >
          <label htmlFor="network-port" className="text-sm">
            {t("port")}
          </label>
          <div className="flex items-center gap-2">
            <Input
              id="network-port"
              type="number"
              min={1024}
              max={65535}
              value={port}
              onChange={(event) => setPort(event.target.value)}
              className="w-32 font-mono"
            />
            <Button type="submit" size="sm" disabled={!portChanged || save.isPending}>
              {t("apply")}
            </Button>
          </div>
        </form>

        <div className="flex flex-col gap-1.5">
          <span className="text-sm">{t("allowed")}</span>
          <div className="flex flex-wrap gap-1.5">
            {allowed.length === 0 ? (
              <span className="text-xs text-muted-foreground">{t("allowedEmpty")}</span>
            ) : (
              allowed.map((entry) => (
                <Badge key={entry} tone="outline" className="gap-0.5 pr-0.5 font-mono">
                  {entry}
                  <button
                    type="button"
                    onClick={() => apply({ allowed: allowed.filter((other) => other !== entry) })}
                    className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                    aria-label={t("remove", { range: entry })}
                  >
                    <X />
                  </button>
                </Badge>
              ))
            )}
          </div>
          <form onSubmit={addRange} className="flex items-center gap-2">
            <Input
              value={range}
              onChange={(event) => setRange(event.target.value)}
              placeholder={t("allowedPlaceholder")}
              aria-label={t("allowed")}
              spellCheck={false}
              className="w-72 font-mono"
            />
            <Button type="submit" size="sm" variant="secondary" disabled={!range.trim()}>
              <Plus />
              {t("add")}
            </Button>
          </form>
          <p className="text-[11px] text-muted-foreground">{t("allowedHint")}</p>
          {invalid && <p className="text-xs text-destructive">{invalid}</p>}
        </div>
      </CardContent>

      <ConfirmDialog
        open={renewing}
        onOpenChange={setRenewing}
        title={t("renewTitle")}
        confirmLabel={t("renew")}
        busy={renew.isPending}
        onConfirm={() => renew.mutate(undefined, { onSuccess: () => setRenewing(false) })}
      >
        <p>{t("renewBody")}</p>
      </ConfirmDialog>
    </Card>
  );
}
