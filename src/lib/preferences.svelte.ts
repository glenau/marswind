/**
 * How the interface itself is set up - language, theme and text size.
 *
 * Kept apart from the pipeline settings because it is a different kind of
 * thing: it changes nothing about what the app does, only how it looks, and it
 * has to survive a restart. Written to `localStorage` rather than the settings
 * file so the window can be laid out before any backend call returns.
 *
 * The size is a multiplier on the root font size, and every dimension in the
 * app - text, control heights, padding, radii - is expressed in `rem`. One
 * number therefore scales the whole interface rather than only its text, which
 * is what "make it bigger" actually means.
 *
 * The theme is a single attribute on the root element. Every colour in the app
 * is a custom property, and the light theme is those properties with different
 * values in them - no component knows which theme it is being drawn in.
 */
import { LOCALE_CODES, translator, type Locale } from "./i18n";

export const SIZES = [
  { id: "small", scale: 0.9 },
  { id: "medium", scale: 1 },
  { id: "large", scale: 1.15 },
  { id: "huge", scale: 1.3 },
] as const;

export type SizeId = (typeof SIZES)[number]["id"];

/** Dark, light, or whichever of the two macOS is currently in. */
export const THEMES = ["dark", "light", "system"] as const;

export type ThemeId = (typeof THEMES)[number];

const LANGUAGE_KEY = "marswind.ui.language";
const SIZE_KEY = "marswind.ui.size";
const THEME_KEY = "marswind.ui.theme";

/** What the app opens in before anyone has chosen anything. */
const DEFAULT_LANGUAGE: Locale = "en";
const DEFAULT_THEME: ThemeId = "dark";

function stored<T extends string>(key: string, allowed: readonly T[], fallback: T): T {
  if (typeof localStorage === "undefined") return fallback;
  const value = localStorage.getItem(key) as T | null;
  return value && allowed.includes(value) ? value : fallback;
}

export function createPreferences() {
  let language = $state<Locale>(stored(LANGUAGE_KEY, LOCALE_CODES, DEFAULT_LANGUAGE));
  let theme = $state<ThemeId>(stored(THEME_KEY, THEMES, DEFAULT_THEME));
  let size = $state<SizeId>(
    stored(
      SIZE_KEY,
      SIZES.map((s) => s.id),
      "medium",
    ),
  );

  const scale = $derived(SIZES.find((s) => s.id === size)?.scale ?? 1);

  /// Only consulted while the theme is "system", but watched from the start:
  /// the setting can be switched to it at any point, and a listener attached
  /// then would not know what the system was already in.
  let systemDark = $state(
    typeof matchMedia === "undefined" ? true : matchMedia("(prefers-color-scheme: dark)").matches,
  );

  $effect(() => {
    if (typeof matchMedia === "undefined") return;
    const query = matchMedia("(prefers-color-scheme: dark)");
    const update = () => (systemDark = query.matches);
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  });

  /// Which of the two is actually on screen. "system" is a rule, not a colour.
  const resolvedTheme = $derived<Exclude<ThemeId, "system">>(
    theme === "system" ? (systemDark ? "dark" : "light") : theme,
  );

  // The root font size is the one place the multiplier is applied; everything
  // else is in rem and follows. The theme is one attribute for the same reason:
  // every colour in the app reads from the custom properties it selects.
  $effect(() => {
    document.documentElement.style.setProperty("--ui-scale", String(scale));
  });

  $effect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  });

  return {
    get language() {
      return language;
    },
    set language(value: Locale) {
      language = value;
      localStorage.setItem(LANGUAGE_KEY, value);
    },
    get theme() {
      return theme;
    },
    set theme(value: ThemeId) {
      theme = value;
      localStorage.setItem(THEME_KEY, value);
    },
    get size() {
      return size;
    },
    set size(value: SizeId) {
      size = value;
      localStorage.setItem(SIZE_KEY, value);
    },
    get t() {
      return translator(language);
    },
  };
}

export type Preferences = ReturnType<typeof createPreferences>;
