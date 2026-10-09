import { useQuery } from "@tanstack/react-query";
import { BarChart3 } from "lucide-react";
import { useState } from "react";
import { EmptyState } from "@/components/calcine/feedback/empty-state";
import { Page } from "@/components/calcine/layout/page";
import { useT } from "@/i18n";
import { benchHistoryQuery } from "./api";
import { EnergyCard } from "./components/energy-card";
import { HistoryCard, LeaderboardCard, SessionCard, UsageCard } from "./components/results-cards";
import { RunCard } from "./components/run-card";
import { ToolCard } from "./components/tool-card";
import { sessions as groupSessions } from "./lib/results";
import { messages } from "./messages";

export function BenchmarkPage() {
  const t = useT(messages);
  const { data: history = [] } = useQuery(benchHistoryQuery);
  // Energy profiles have their own card.
  const sessions = groupSessions(history).filter((session) => session.source !== "energy_profile");
  const [selected, setSelected] = useState<string | null>(null);
  const shown = sessions.find((session) => session.id === selected) ?? sessions[0];

  return (
    <Page title={t("title")} description={t("description")}>
      <ToolCard />
      <RunCard />
      {shown ? (
        <SessionCard session={shown} latest={shown.id === sessions[0]?.id} />
      ) : (
        <EmptyState icon={BarChart3} title={t("resultsTitle")}>
          {t("noResults")}
        </EmptyState>
      )}
      <EnergyCard />
      <LeaderboardCard history={history} />
      <UsageCard />
      {sessions.length > 0 && (
        <HistoryCard
          sessions={sessions}
          history={history}
          selected={shown?.id ?? null}
          onSelect={setSelected}
        />
      )}
    </Page>
  );
}
