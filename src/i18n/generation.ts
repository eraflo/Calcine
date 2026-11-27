import { defineMessages } from ".";

/** Generation options named the same in the chat and the benchmark:
 * power modes, speculative decoding. */
export const generationMessages = defineMessages({
  en: {
    powerMode: "Power mode",
    speculative: "Speculative decoding",
    spec_off: "Off",
    "spec_ngram-cache": "Repeated text (n-gram cache)",
    "spec_ngram-simple": "N-gram, simple",
    "spec_ngram-map-k": "N-gram map",
    "spec_ngram-map-k4v": "N-gram map (4 values)",
    "spec_ngram-mod": "N-gram, modified",
    "spec_draft-simple": "Draft model",
    "spec_draft-eagle3": "EAGLE-3 draft model",
    "spec_draft-mtp": "Multi-token prediction (MTP)",
    draftModel: "Draft model",
    draftModelHint:
      "A smaller llama.cpp model with the same vocabulary, e.g. a 0.6B for a 4B of the same family.",
    draftModelNone: "Pick a draft model",
    noDraftModels: "Download a smaller llama.cpp model of the same family to use as a draft.",
    power_burst: "Burst (fastest)",
    power_sustained_high_performance: "Sustained high performance",
    power_high_performance: "High performance",
    power_balanced: "Balanced",
    power_low_balanced: "Low balanced",
    power_high_power_saver: "High power saver",
    power_power_saver: "Power saver",
    power_low_power_saver: "Low power saver (coolest)",
  },
  fr: {
    powerMode: "Mode d'alimentation",
    speculative: "Décodage spéculatif",
    spec_off: "Désactivé",
    "spec_ngram-cache": "Texte répété (cache n-gram)",
    "spec_ngram-simple": "N-gram, simple",
    "spec_ngram-map-k": "Table de n-grams",
    "spec_ngram-map-k4v": "Table de n-grams (4 valeurs)",
    "spec_ngram-mod": "N-gram, modifié",
    "spec_draft-simple": "Modèle brouillon",
    "spec_draft-eagle3": "Modèle brouillon EAGLE-3",
    "spec_draft-mtp": "Prédiction multi-token (MTP)",
    draftModel: "Modèle brouillon",
    draftModelHint:
      "Un modèle llama.cpp plus petit avec le même vocabulaire, par exemple un 0,6B pour un 4B de la même famille.",
    draftModelNone: "Choisir un modèle brouillon",
    noDraftModels:
      "Téléchargez un modèle llama.cpp plus petit de la même famille pour l'utiliser comme brouillon.",
    power_burst: "Burst (le plus rapide)",
    power_sustained_high_performance: "Haute performance soutenue",
    power_high_performance: "Haute performance",
    power_balanced: "Équilibré",
    power_low_balanced: "Équilibré bas",
    power_high_power_saver: "Économie d'énergie haute",
    power_power_saver: "Économie d'énergie",
    power_low_power_saver: "Économie d'énergie basse (le plus frais)",
  },
});
