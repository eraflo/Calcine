import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useApplyTheme } from "@/stores/theme";
import { QuickAskWindow } from "./components/quick-ask-window";

const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 10_000, retry: 1 } },
});

/** What the quick question window shows, instead of Calcine's app. */
export function QuickAskApp() {
  useApplyTheme();
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider delayDuration={300}>
        <QuickAskWindow />
      </TooltipProvider>
    </QueryClientProvider>
  );
}
