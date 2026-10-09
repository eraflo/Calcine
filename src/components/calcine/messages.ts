import { defineMessages } from "@/i18n";

/** Strings of the shared Calcine components. */
export const messages = defineMessages({
  en: {
    // Runtime and type badges
    qairt: "QAIRT · NPU",
    qairtHint: "Pre-compiled Qualcomm AI Hub bundle. Runs natively on the Hexagon NPU.",
    llamaCpp: "llama.cpp",
    llamaCppHint: "GGUF model. Runs on the NPU, GPU or CPU.",
    unknownRuntime: "Unknown runtime",
    unknownRuntimeHint: "Runtime not recognized.",
    typeUnknown: "Type unknown",
    vision: "Vision",
    text: "Text",
    // Error states
    desktopOnly: "Open Calcine in the desktop app",
    desktopOnlyHint:
      "This page needs Calcine's backend, which isn't available in a regular browser. Run <code>bun run app</code> or <code>bun run app:mock</code>.",
    geniexMissing: "GenieX isn't installed",
    geniexMissingHint:
      "Calcine runs models with GenieX, Qualcomm's runtime. Install it from the <strong>welcome screen</strong>.",
    setUpGeniex: "Set up GenieX",
    somethingWrong: "Something went wrong",
  },
  fr: {
    qairt: "QAIRT · NPU",
    qairtHint: "Bundle Qualcomm AI Hub précompilé. Tourne nativement sur le NPU Hexagon.",
    llamaCpp: "llama.cpp",
    llamaCppHint: "Modèle GGUF. Tourne sur le NPU, le GPU ou le CPU.",
    unknownRuntime: "Runtime inconnu",
    unknownRuntimeHint: "Runtime non reconnu.",
    typeUnknown: "Type inconnu",
    vision: "Vision",
    text: "Texte",
    desktopOnly: "Ouvrez Calcine dans l'application de bureau",
    desktopOnlyHint:
      "Cette page a besoin du backend de Calcine, indisponible dans un navigateur classique. Lancez <code>bun run app</code> ou <code>bun run app:mock</code>.",
    geniexMissing: "GenieX n'est pas installé",
    geniexMissingHint:
      "Calcine exécute les modèles avec GenieX, le moteur de Qualcomm. Installez-le depuis l'<strong>écran d'accueil</strong>.",
    setUpGeniex: "Installer GenieX",
    somethingWrong: "Une erreur est survenue",
  },
});
