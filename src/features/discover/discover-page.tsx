import { Page } from "@/components/calcine/layout/page";
import { AddModelCard } from "./components/add-model-card";
import { AiHubCatalog } from "./components/catalog";

export function DiscoverPage() {
  return (
    <Page title="Discover" description="Find models that run well on your NPU">
      <AddModelCard />
      <AiHubCatalog />
    </Page>
  );
}
