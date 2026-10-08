import { useNavigate } from "@tanstack/react-router";
import { ArrowRight } from "lucide-react";
import { LogoMark } from "@/components/calcine/brand/logo";
import { Button } from "@/components/ui/button";
import { markOnboarded } from "@/stores/onboarding";
import { DeviceStep } from "./components/device-step";
import { FirstModelStep } from "./components/first-model-step";
import { RuntimeStep } from "./components/runtime-step";

/** First-launch checklist: runtime, device, first model. */
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
