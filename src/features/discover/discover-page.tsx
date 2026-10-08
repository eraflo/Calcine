import { Compass } from "lucide-react";
import { EmptyState, Page } from "@/components/calcine/page";

export function DiscoverPage() {
  return (
    <Page title="Discover" description="Find models for your NPU">
      <EmptyState icon={Compass} title="Model discovery is coming soon">
        Browse the Qualcomm AI Hub catalog for your chipset, paste any Hugging Face, ModelScope or
        Docker Hub link, or import a local folder.
      </EmptyState>
    </Page>
  );
}
