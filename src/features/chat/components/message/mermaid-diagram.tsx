import { Code, Workflow } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Button } from "@/components/ui/button";
import { useT } from "@/i18n";
import { useTheme } from "@/stores/theme";
import { messages } from "../../messages";

/** A diagram is drawn once its code stops changing (it's streamed). */
const SETTLE_MS = 400;

/** A Mermaid code block, drawn. Mermaid is loaded the first time one shows. */
export function MermaidDiagram({ code }: { code: string }) {
  const t = useT(messages);
  const dark = useDark();
  const id = `mermaid-${useId().replace(/[^\w-]/g, "")}`;
  const [svg, setSvg] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  const [showCode, setShowCode] = useState(false);
  const drawing = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(async () => {
      try {
        const mermaid = (await import("mermaid")).default;
        mermaid.initialize({
          startOnLoad: false,
          securityLevel: "strict",
          theme: dark ? "dark" : "neutral",
          fontFamily: "inherit",
        });
        const valid = await mermaid.parse(code, { suppressErrors: true });
        if (!valid) throw new Error("invalid diagram");
        const rendered = await mermaid.render(id, code);
        if (!cancelled) {
          setSvg(rendered.svg);
          setFailed(false);
        }
      } catch {
        // Mermaid leaves its error drawing behind.
        document.getElementById(`d${id}`)?.remove();
        if (!cancelled) setFailed(true);
      }
    }, SETTLE_MS);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [code, dark, id]);

  // Mermaid's own output, sanitized by it (securityLevel "strict").
  useEffect(() => {
    if (drawing.current) drawing.current.innerHTML = svg ?? "";
  }, [svg]);

  const drawn = svg !== null && !failed && !showCode;
  return (
    <div className="group/diagram relative rounded-md border bg-card">
      <div className="absolute top-1.5 right-1.5 z-10 flex gap-0.5 opacity-0 transition-opacity group-hover/diagram:opacity-100 focus-within:opacity-100">
        {svg !== null && !failed && (
          <Button
            variant="ghost"
            size="icon"
            className="size-7 text-muted-foreground"
            onClick={() => setShowCode((shown) => !shown)}
            aria-label={showCode ? t("diagramShow") : t("diagramCode")}
            title={showCode ? t("diagramShow") : t("diagramCode")}
          >
            {showCode ? <Workflow /> : <Code />}
          </Button>
        )}
        <CopyButton value={code} className="size-7" />
      </div>
      <div
        ref={drawing}
        hidden={!drawn}
        className="flex justify-center overflow-x-auto p-4 [&_svg]:h-auto [&_svg]:max-w-full"
      />
      {!drawn && (
        <>
          <pre className="m-0 overflow-x-auto border-0 bg-transparent p-3 font-mono text-xs">
            {code}
          </pre>
          {failed && (
            <p className="border-t px-3 py-1.5 text-[11px] text-muted-foreground">
              {t("diagramFailed")}
            </p>
          )}
        </>
      )}
    </div>
  );
}

/** Whether the app shows dark right now. */
function useDark(): boolean {
  const theme = useTheme((state) => state.theme);
  if (theme !== "system") return theme === "dark";
  return !window.matchMedia("(prefers-color-scheme: light)").matches;
}
