import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowUpCircle, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import type { ComponentType } from "react";
import { LogoMark } from "@/components/calcine/brand/logo";
import { StatusDot } from "@/components/calcine/feedback/status-dot";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useLiveReply } from "@/features/chat/store";
import { runtimeQuery, runtimeUpdateQuery } from "@/features/hardware/api";
import { gatewayQuery } from "@/features/server/api";
import { appInfoQuery } from "@/features/settings/api";
import { useT } from "@/i18n";
import { CalcineError } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useLayout } from "@/stores/layout";
import { useUpdates } from "@/stores/updates";
import { messages } from "../messages";
import { mainNavigation, settingsNavigation } from "../navigation";
import { LiveGauges } from "./live-gauges";

const linkClass =
  "flex h-8 items-center gap-2.5 rounded-md px-2.5 text-[13px] font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground [&.active]:bg-accent [&.active]:text-foreground [&.active_svg]:text-primary";

/** The navigation. Collapsed (Ctrl+B), it shows icons only, named in tooltips. */
export function Sidebar() {
  const t = useT(messages);
  const generating = useGenerating();
  const collapsed = useLayout((state) => state.sidebarCollapsed);
  const toggle = useLayout((state) => state.toggleSidebar);
  const toggleLabel = collapsed ? t("expandSidebar") : t("collapseSidebar");

  return (
    <aside
      className={cn(
        "flex min-h-0 flex-col border-r bg-sidebar py-3",
        collapsed ? "items-center px-1.5" : "px-2.5",
      )}
    >
      <div className={cn("flex items-center gap-2 pb-5", collapsed ? "flex-col" : "pl-2")}>
        <LogoMark />
        {!collapsed && (
          <>
            <span className="text-[15px] font-semibold tracking-tight">Calcine</span>
            <BackendBadge />
          </>
        )}
        <Tooltip content={toggleLabel} side="right">
          <Button
            variant="ghost"
            size="icon"
            className={cn("size-7 text-muted-foreground", !collapsed && "ml-auto")}
            onClick={toggle}
            aria-label={toggleLabel}
            aria-expanded={!collapsed}
          >
            {collapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
          </Button>
        </Tooltip>
      </div>

      <nav className="flex flex-col gap-0.5" aria-label={t("mainNavigation")}>
        {mainNavigation.map(({ to, label, icon }) => (
          <NavLink key={to} to={to} label={t(label)} icon={icon} collapsed={collapsed} />
        ))}
      </nav>

      <div className="mt-auto flex flex-col gap-2">
        {generating && !collapsed && <LiveGauges />}
        <NavLink
          to={settingsNavigation.to}
          label={t(settingsNavigation.label)}
          icon={settingsNavigation.icon}
          collapsed={collapsed}
        />
        <RuntimeStatus collapsed={collapsed} />
      </div>
    </aside>
  );
}

function NavLink({
  to,
  label,
  icon: Icon,
  collapsed,
}: {
  to: string;
  label: string;
  icon: ComponentType<{ className?: string }>;
  collapsed: boolean;
}) {
  if (!collapsed) {
    return (
      <Link to={to} className={linkClass}>
        <Icon className="size-4" />
        {label}
      </Link>
    );
  }
  return (
    <Tooltip content={label} side="right">
      <Link to={to} className={cn(linkClass, "w-9 justify-center px-0")} aria-label={label}>
        <Icon className="size-4" />
      </Link>
    </Tooltip>
  );
}

/** A model is answering: in Calcine's chat, or for another app through the API. */
function useGenerating() {
  const chatting = useLiveReply((state) => state.messageId !== null);
  const { data: gateway } = useQuery(gatewayQuery);
  return chatting || (gateway?.activeRequests ?? 0) > 0;
}

function BackendBadge() {
  const t = useT(messages);
  const { data } = useQuery(appInfoQuery);
  if (data?.backend !== "mock") return null;
  return (
    <Badge tone="warning" className="ml-auto">
      {t("mock")}
    </Badge>
  );
}

function RuntimeStatus({ collapsed }: { collapsed: boolean }) {
  const t = useT(messages);
  const { data, error, isPending } = useQuery(runtimeQuery);
  const channel = useUpdates((state) => state.geniexChannel);
  const autoCheck = useUpdates((state) => state.autoCheck);
  const { data: update } = useQuery({
    ...runtimeUpdateQuery(channel),
    enabled: Boolean(data) && autoCheck,
  });
  const kind = error instanceof CalcineError ? error.kind : undefined;

  const [tone, label] = isPending
    ? (["busy", t("detectingGeniex")] as const)
    : data
      ? (["ok", t("geniexVersion", { version: data.cliVersion })] as const)
      : kind === "not_in_tauri"
        ? (["idle", t("browserPreview")] as const)
        : kind === "runtime_not_found"
          ? (["error", t("geniexMissing")] as const)
          : (["error", t("geniexUnavailable")] as const);

  if (collapsed) {
    return (
      <Tooltip content={label} side="right">
        <Link
          to="/hardware"
          className="flex size-9 items-center justify-center rounded-md border bg-background/40"
          aria-label={label}
        >
          <StatusDot tone={tone} />
        </Link>
      </Tooltip>
    );
  }
  return (
    <Link
      to="/hardware"
      className="flex items-center gap-2 rounded-md border bg-background/40 px-2.5 py-2 text-xs text-muted-foreground hover:text-foreground"
    >
      <StatusDot tone={tone} />
      <span className="truncate">{label}</span>
      {update?.updateAvailable && update.latest && (
        <ArrowUpCircle
          className="ml-auto size-3.5 shrink-0 text-info"
          aria-label={t("geniexUpdate", { version: update.latest.version })}
        >
          <title>{t("geniexUpdate", { version: update.latest.version })}</title>
        </ArrowUpCircle>
      )}
    </Link>
  );
}
