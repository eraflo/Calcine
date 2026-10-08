import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ModelTypeBadge, RuntimeBadge } from "./runtime-badge";

describe("RuntimeBadge", () => {
  it("labels QAIRT models as NPU-native", () => {
    render(
      <TooltipProvider>
        <RuntimeBadge runtime="qairt" />
      </TooltipProvider>,
    );
    expect(screen.getByText("QAIRT · NPU")).toBeInTheDocument();
  });

  it("labels GGUF models as llama.cpp", () => {
    render(
      <TooltipProvider>
        <RuntimeBadge runtime="llama_cpp" />
      </TooltipProvider>,
    );
    expect(screen.getByText("llama.cpp")).toBeInTheDocument();
  });
});

describe("ModelTypeBadge", () => {
  it("shows vision models as such", () => {
    render(<ModelTypeBadge type="vlm" />);
    expect(screen.getByText("Vision")).toBeInTheDocument();
  });
});
