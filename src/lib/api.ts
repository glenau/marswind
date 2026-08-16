/** Typed wrappers over the Rust commands, so the UI never spells one wrong. */
import { getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

/** Where the app says it came from. One place, so a fork only edits one line. */
export const HOMEPAGE = "https://github.com/glenau/marswind";
export const ISSUES = `${HOMEPAGE}/issues`;
export const LICENSE = `${HOMEPAGE}/blob/main/LICENSE`;
/** Everything the app is built out of, and what each piece is licensed under. */
export const NOTICES = `${HOMEPAGE}/blob/main/THIRD-PARTY-NOTICES.md`;

/** The version in `src-tauri/tauri.conf.json`, read from the running bundle. */
export const appVersion = () => getVersion();
export const runtimeVersion = () => getTauriVersion();

export type SourceKind = "system" | "process";

export type SourceInfo = {
  id: string;
  name: string;
  detail: string | null;
  kind: SourceKind;
  active: boolean;
};

export type CaptureFormat = {
  sampleRate: number;
  channels: number;
};

export type CaptureState = {
  running: boolean;
  sourceId: string | null;
  sourceName: string | null;
  format: CaptureFormat | null;
  droppedSamples: number;
  recording: boolean;
};

export type ModelKind = "asr" | "vad" | "mt";

export type ModelStatus = {
  id: string;
  name: string;
  note: string;
  kind: ModelKind;
  sizeBytes: number;
  installed: boolean;
  downloading: boolean;
  recommended: boolean;
  /** The terms the weights come under - not all of them are open source. */
  license: string;
  licenseUrl: string;
};

export type AsrState = {
  running: boolean;
  modelId: string | null;
  language: string | null;
};

export type Language = {
  code: string;
  name: string;
  endonym: string;
};

export type TranslateState = {
  running: boolean;
  engine: string | null;
  modelId: string | null;
  targetLanguage: string | null;
};

export type TranslationEvent = {
  line: number;
  /** Which piece of the row this is. A row is translated in segments as its
   *  words are committed, and they are appended in this order. */
  seq: number;
  source: string;
  text: string;
  translationMs: number;
  /** Time to the first word appearing, which is what the reader waits for. */
  firstWordMs: number;
};

/** A translation while it is still being written. Carries the text so far. */
export type TranslationPartialEvent = {
  line: number;
  seq: number;
  text: string;
};

export type LevelEvent = {
  peak: number;
  rms: number;
  droppedSamples: number;
};

export type ProgressEvent = {
  id: string;
  downloadedBytes: number;
  totalBytes: number;
  done: boolean;
  error: string | null;
};

export type TranscriptEvent = {
  line: number;
  /** How many translation segments this row was cut into. Only meaningful once
   *  the row is final, and it is what tells "still coming" from "never
   *  arriving". */
  segments: number;
  text: string;
  tentative: string;
  final: boolean;
  inferenceMs: number;
  windowSeconds: number;
};

/** A segment the translator was too far behind to take. */
export type SkippedEvent = {
  line: number;
  seq: number;
};

export type SessionSummary = {
  id: string;
  startedAt: string;
  durationSeconds: number;
  rows: number;
  words: number;
  asrModel: string;
  targetLanguage: string;
  translated: boolean;
};

export type SessionRow = {
  line: number;
  at: number;
  source: string;
  translation: string;
  recognitionMs: number;
  translationMs: number;
  skippedSegments: number;
};

export type Session = {
  id: string;
  startedAt: string;
  durationSeconds: number;
  source: string;
  asrModel: string;
  spokenLanguage: string;
  mtModel: string;
  targetLanguage: string;
  rows: SessionRow[];
};

export type ExportFormat = "text" | "srt" | "json";

export type SampleInfo = {
  id: string;
  name: string;
  note: string;
  /** The exact words spoken, so the transcript can be checked against them. */
  transcript: string;
};

export const listAudioSources = () => invoke<SourceInfo[]>("list_audio_sources");
export const startCapture = (sourceId: string) =>
  invoke<CaptureFormat>("start_capture", { sourceId });
export const stopCapture = () => invoke<void>("stop_capture");
// Stopping capture stops recognition and translation with it, in that order -
// there is deliberately no command for stopping half the pipeline. See
// `commands.rs`.
export const captureState = () => invoke<CaptureState>("capture_state");

export const listModels = () => invoke<ModelStatus[]>("list_models");
export const modelsDiskUsage = () => invoke<number>("models_disk_usage");
export const downloadModel = (modelId: string) => invoke<void>("download_model", { modelId });
export const cancelDownload = (modelId: string) => invoke<void>("cancel_download", { modelId });
export const removeModel = (modelId: string) => invoke<void>("remove_model", { modelId });

export const asrState = () => invoke<AsrState>("asr_state");
export const listLanguages = () => invoke<Language[]>("list_languages");
export const translateState = () => invoke<TranslateState>("translate_state");
export const startTranslation = (modelId: string, targetLanguage: string) =>
  invoke<void>("start_translation", { modelId, targetLanguage });
export const startRecognition = (modelId: string, language: string | null) =>
  invoke<void>("start_recognition", { modelId, language });

export const startSession = (meta: {
  startedAt: string;
  source: string;
  asrModel: string;
  spokenLanguage: string;
  mtModel: string;
  targetLanguage: string;
}) => invoke<string>("start_session", meta);
export const stopSession = () => invoke<string | null>("stop_session");
export const listSessions = () => invoke<SessionSummary[]>("list_sessions");
export const readSession = (id: string) => invoke<Session>("read_session", { id });
export const removeSession = (id: string) => invoke<void>("remove_session", { id });
/** Returns the file it wrote, which is what the History view reveals in Finder. */
export const exportSession = (id: string, format: ExportFormat) =>
  invoke<string>("export_session", { id, format });

/** A release newer than the one running, as `check_for_update` found it. */
export type UpdateInfo = {
  version: string;
  pageUrl: string;
  assetName: string;
  assetUrl: string;
  checksumUrl: string;
  sizeBytes: number;
};

export type UpdateProgressEvent = {
  downloadedBytes: number;
  totalBytes: number;
  done: boolean;
};

/** The app's only network request that is not a model download, and it happens
 *  on a button press. `null` means there is nothing newer. */
export const checkForUpdate = () => invoke<UpdateInfo | null>("check_for_update");
/** Downloads the image into Downloads, checksum verified, and returns its path. */
export const downloadUpdate = (info: UpdateInfo) => invoke<string>("download_update", { info });

export const listSamples = () => invoke<SampleInfo[]>("list_samples");
export const playSample = (id: string) => invoke<void>("play_sample", { id });
export const stopSample = () => invoke<void>("stop_sample");
export const playingSample = () => invoke<string | null>("playing_sample");

/** Seconds as m:ss, which is how long a session ever is. */
export function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

export function formatSize(bytes: number): string {
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  if (bytes >= 1024 ** 2) return `${Math.round(bytes / 1024 ** 2)} MB`;
  return `${Math.round(bytes / 1024)} KB`;
}
