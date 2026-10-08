import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, Select } from "@/components/ui/field";
import { useT } from "@/i18n";
import type { ServerOptions } from "@/lib/api";
import { formatNumber } from "@/lib/format";
import { gatewayQuery, serverOptionsQuery, useSetServerOptions } from "../api";
import { messages } from "../messages";

const UNLOAD_AFTER_SECS = [60, 300, 900, 1800, 3600, 4 * 3600, 24 * 3600];
const CONTEXT_SIZES = [2048, 4096, 8192, 16384, 32768, 65536, 131072];
const DEFAULTS: ServerOptions = { keepaliveSecs: 300, contextSize: 4096 };

/** `geniex serve` options: how long models stay loaded, llama.cpp context. */
export function LoadingCard() {
  const t = useT(messages);
  const { data: options } = useQuery(serverOptionsQuery);
  const { data: status } = useQuery(gatewayQuery);
  const save = useSetServerOptions();
  const running = status?.server.state === "ready" || status?.server.state === "starting";

  const duration = (secs: number) =>
    secs >= 3600 ? t.plural("hours", secs / 3600) : t.plural("minutes", secs / 60);
  const withDefault = (label: string, isDefault: boolean) =>
    isDefault ? t("defaultChoice", { value: label }) : label;
  const update = (patch: Partial<ServerOptions>) =>
    options && save.mutate({ ...options, ...patch });

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("loading")}</CardTitle>
        <CardDescription>{t("loadingHint")}</CardDescription>
      </CardHeader>
      <CardContent className="grid gap-5 sm:grid-cols-2">
        <Field label={t("unloadAfter")} htmlFor="unload-after" hint={t("unloadAfterHint")}>
          <Select
            id="unload-after"
            value={options?.keepaliveSecs ?? ""}
            disabled={!options || save.isPending}
            onChange={(event) => update({ keepaliveSecs: Number(event.target.value) })}
          >
            {withCurrent(UNLOAD_AFTER_SECS, options?.keepaliveSecs).map((secs) => (
              <option key={secs} value={secs}>
                {withDefault(duration(secs), secs === DEFAULTS.keepaliveSecs)}
              </option>
            ))}
          </Select>
        </Field>
        <Field label={t("contextWindow")} htmlFor="context-window" hint={t("contextWindowHint")}>
          <Select
            id="context-window"
            value={options?.contextSize ?? ""}
            disabled={!options || save.isPending}
            onChange={(event) => update({ contextSize: Number(event.target.value) })}
          >
            {withCurrent(CONTEXT_SIZES, options?.contextSize).map((size) => (
              <option key={size} value={size}>
                {withDefault(
                  t("tokens", { count: formatNumber(size, 0) }),
                  size === DEFAULTS.contextSize,
                )}
              </option>
            ))}
          </Select>
        </Field>
        {(running || save.isError) && (
          <p
            className={
              save.isError
                ? "text-xs text-destructive sm:col-span-2"
                : "text-[11px] text-muted-foreground sm:col-span-2"
            }
          >
            {save.isError ? save.error.message : t("restartsGeniex")}
          </p>
        )}
      </CardContent>
    </Card>
  );
}

/** The choices, plus the current value if it was set to something else. */
function withCurrent(choices: readonly number[], current: number | undefined) {
  return current === undefined || choices.includes(current)
    ? choices
    : [...choices, current].sort((a, b) => a - b);
}
