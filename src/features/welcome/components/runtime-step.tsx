import { useQuery } from "@tanstack/react-query";
import { runtimeQuery } from "@/features/hardware/api";
import { Step } from "./step";

export function RuntimeStep() {
  const runtime = useQuery(runtimeQuery);
  if (runtime.isPending) {
    return (
      <Step index={1} title="GenieX runtime" status="loading">
        Looking for GenieX…
      </Step>
    );
  }
  if (runtime.isError) {
    return (
      <Step index={1} title="GenieX runtime" status="problem">
        GenieX wasn't found. Calcine's installer sets it up; you can also install it from the GenieX
        releases page, then reopen Calcine.
      </Step>
    );
  }
  return (
    <Step index={1} title="GenieX runtime" status="done">
      GenieX {runtime.data.cliVersion} is ready, with QAIRT {runtime.data.qairtVersion ?? "—"} for
      the NPU and llama.cpp for GGUF models.
    </Step>
  );
}
