import { defineMessages } from "@/i18n";

export const messages = defineMessages({
  en: {
    title: "Battery saver",
    description:
      "On battery, models run in a power-saving mode so the laptop lasts longer. It applies to the chat and to apps; a request that picks its own power mode keeps it.",
    enable: "Save power on battery",
    enableHint: "Back to full speed as soon as it's plugged in.",
    mode: "Power mode on battery",
    modeHint: "Slower replies, less heat and drain. Power saver is a good trade-off.",
    onMains: "On mains",
    onBattery: "On battery ({percent}%)",
    onBatteryUnknown: "On battery",
    active: "Saving power now",
    badge: "Battery saver",
    badgeHint: "On battery: models run in “{mode}” mode. Change it in Settings.",
  },
  fr: {
    title: "Économie sur batterie",
    description:
      "Sur batterie, les modèles tournent dans un mode économe pour que le portable tienne plus longtemps. Ça vaut pour le chat et les applis ; une requête qui choisit son propre mode le garde.",
    enable: "Économiser sur batterie",
    enableHint: "Retour à pleine vitesse dès que le portable est branché.",
    mode: "Mode d'alimentation sur batterie",
    modeHint:
      "Réponses plus lentes, moins de chaleur et de consommation. Économie d'énergie est un bon compromis.",
    onMains: "Sur secteur",
    onBattery: "Sur batterie ({percent} %)",
    onBatteryUnknown: "Sur batterie",
    active: "Économie en cours",
    badge: "Économie batterie",
    badgeHint:
      "Sur batterie : les modèles tournent en mode « {mode} ». Modifiable dans les Réglages.",
  },
});
