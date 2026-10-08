import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { LogoMark } from "@/components/calcine/logo";
import { StatusDot } from "@/components/calcine/status-dot";
import { Badge } from "@/components/ui/badge";
import { runtimeQuery } from "@/features/hardware/api";
import { appInfoQuery } from "@/features/settings/api";
import { CalcineError } from "@/lib/api";
import { mainNavigation, settingsNavigation } from "../navigation";

const linkClass =
  "flex h-8 items-center gap-2.5 rounded-md px-2.5 text-[13px] font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground [&.active]:bg-accent [&.active]:text-foreground [&.active_svg]:text-primary";

export function Sidebar() {
  return (
    <aside className="flex min-h-0 flex-col border-r bg-sidebar px-2.5 py-3">
      <div className="flex items-center gap-2 px-2 pb-5">
        <LogoMark />
        <span className="text-[15px] font-semibold tracking-tight">Calcine</span>
        <BackendBadge />
      </div>

      <nav className="flex flex-col gap-0.5" aria-label="Main">
        {mainNavigation.map(({ to, label, icon: Icon }) => (
          <Link key={to} to={to} className={linkClass}>
            <Icon className="size-4" />
            {label}
          </Link>
        ))}
      </nav>

      <div className="mt-auto flex flex-col gap-2">
        <Link to={settingsNavigation.to} className={linkClass}>
          <settingsNavigation.icon className="size-4" />
          {settingsNavigation.label}
        </Link>
        <RuntimeStatus />
      </div>
    </aside>
  );
}

function BackendBadge() {
  const { data } = useQuery(appInfoQuery);
  if (data?.backend !== "mock") return null;
  return (
    <Badge tone="warning" className="ml-auto">
      Mock
    </Badge>
  );
}

function RuntimeStatus() {
  const { data, error, isPending } = useQuery(runtimeQuery);
  const kind = error instanceof CalcineError ? error.kind : undefined;

  const [tone, label] = isPending
    ? (["busy", "Detecting GenieX…"] as const)
    : data
      ? (["ok", `GenieX ${data.cliVersion}`] as const)
      : kind === "not_in_tauri"
        ? (["idle", "Browser preview"] as const)
        : kind === "runtime_not_found"
          ? (["error", "GenieX not installed"] as const)
          : (["error", "GenieX unavailable"] as const);

  return (
    <Link
      to="/hardware"
      className="flex items-center gap-2 rounded-md border bg-background/40 px-2.5 py-2 text-xs text-muted-foreground hover:text-foreground"
    >
      <StatusDot tone={tone} />
      <span className="truncate">{label}</span>
    </Link>
  );
}
