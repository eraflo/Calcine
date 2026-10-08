import { useState } from "react";
import { Page } from "@/components/calcine/layout/page";
import { Tabs } from "@/components/ui/tabs";
import { useT } from "@/i18n";
import type { ModelReference } from "@/lib/api";
import { AddModelCard } from "./components/add-model-card";
import { AiHubCatalog } from "./components/catalog";
import { DownloadDialog } from "./components/download-dialog";
import { HuggingFaceSearch } from "./components/hugging-face-search";
import { messages } from "./messages";

type Source = "ai_hub" | "hugging_face";

export function DiscoverPage() {
  const t = useT(messages);
  const [source, setSource] = useState<Source>("ai_hub");
  const [download, setDownload] = useState<ModelReference | null>(null);

  return (
    <Page title={t("title")} description={t("description")}>
      <AddModelCard onDownload={setDownload} />
      <div className="flex flex-col gap-4">
        <Tabs
          label={t("title")}
          value={source}
          onChange={setSource}
          tabs={[
            { value: "ai_hub", label: t("tabAiHub") },
            { value: "hugging_face", label: t("tabHuggingFace") },
          ]}
        />
        {source === "ai_hub" ? <AiHubCatalog /> : <HuggingFaceSearch onDownload={setDownload} />}
      </div>
      <DownloadDialog reference={download} onOpenChange={(open) => !open && setDownload(null)} />
    </Page>
  );
}
