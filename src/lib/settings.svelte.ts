/**
 * What the pipeline is set up to do, remembered between runs.
 *
 * Everything in here is a choice the reader made once - the source to listen
 * to, the two models, the languages either side of the translation - and having
 * to make all of them again on every launch was the app forgetting who it was
 * working for. It lives in `localStorage` beside the interface preferences
 * rather than in a file the backend owns: it is read before the window is
 * drawn, and nothing on the Rust side needs it until Start is pressed.
 *
 * Stored as one object under one key, so a settings shape that grows a field
 * costs nothing and an old store simply lacks it - anything missing or
 * nonsensical falls back to the default rather than being trusted.
 */

export type Settings = {
  source: string;
  asrModel: string;
  /** The language recognition is told to expect. Empty means "work it out". */
  spokenLanguage: string;
  mtModel: string;
  targetLanguage: string;
  translateEnabled: boolean;
  showOriginal: boolean;
};

const KEY = "marswind.settings";

/**
 * A fresh install.
 *
 * Recognition is left to detect the language: the app is pointed at whatever
 * the machine happens to be playing, and guessing wrong there is worse than
 * spending a moment working it out.
 */
export const DEFAULTS: Settings = {
  source: "system",
  asrModel: "",
  spokenLanguage: "",
  mtModel: "",
  targetLanguage: "en",
  translateEnabled: true,
  showOriginal: true,
};

/**
 * The stored settings, with anything missing or of the wrong type defaulted.
 *
 * `targetLanguage` defaults to the language the interface is in, which on a
 * first run is the only thing the app knows about the person using it - someone
 * reading the window in Polish is more likely to want Polish subtitles than the
 * language whoever wrote this happened to type into the defaults.
 */
export function loadSettings(interfaceLanguage = DEFAULTS.targetLanguage): Settings {
  const defaults = { ...DEFAULTS, targetLanguage: interfaceLanguage };
  if (typeof localStorage === "undefined") return defaults;

  let stored: unknown;
  try {
    stored = JSON.parse(localStorage.getItem(KEY) ?? "null");
  } catch {
    // A store written by a broken build, or by hand. Nothing here is worth
    // failing a launch over.
    return defaults;
  }
  if (!stored || typeof stored !== "object") return defaults;

  const source = stored as Partial<Record<keyof Settings, unknown>>;
  const settings = { ...defaults };
  for (const key of Object.keys(DEFAULTS) as (keyof Settings)[]) {
    const value = source[key];
    if (typeof value === typeof DEFAULTS[key]) {
      // The key check above is what makes this safe; TypeScript cannot see it.
      (settings as Record<string, unknown>)[key] = value;
    }
  }
  return settings;
}

export function saveSettings(settings: Settings): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(KEY, JSON.stringify(settings));
}
