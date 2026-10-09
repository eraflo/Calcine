import { defineMessages } from "@/i18n";

export const messages = defineMessages({
  en: {
    title: "Welcome to Calcine",
    intro: "Run language models on your Snapdragon NPU, and share them with your other apps.",
    skip: "Skip for now",
    openLibrary: "Open my library",
    // Runtime step
    runtimeTitle: "GenieX runtime",
    runtimeLooking: "Looking for GenieX…",
    runtimeMissing: "Calcine runs models with GenieX, Qualcomm's runtime. It isn't installed yet.",
    installRuntime: "Install GenieX {version} ({size})",
    installRuntimeHint: "Downloaded from Qualcomm and checked before installing.",
    runtimeReady:
      "GenieX {cli} is ready, with QAIRT {qairt} for the NPU and llama.cpp for GGUF models.",
    // Device step
    deviceTitle: "Your device",
    deviceDetecting: "Detecting your chipset and NPU…",
    unknownChipset: "Unknown chipset",
    noNpu: "no NPU detected, models will run on the GPU or CPU",
    // First model step
    firstModelTitle: "Download a first model",
    firstModelHint:
      "These are compiled for your NPU. You can add any other model later from Discover.",
    ready: "Ready",
    downloadingModel: "Downloading {name}",
    starterQwenTiny: "Tiny and fast. About 725 MiB.",
    starterLlama: "Meta's compact assistant.",
    starterQwenSmart: "Smarter answers. About 3 GiB.",
  },
  fr: {
    title: "Bienvenue dans Calcine",
    intro:
      "Exécutez des modèles de langage sur le NPU de votre Snapdragon et partagez-les avec vos autres applications.",
    skip: "Passer pour l'instant",
    openLibrary: "Ouvrir ma bibliothèque",
    // Runtime step
    runtimeTitle: "Moteur GenieX",
    runtimeLooking: "Recherche de GenieX…",
    runtimeMissing:
      "Calcine exécute les modèles avec GenieX, le moteur de Qualcomm. Il n'est pas encore installé.",
    installRuntime: "Installer GenieX {version} ({size})",
    installRuntimeHint: "Téléchargé chez Qualcomm et vérifié avant l'installation.",
    runtimeReady:
      "GenieX {cli} est prêt, avec QAIRT {qairt} pour le NPU et llama.cpp pour les modèles GGUF.",
    // Device step
    deviceTitle: "Votre appareil",
    deviceDetecting: "Détection de votre puce et de votre NPU…",
    unknownChipset: "Puce inconnue",
    noNpu: "aucun NPU détecté, les modèles tourneront sur le GPU ou le CPU",
    // First model step
    firstModelTitle: "Téléchargez un premier modèle",
    firstModelHint:
      "Ces modèles sont compilés pour votre NPU. Vous pourrez en ajouter d'autres plus tard depuis « Découvrir ».",
    ready: "Prêt",
    downloadingModel: "Téléchargement de {name}",
    starterQwenTiny: "Petit et rapide. Environ 725 Mo.",
    starterLlama: "L'assistant compact de Meta.",
    starterQwenSmart: "Des réponses plus fines. Environ 3 Go.",
  },
});
