import { Page } from "@/components/calcine/layout/page";
import { useT } from "@/i18n";
import { ConnectCard } from "./components/connect-card";
import { KeysCard } from "./components/keys-card";
import { RequestsCard } from "./components/requests-card";
import { StatusCard } from "./components/status-card";
import { messages } from "./messages";

export function ServerPage() {
  const t = useT(messages);
  return (
    <Page title={t("title")} description={t("description")}>
      <StatusCard />
      <ConnectCard />
      <KeysCard />
      <RequestsCard />
    </Page>
  );
}
