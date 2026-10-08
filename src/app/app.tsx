import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { TooltipProvider } from "@/components/ui/tooltip";
import { CalcineError } from "@/lib/api";
import { useApplyTheme } from "@/stores/theme";
import { router } from "./router";

/** Errors that retrying can't fix. */
const PERMANENT_ERRORS = new Set(["not_in_tauri", "runtime_not_found", "not_implemented"]);

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 10_000,
      retry: (failureCount, error) =>
        !(error instanceof CalcineError && PERMANENT_ERRORS.has(error.kind)) && failureCount < 2,
    },
  },
});

export function App() {
  useApplyTheme();

  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider delayDuration={300}>
        <RouterProvider router={router} />
      </TooltipProvider>
    </QueryClientProvider>
  );
}
