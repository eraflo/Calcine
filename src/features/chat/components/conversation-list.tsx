import { MessageSquarePlus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useT } from "@/i18n";
import { formatRelative } from "@/lib/format";
import { cn } from "@/lib/utils";
import { messages } from "../messages";
import { useChat } from "../store";

export function ConversationList({ onNew }: { onNew: () => void }) {
  const t = useT(messages);
  const conversations = useChat((state) => state.conversations);
  const activeId = useChat((state) => state.activeId);
  const select = useChat((state) => state.select);
  const remove = useChat((state) => state.remove);

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r">
      <div className="p-3">
        <Button className="w-full justify-start" onClick={onNew}>
          <MessageSquarePlus />
          {t("newChat")}
        </Button>
      </div>
      <nav
        className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 pb-3"
        aria-label={t("conversations")}
      >
        {conversations.length === 0 ? (
          <p className="px-2 py-4 text-xs text-muted-foreground">{t("conversationsHint")}</p>
        ) : (
          <ul className="flex flex-col gap-0.5">
            {conversations.map((conversation) => {
              const title = conversation.title || t("newChat");
              return (
                <li key={conversation.id} className="group relative">
                  <button
                    type="button"
                    onClick={() => select(conversation.id)}
                    className={cn(
                      "flex w-full flex-col items-start rounded-md px-2.5 py-2 pr-8 text-left transition-colors hover:bg-accent",
                      conversation.id === activeId && "bg-accent",
                    )}
                  >
                    <span className="w-full truncate text-[13px]">{title}</span>
                    <span className="text-[11px] text-muted-foreground">
                      {formatRelative(conversation.updatedAt)}
                    </span>
                  </button>
                  <button
                    type="button"
                    onClick={() => remove(conversation.id)}
                    className="absolute top-2 right-1.5 rounded-sm p-1 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:text-destructive focus-visible:opacity-100"
                    aria-label={t("deleteConversation", { title })}
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </nav>
    </aside>
  );
}
