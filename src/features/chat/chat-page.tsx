import { MessagesSquare } from "lucide-react";
import { EmptyState, Page } from "@/components/calcine/page";

export function ChatPage() {
  return (
    <Page title="Chat" description="Talk to your models, with every GenieX option at hand">
      <EmptyState icon={MessagesSquare} title="Chat is coming soon">
        Streaming replies, reasoning, images and audio, compute unit and power mode per
        conversation.
      </EmptyState>
    </Page>
  );
}
