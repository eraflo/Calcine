import { Server } from "lucide-react";
import { EmptyState } from "@/components/calcine/feedback/empty-state";
import { Page } from "@/components/calcine/layout/page";
import { API_BASE_URL } from "@/lib/api";

export function ServerPage() {
  return (
    <Page title="Server" description="The local, OpenAI-compatible API for your other apps">
      <EmptyState icon={Server} title="The local API is coming soon">
        Apps will connect to <code className="font-mono text-foreground">{API_BASE_URL}</code> with
        a per-app key.
      </EmptyState>
    </Page>
  );
}
