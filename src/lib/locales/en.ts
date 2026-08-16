/**
 * English - the source of truth.
 *
 * Every other locale is a partial of this one: `t` falls back here for anything
 * a translation has not covered, so a missing string is a missing translation
 * rather than a blank label. The keys are declared here and nowhere else, which
 * is what makes a typo in one of them a type error instead of a label that says
 * `settings.langauge`.
 */

export const strings = {
  // Toolbar
  "stage.audio": "Audio",
  "stage.recognition": "Recognition",
  "stage.translation": "Translation",
  "stage.notCapturing": "Not capturing",
  "stage.notRunning": "Not running",
  "meter.level": "Input level",
  "action.start": "Start listening",
  "action.stop": "Stop",
  "action.working": "Working…",
  "action.history": "History",
  "action.settings": "Settings",
  "action.clear": "Clear",
  "action.dismiss": "Dismiss",
  "action.refresh": "Refresh",
  "action.delete": "Delete",
  "action.play": "Play",
  "action.text": "Text",
  "action.hideText": "Hide text",
  "action.install": "Install",
  "action.remove": "Remove",
  "action.cancel": "Cancel",
  "action.github": "The project on GitHub",

  // Transcript
  "transcript.original": "Original",
  "transcript.translation": "Translation",
  "transcript.into": "Translated into",
  "transcript.toLanguage": "to",
  "transcript.emptySource": "Press Start listening, then play something with speech - or play one of the sample clips in Settings. What is said appears here.",
  "transcript.emptyTarget": "The translation appears here, a clause at a time, while the speaker is still talking.",
  "transcript.translating": "translating…",
  "transcript.notTranslated": "- not translated",
  "transcript.toBottom": "Follow the newest line",

  // Settings
  "settings.general": "General",
  "settings.models": "Models",
  "settings.about": "About",
  "settings.audioTitle": "Audio",
  "settings.audioNote":
    "Where the sound comes from. Either everything this Mac plays, or one app on its own.",
  "settings.audio": "Audio source",
  "settings.recognition": "Speech recognition",
  "settings.recognitionNote":
    "Turns what the machine is playing into text. Needs a recognition model.",
  "settings.translation": "Translation",
  "settings.translationNote":
    "Turns the recognized text into another language. Needs a translation model - a different model from the one above.",
  "settings.on": "On",
  "settings.showOriginal": "Show the original beside the translation",
  "settings.locked": "Stop listening to change these.",
  "settings.noModel": "No model installed",
  "settings.needVad": "Install the voice activity model below - recognition needs it.",
  "settings.needAsr": "Install a recognition model below.",
  "settings.needMt": "Install a translation model below.",
  "settings.detect": "Detect automatically",
  "settings.model": "Model",
  "settings.interface": "Interface",
  "settings.interfaceNote":
    "How the window itself looks. None of these changes what the app does with sound.",
  "settings.language": "Language",
  "settings.theme": "Theme",
  "settings.textSize": "Text size",
  "settings.samples": "Try it without finding a video",
  "settings.samplesNote":
    "Plays a recorded clip through the speakers, so it goes through the same capture path as anything else. Start listening first.",

  // Theme
  "theme.dark": "Dark",
  "theme.light": "Light",
  "theme.system": "Follow the system",

  // About
  "about.title": "About",
  "about.tagline":
    "Live subtitles and translation for whatever this computer is playing. Recognition and translation both run here - no account, no API key, and nothing leaves the machine.",
  "about.version": "Version",
  "about.runtime": "Runtime",
  "about.license": "License",
  "about.source": "Source code",
  "about.issues": "Report a problem",
  "about.licenseFile": "Read the license",
  "about.notices": "Third-party licenses",
  "about.built": "Built with",
  "about.how": "How it works",
  "about.howCapture":
    "The sound is taken from macOS itself, through a Core Audio process tap: the audio your Mac is already playing, either all of it or a single application. No virtual audio driver, no microphone, and nothing about what you hear changes.",
  "about.howPipeline":
    "That stream is mixed down to mono at 16 kHz and stays in memory. Silero VAD cuts it into phrases, whisper.cpp turns them into text on the GPU, and llama.cpp translates in a separate process while the speaker is still talking. The audio itself is never written to disk and never leaves this Mac.",

  // Models
  "models.title": "Models",
  "models.note":
    "Everything runs on this machine, so every model is a file on this disk. Install one for recognition and one for translation.",
  "models.onDisk": "on disk",
  "models.forRecognition": "recognition",
  "models.forTranslation": "translation",
  "models.forVad": "voice detection",
  "models.recommended": "recommended",
  "models.required": "required",
  "models.installed": "installed",
  "models.of": "of",

  // History
  "history.empty":
    "No sessions yet. Every time you press Start listening, what is recognized and translated is written here.",
  "history.pick": "Pick a session on the left.",
  "history.loading": "Loading…",
  "history.rows": "rows",
  "history.words": "words",
  "history.exportText": "Export text",
  "history.exportSrt": "Export subtitles",
  "history.exportJson": "Export JSON",
  "history.saved": "Saved to",
  "history.noModel": "no recognition model",
  "history.notTranslated": "not translated",
  "history.segment": "segment",
  "history.segments": "segments",

  // Sizes
  "size.small": "Small",
  "size.medium": "Medium",
  "size.large": "Large",
  "size.huge": "Extra large",
};

/** Every string the interface can ask for. */
export type Key = keyof typeof strings;

/**
 * The line beside the level orb.
 *
 * Kept out of the dictionary deliberately: these are a set rather than a string
 * each, the code picks one at random and nothing ever asks for "the third idle
 * phrase". Two moods - waiting, and listening, where the app is allowed to be
 * funny about the fact that it is guessing at words.
 */
export type Phrases = { idle: string[]; live: string[] };

/** A locale: as much of the dictionary as it has, and its own phrases. */
export type Dictionary = {
  strings: Partial<Record<Key, string>>;
  phrases: Phrases;
};

export const phrases: Phrases = {
  idle: [
    "Hello. Shall we translate something?",
    "Play anything with talking in it",
    "Ready when you are",
    "Quiet in here. I'm listening anyway",
    "One press and the words start arriving",
    "Nothing you play ever leaves this Mac",
    "What are we watching today?",
    "A podcast, a lecture, a call - all the same to me",
    "Waiting for the first sentence",
    "Press Start listening and we're off",
  ],
  live: [
    "Listening closely…",
    "Trying to make out the words…",
    "Catching phrases mid-air…",
    "That was definitely a word…",
    "Flipping through my dictionary…",
    "Keeping up, mostly…",
    "Translating faster than I can think…",
    "All ears, no distractions…",
    "Almost have that sentence…",
    "Following along…",
  ],
};

const en: Dictionary = { strings, phrases };
export default en;
