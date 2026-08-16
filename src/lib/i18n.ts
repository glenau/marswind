/**
 * Interface language.
 *
 * A dictionary rather than a library: a few hundred strings across the
 * languages the app can already translate into, and the whole point of this app
 * is that it ships nothing it does not need. Every locale is a partial of
 * English - `t` falls back to it for anything a translation has not covered, so
 * a missing string is a missing translation rather than a blank label.
 *
 * The list is deliberately the same one the translator offers: a person reading
 * subtitles in Polish should be able to have the window in Polish too, and
 * every language here is one the app already knows the name of.
 */
import en, { type Dictionary, type Key } from "./locales/en";
import ru from "./locales/ru";
import de from "./locales/de";
import es from "./locales/es";
import fr from "./locales/fr";
import it from "./locales/it";
import pt from "./locales/pt";
import pl from "./locales/pl";
import tr from "./locales/tr";
import uk from "./locales/uk";
import zh from "./locales/zh";
import ja from "./locales/ja";
import ko from "./locales/ko";

export type Locale =
  | "en"
  | "ru"
  | "de"
  | "es"
  | "fr"
  | "it"
  | "pt"
  | "pl"
  | "tr"
  | "uk"
  | "zh"
  | "ja"
  | "ko";

const DICTIONARIES: Record<Locale, Dictionary> = {
  en,
  ru,
  de,
  es,
  fr,
  it,
  pt,
  pl,
  tr,
  uk,
  zh,
  ja,
  ko,
};

/** The interface languages, each in its own name. */
export const LOCALES: { code: Locale; name: string }[] = [
  { code: "en", name: "English" },
  { code: "ru", name: "Русский" },
  { code: "de", name: "Deutsch" },
  { code: "es", name: "Español" },
  { code: "fr", name: "Français" },
  { code: "it", name: "Italiano" },
  { code: "pt", name: "Português" },
  { code: "pl", name: "Polski" },
  { code: "tr", name: "Türkçe" },
  { code: "uk", name: "Українська" },
  { code: "zh", name: "中文" },
  { code: "ja", name: "日本語" },
  { code: "ko", name: "한국어" },
];

export const LOCALE_CODES = LOCALES.map((locale) => locale.code);

/**
 * Languages recognition can be told to expect, by their own name.
 *
 * The empty code is "work it out from the audio", and it is first because it is
 * the one that is right for a window playing something in a language the reader
 * did not choose - which is most of them.
 */
export const SPOKEN_LANGUAGES = [
  { code: "", name: "" },
  ...LOCALES.map(({ code, name }) => ({ code: code as string, name })),
];

export function phrases(locale: Locale, live: boolean): string[] {
  const dictionary = DICTIONARIES[locale] ?? en;
  return dictionary.phrases[live ? "live" : "idle"];
}

export function translator(locale: Locale) {
  const dictionary = DICTIONARIES[locale] ?? en;
  return (key: Key): string => dictionary.strings[key] ?? en.strings[key] ?? key;
}

export type Translate = ReturnType<typeof translator>;
export type { Key };
