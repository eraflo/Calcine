import { defineMessages } from "@/i18n";

export const messages = defineMessages({
  en: {
    title: "Battery saver",
    description:
      "On battery, models write less so the laptop lasts longer: each token costs about the same energy, so fewer tokens use less battery. It applies to the chat and to apps.",
    enable: "Save battery",
    enableHint: "Back to normal as soon as it's plugged in.",
    noThinking: "Turn thinking off",
    noThinkingHint: "Reasoning is often most of a reply's tokens.",
    replyLimit: "Longest reply",
    replyLimitHint: "Longer replies are cut there.",
    tokens: "{count} tokens",
    noLimit: "No limit",
    mode: "Power mode",
    modeHint:
      "No effect for now: GenieX 0.12 runs at the same speed and draw in every mode (measured). Kept for later versions.",
    onMains: "On mains",
    onBattery: "On battery ({percent}%)",
    onBatteryUnknown: "On battery",
    active: "Saving now",
    badge: "Battery saver",
    badgeThinking: "thinking off",
    badgeLimit: "replies up to {count} tokens",
    badgeHint: "On battery: {changes}. Change it in Settings.",
  },
  fr: {
    title: "Économie sur batterie",
    description:
      "Sur batterie, les modèles écrivent moins pour que le portable tienne plus longtemps : chaque token coûte à peu près la même énergie, donc moins de tokens, c'est moins de batterie. Ça vaut pour le chat et les applis.",
    enable: "Économiser la batterie",
    enableHint: "Retour à la normale dès que le portable est branché.",
    noThinking: "Couper la réflexion",
    noThinkingHint: "La réflexion fait souvent l'essentiel des tokens d'une réponse.",
    replyLimit: "Longueur maximale des réponses",
    replyLimitHint: "Les réponses plus longues sont coupées là.",
    tokens: "{count} tokens",
    noLimit: "Sans limite",
    mode: "Mode d'alimentation",
    modeHint:
      "Sans effet pour l'instant : GenieX 0.12 tourne à la même vitesse et la même puissance dans tous les modes (mesuré). Gardé pour les prochaines versions.",
    onMains: "Sur secteur",
    onBattery: "Sur batterie ({percent} %)",
    onBatteryUnknown: "Sur batterie",
    active: "Économie en cours",
    badge: "Économie batterie",
    badgeThinking: "réflexion coupée",
    badgeLimit: "réponses de {count} tokens au plus",
    badgeHint: "Sur batterie : {changes}. Modifiable dans les Réglages.",
  },
});
