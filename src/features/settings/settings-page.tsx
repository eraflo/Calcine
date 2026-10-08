import { useQuery } from "@tanstack/react-query";
import type { LucideIcon } from "lucide-react";
import { Monitor, Moon, Sun } from "lucide-react";
import { Page } from "@/components/calcine/layout/page";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { type ThemePreference, useTheme } from "@/stores/theme";
import { appInfoQuery } from "./api";

const THEMES: { value: ThemePreference; label: string; icon: LucideIcon }[] = [
  { value: "dark", label: "Dark", icon: Moon },
  { value: "light", label: "Light", icon: Sun },
  { value: "system", label: "System", icon: Monitor },
];

export function SettingsPage() {
  return (
    <Page title="Settings">
      <AppearanceCard />
      <AboutCard />
    </Page>
  );
}

function AppearanceCard() {
  const { theme, setTheme } = useTheme();
  return (
    <Card>
      <CardHeader>
        <CardTitle>Appearance</CardTitle>
        <CardDescription>Calcine is designed dark-first.</CardDescription>
      </CardHeader>
      <CardContent>
        <fieldset className="inline-flex rounded-md border p-0.5">
          <legend className="sr-only">Theme</legend>
          {THEMES.map(({ value, label, icon: Icon }) => (
            <label
              key={value}
              className={cn(
                "inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-sm px-3 text-xs font-medium text-muted-foreground transition-colors has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
                theme === value ? "bg-accent text-foreground" : "hover:text-foreground",
              )}
            >
              <input
                type="radio"
                name="theme"
                value={value}
                checked={theme === value}
                onChange={() => setTheme(value)}
                className="sr-only"
              />
              <Icon className="size-3.5" />
              {label}
            </label>
          ))}
        </fieldset>
      </CardContent>
    </Card>
  );
}

function AboutCard() {
  const { data } = useQuery(appInfoQuery);
  return (
    <Card>
      <CardHeader>
        <CardTitle>About</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
          <dt className="text-muted-foreground">Version</dt>
          <dd className="font-mono text-[13px]">{data?.version ?? "—"}</dd>
          <dt className="text-muted-foreground">Backend</dt>
          <dd>
            {data?.backend === "mock" ? (
              <Badge tone="warning">Mock data</Badge>
            ) : (
              <Badge tone="success">GenieX</Badge>
            )}
          </dd>
          <dt className="text-muted-foreground">License</dt>
          <dd>MIT · GenieX is BSD 3-Clause</dd>
          <dt className="text-muted-foreground">Source</dt>
          <dd className="font-mono text-[13px]">github.com/eraflo/calcine</dd>
        </dl>
      </CardContent>
    </Card>
  );
}
