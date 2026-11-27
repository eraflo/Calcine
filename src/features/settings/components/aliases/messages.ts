import { defineMessages } from "@/i18n";

export const messages = defineMessages({
  en: {
    title: "Model aliases",
    description:
      "Some apps only offer OpenAI or Ollama model names. Give one of those names to a model on this PC, and it answers in its place.",
    empty: "No aliases yet.",
    alias: "Name apps ask for",
    aliasPlaceholder: "gpt-4o-mini",
    model: "Model that answers",
    pickModel: "Choose a model",
    noModels: "Download a model first.",
    add: "Add",
    remove: "Remove {alias}",
    suggestions: "Common names:",
    missing: "not downloaded",
    answeredBy: "answered by",
  },
  fr: {
    title: "Alias de modèles",
    description:
      "Certaines applis ne proposent que des noms de modèles OpenAI ou Ollama. Donnez un de ces noms à un modèle de ce PC : il répondra à sa place.",
    empty: "Aucun alias pour l'instant.",
    alias: "Nom demandé par les applis",
    aliasPlaceholder: "gpt-4o-mini",
    model: "Modèle qui répond",
    pickModel: "Choisir un modèle",
    noModels: "Téléchargez d'abord un modèle.",
    add: "Ajouter",
    remove: "Retirer {alias}",
    suggestions: "Noms courants :",
    missing: "non téléchargé",
    answeredBy: "répondu par",
  },
});
