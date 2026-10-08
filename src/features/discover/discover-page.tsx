import { Page } from "@/components/calcine/page";
import { AddModelCard } from "./add-model-card";
import { AiHubCatalog } from "./catalog";

export function DiscoverPage() {
  return (
    <Page title="Discover" description="Find models that run well on your NPU">
      <AddModelCard />
      <AiHubCatalog />
    </Page>
  );
}
