import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArrowRight, Check, CircleAlert, Download, Loader } from "lucide-react";
import type * as React from "react";
import { LogoMark } from "@/components/calcine/logo";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { catalogQuery } from "@/features/discover/api";
import { chipsetQuery, hardwareQuery, runtimeQuery } from "@/features/hardware/api";
import { useAvailability } from "@/features/library/availability";
import { usePullByName } from "@/features/tasks/api";
import { percent } from "@/features/tasks/format";
import { markOnboarded } from "@/stores/onboarding";

/** Small models that download quickly and run on the NPU: good first picks. */
const STARTER_MODELS = [
  { name: "qualcomm/Qwen3-0.6B", blurb: "Tiny and fast. About 725 MiB." },
  { name: "qualcomm/Llama-v3.2-1B-Instruct", blurb: "Meta's compact assistant." },
  { name: "qualcomm/Qwen3-4B", blurb: "Smarter answers. About 3 GiB." },
];

export function WelcomePage() {
  const navigate = useNavigate();
  const finish = () => {
    markOnboarded();
    void navigate({ to: "/library" });
  };

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8 px-8 py-12">
      <header className="flex flex-col items-start gap-4">
        <LogoMark className="size-10" />
        <div className="flex flex-col gap-1">
          <h1 className="text-2xl font-semibold tracking-tight">Welcome to Calcine</h1>
          <p className="text-sm text-muted-foreground">
            Run language models on your Snapdragon NPU, and share them with your other apps.
          </p>
        </div>
      </header>

      <ol className="flex flex-col gap-3">
        <RuntimeStep />
        <DeviceStep />
        <FirstModelStep />
      </ol>

      <div className="flex items-center justify-between">
        <Button variant="ghost" onClick={finish}>
          Skip for now
        </Button>
        <Button variant="default" size="lg" onClick={finish}>
          Open my library
          <ArrowRight />
        </Button>
      </div>
    </div>
  );
}

function Step({
  index,
  title,
  status,
  children,
}: {
  index: number;
  title: string;
  status: "todo" | "loading" | "done" | "problem";
  children: React.ReactNode;
}) {
  return (
    <li>
      <Card className="flex gap-4 p-4">
        <div className="flex size-7 shrink-0 items-center justify-center rounded-full border bg-background text-xs font-medium">
          {status === "done" ? (
            <Check className="size-4 text-success" />
          ) : status === "problem" ? (
            <CircleAlert className="size-4 text-warning" />
          ) : status === "loading" ? (
            <Loader className="size-4 animate-spin text-muted-foreground" />
          ) : (
            index
          )}
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <h2 className="text-sm font-medium">{title}</h2>
          <div className="text-sm text-muted-foreground">{children}</div>
        </div>
      </Card>
    </li>
  );
}

function RuntimeStep() {
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

function DeviceStep() {
  const chipset = useQuery(chipsetQuery);
  const hardware = useQuery(hardwareQuery);
  const npu = hardware.data?.accelerators.find((accelerator) => accelerator.unit === "npu");

  if (chipset.isPending || hardware.isPending) {
    return (
      <Step index={2} title="Your device" status="loading">
        Detecting your chipset and NPU…
      </Step>
    );
  }
  return (
    <Step index={2} title="Your device" status={npu ? "done" : "problem"}>
      {chipset.data ?? "Unknown chipset"}
      {npu ? ` · ${npu.name}` : " · no NPU detected, models will run on the GPU or CPU"}
    </Step>
  );
}

function FirstModelStep() {
  const catalog = useQuery(catalogQuery(false));
  const availability = useAvailability();
  const pull = usePullByName();

  const available = new Set(catalog.data?.models.map((model) => model.name));
  const starters = catalog.data
    ? STARTER_MODELS.filter((model) => available.has(model.name))
    : STARTER_MODELS;
  const hasModel = starters.some((model) => availability(model.name).status === "installed");

  return (
    <Step index={3} title="Download a first model" status={hasModel ? "done" : "todo"}>
      <p>These are compiled for your NPU. You can add any other model later from Discover.</p>
      <ul className="mt-3 flex flex-col gap-2">
        {starters.map((model) => {
          const state = availability(model.name);
          return (
            <li
              key={model.name}
              className="flex flex-col gap-2 rounded-md border bg-background/40 p-3"
            >
              <div className="flex items-center gap-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate font-mono text-[13px] text-foreground">{model.name}</p>
                  <p className="text-xs">{model.blurb}</p>
                </div>
                {state.status === "installed" ? (
                  <Badge tone="success">
                    <Check />
                    Ready
                  </Badge>
                ) : state.status === "downloading" ? (
                  <span className="text-xs tabular-nums">{percent(state.job) ?? 0}%</span>
                ) : (
                  <Button size="sm" onClick={() => pull.pull(model.name)} disabled={pull.isPending}>
                    <Download />
                    Download
                  </Button>
                )}
              </div>
              {state.status === "downloading" && (
                <Progress value={percent(state.job)} label={`Downloading ${model.name}`} />
              )}
            </li>
          );
        })}
      </ul>
    </Step>
  );
}
