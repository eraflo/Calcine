import { Page } from "@/components/calcine/layout/page";
import { ConnectCard } from "./components/connect-card";
import { KeysCard } from "./components/keys-card";
import { RequestsCard } from "./components/requests-card";
import { StatusCard } from "./components/status-card";

export function ServerPage() {
  return (
    <Page title="Server" description="The local, OpenAI-compatible API for your other apps">
      <StatusCard />
      <ConnectCard />
      <KeysCard />
      <RequestsCard />
    </Page>
  );
}
