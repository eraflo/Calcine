/**
 * Translations.
 *
 * Each feature keeps its strings next to its code in a `messages.ts`:
 *
 * ```ts
 * export const messages = defineMessages({
 *   en: { title: "Library", models_one: "{count} model", models_other: "{count} models" },
 *   fr: { title: "Bibliothèque", models_one: "{count} modèle", models_other: "{count} modèles" },
 * });
 * ```
 *
 * English is the reference: French must define the same keys (checked by
 * TypeScript). In components, `const t = useT(messages)` then:
 *
 * - `t("title")`, with `{name}` placeholders: `t("greeting", { name })`
 * - `t.plural("models", count)` picks `models_one` or `models_other`
 * - `t.rich("hint", { code: (text) => <code>{text}</code> })` for `<code>…</code>` in a string
 */

import { Fragment, type ReactNode, useMemo } from "react";
import { type Language, useLanguage } from "./store";

export { getLanguage, type Language, type LanguagePreference, useLanguage } from "./store";

type Messages = Record<string, string>;

/** Declare a feature's strings. French must have every English key. */
export function defineMessages<const E extends Messages>(messages: {
  en: E;
  fr: { [K in keyof E]: string };
}) {
  return messages;
}

type Vars = Record<string, string | number>;

/** `models` for a dictionary with `models_one` and `models_other`. */
type PluralKey<E> = {
  [K in keyof E]: K extends `${infer Base}_one`
    ? `${Base}_other` extends keyof E
      ? Base
      : never
    : never;
}[keyof E];

export type Translate<E extends Messages> = {
  (key: keyof E & string, vars?: Vars): string;
  plural: (key: PluralKey<E> & string, count: number, vars?: Vars) => string;
  rich: (
    key: keyof E & string,
    tags: Record<string, (text: string) => ReactNode>,
    vars?: Vars,
  ) => ReactNode;
};

/** The translator for `messages` in the current language. */
export function useT<E extends Messages>(messages: { en: E; fr: { [K in keyof E]: string } }) {
  const language = useLanguage((state) => state.language);
  return useMemo(() => translator(messages, language), [messages, language]);
}

/** The translator outside React (event handlers, query functions). */
export function translator<E extends Messages>(
  messages: { en: E; fr: { [K in keyof E]: string } },
  language: Language,
): Translate<E> {
  const lookup = (key: string): string =>
    (messages[language] as Messages)[key] ?? (messages.en as Messages)[key] ?? key;
  const t = ((key: string, vars?: Vars) =>
    interpolate(lookup(key), vars, language)) as Translate<E>;
  t.plural = (key, count, vars) => {
    const category = new Intl.PluralRules(language).select(count) === "one" ? "one" : "other";
    return interpolate(lookup(`${key}_${category}`), { count, ...vars }, language);
  };
  t.rich = (key, tags, vars) => rich(interpolate(lookup(key), vars, language), tags);
  return t;
}

function interpolate(text: string, vars: Vars | undefined, language: Language) {
  if (!vars) return text;
  return text.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = vars[name];
    if (value === undefined) return match;
    return typeof value === "number" ? value.toLocaleString(language) : value;
  });
}

/** Replace `<tag>text</tag>` with `tags.tag(text)`. Tags don't nest. */
export function rich(text: string, tags: Record<string, (text: string) => ReactNode>): ReactNode {
  const parts: ReactNode[] = [];
  const pattern = /<(\w+)>(.*?)<\/\1>/g;
  let last = 0;
  for (const match of text.matchAll(pattern)) {
    const [whole, tag = "", inner = ""] = match;
    const index = match.index ?? 0;
    if (index > last) parts.push(text.slice(last, index));
    const render = tags[tag];
    parts.push(<Fragment key={index}>{render ? render(inner) : inner}</Fragment>);
    last = index + whole.length;
  }
  if (last < text.length) parts.push(text.slice(last));
  return parts;
}
