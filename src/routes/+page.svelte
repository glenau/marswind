<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import HistoryView from "$lib/HistoryView.svelte";
  import SettingsView from "$lib/SettingsView.svelte";
  import StatusBar from "$lib/StatusBar.svelte";
  import Toolbar, { type View } from "$lib/Toolbar.svelte";
  import Transcript from "$lib/Transcript.svelte";
  import { SPOKEN_LANGUAGES } from "$lib/i18n";
  import { createPreferences } from "$lib/preferences.svelte";
  import { loadSettings, saveSettings } from "$lib/settings.svelte";
  import {
    asrState,
    captureState,
    listAudioSources,
    listLanguages,
    listModels,
    startCapture,
    startRecognition,
    startSession,
    startTranslation,
    stopCapture,
    stopSession,
    translateState,
    type AsrState,
    type CaptureState,
    type Language,
    type LevelEvent,
    type ModelStatus,
    type SourceInfo,
    type TranslateState,
  } from "$lib/api";

  const preferences = createPreferences();
  const t = $derived(preferences.t);

  let sources = $state<SourceInfo[]>([]);
  let languages = $state<Language[]>([]);
  let models = $state<ModelStatus[]>([]);

  let capture = $state<CaptureState | null>(null);
  let asr = $state<AsrState | null>(null);
  let translation = $state<TranslateState | null>(null);

  /// Everything the pipeline is set up to do is read back from the last run.
  /// Nothing here is a first-run default unless there is no last run: choosing
  /// the source, the two models and the languages again on every launch was the
  /// app forgetting who it was working for.
  const saved = loadSettings(preferences.language);

  let source = $state(saved.source);
  let asrModel = $state(saved.asrModel);
  let spokenLanguage = $state(saved.spokenLanguage);
  let mtModel = $state(saved.mtModel);
  let targetLanguage = $state(saved.targetLanguage);
  let translateEnabled = $state(saved.translateEnabled);
  let showOriginal = $state(saved.showOriginal);

  // One place they are written back, rather than a `localStorage.setItem` at
  // every point one of them can change - several of them are also set by the
  // backend, when a model is chosen for the reader or a session is already
  // running, and those changes are worth keeping too.
  $effect(() => {
    saveSettings({
      source,
      asrModel,
      spokenLanguage,
      mtModel,
      targetLanguage,
      translateEnabled,
      showOriginal,
    });
  });

  /// One of three views fills the window. Settings and history used to float
  /// over the transcript in dialogs, which bought a second set of edges and a
  /// smaller area to put things in - there is nothing to see behind them.
  let view = $state<View>("transcript");
  let busy = $state(false);
  let error = $state("");
  /// The waterline as it is drawn: eased, so a loud syllable raises it rather
  /// than snapping it.
  let level = $state(0);
  /// The loudest thing heard lately, before easing. It jumps the moment a peak
  /// arrives and drains on the timer below; `level` chases it.
  let peak = 0;

  let unlisten: UnlistenFn[] = [];
  let timers: ReturnType<typeof setInterval>[] = [];
  let transcript: Transcript | undefined = $state();
  /// Kept up here because the Clear button lives on the status bar at the
  /// bottom of the window, outside the transcript it clears.
  let lineCount = $state(0);

  const capturing = $derived(capture?.running ?? false);
  const recognizing = $derived(asr?.running ?? false);
  const translating = $derived(translation?.running ?? false);
  const running = $derived(capturing);
  const targetName = $derived(languages.find((l) => l.code === targetLanguage)?.endonym ?? "");
  const sourceName = $derived(SPOKEN_LANGUAGES.find((l) => l.code === spokenLanguage)?.name ?? "");
  /// The transcript is always two columns while translation is switched on,
  /// including before the first translation has arrived - a layout that
  /// reorganises itself partway through the first sentence is worse than one
  /// with an empty column in it for a moment.
  const split = $derived(translateEnabled && showOriginal);

  const stages = $derived([
    {
      label: t("stage.audio"),
      running: capturing,
      detail: capture?.format
        ? `${(capture.format.sampleRate / 1000).toFixed(1)} kHz · ${capture.format.channels} ch · ${capture.droppedSamples} dropped`
        : t("stage.notCapturing"),
    },
    { label: t("stage.recognition"), running: recognizing, detail: asr?.modelId ?? t("stage.notRunning") },
    {
      label: t("stage.translation"),
      running: translating,
      detail: translation?.modelId ?? t("stage.notRunning"),
    },
  ]);

  onMount(async () => {
    // Each step is guarded: a backend call that fails should cost its own
    // panel, not the level meter and the polling that come after it.
    try {
      languages = await listLanguages();
      await Promise.all([refreshSources(), refreshModels(), refreshState()]);
      // Nothing is set up on a first run, so the settings are where to start.
      if (!asrModel) view = "settings";
    } catch (e) {
      error = String(e);
      view = "settings";
    }

    unlisten.push(
      await listen<LevelEvent>("audio://level", (event) => {
        peak = Math.max(peak, event.payload.peak);
        if (capture) capture.droppedSamples = event.payload.droppedSamples;
      }),
    );

    // The peak drains slowly, which is what makes a level meter readable rather
    // than a strobe light. What is drawn climbs towards it a fraction at a time
    // instead of arriving in one step - a syllable used to take the waterline
    // up the orb in a single frame, and a jump that size reads as a glitch
    // rather than as a loud word. Falling needs no easing: the drain is the
    // easing.
    timers.push(
      setInterval(() => {
        peak = Math.max(0, peak - 0.04);
        level = level < peak ? level + (peak - level) * 0.3 : peak;
      }, 50),
    );
    timers.push(setInterval(() => void refreshState(), 2000));
  });

  onDestroy(() => {
    unlisten.forEach((fn) => fn());
    timers.forEach(clearInterval);
  });

  async function refreshSources() {
    try {
      sources = await listAudioSources();
      if (!sources.some((s) => s.id === source)) source = sources[0]?.id ?? "system";
    } catch (e) {
      error = String(e);
    }
  }

  async function refreshModels() {
    models = await listModels();

    const installed = (kind: string) => models.filter((m) => m.kind === kind && m.installed);
    const pick = (kind: string, current: string) =>
      installed(kind).some((m) => m.id === current)
        ? current
        : (installed(kind).find((m) => m.recommended)?.id ?? installed(kind)[0]?.id ?? "");

    asrModel = pick("asr", asrModel);
    mtModel = pick("mt", mtModel);
  }

  async function refreshState() {
    capture = await captureState();
    asr = await asrState();
    translation = await translateState();

    if (capture.sourceId) source = capture.sourceId;
    if (asr.modelId) asrModel = asr.modelId;
    if (translation.modelId) mtModel = translation.modelId;
    if (translation.targetLanguage) targetLanguage = translation.targetLanguage;
  }

  /**
   * One button for the whole pipeline. Each stage needs the one before it, so
   * they start in order and any failure leaves a message rather than a
   * half-started app.
   */
  async function toggleRun() {
    error = "";
    busy = true;
    try {
      if (running) {
        await stopCapture();
        await stopSession().catch(() => {});
        level = 0;
        peak = 0;
      } else {
        // The transcript is deliberately not cleared here. Stopping and
        // starting again is how anyone checks a change, and losing what was on
        // screen every time made that impossible; there is a Clear button.
        await startCapture(source);
        await startRecognition(asrModel, spokenLanguage === "" ? null : spokenLanguage);
        if (translateEnabled && mtModel) {
          await startTranslation(mtModel, targetLanguage);
        }
        // Recorded from here so a session file exists even if the app is closed
        // without pressing Stop.
        await startSession({
          startedAt: new Date().toLocaleString(),
          source,
          asrModel,
          spokenLanguage,
          mtModel: translateEnabled ? mtModel : "",
          targetLanguage: translateEnabled ? targetLanguage : "",
        }).catch(() => {});
        view = "transcript";
      }
    } catch (e) {
      error = String(e);
      // Never leave capture running with nothing behind it.
      if (!running) {
        await stopCapture().catch(() => {});
        await stopSession().catch(() => {});
      }
    } finally {
      busy = false;
      await refreshState();
    }
  }
</script>

<div class="app">
  <Toolbar
    {level}
    {busy}
    {running}
    {view}
    {t}
    locale={preferences.language}
    onToggleRun={toggleRun}
    onShow={(next) => (view = next)}
  />

  {#if error}
    <div class="error" role="alert">
      <span>{error}</span>
      <button class="quiet" onclick={() => (error = "")}>{t("action.dismiss")}</button>
    </div>
  {/if}

  <!-- All three stay mounted: the transcript listens to the pipeline, and
       unmounting it while settings are open would lose everything captured
       while the user was reading them. -->
  <div class="view" hidden={view !== "transcript"}>
    <Transcript
      bind:this={transcript}
      {split}
      {sourceName}
      {targetName}
      {t}
      onLines={(count) => (lineCount = count)}
    />
  </div>

  {#if view === "history"}
    <div class="view"><HistoryView {t} /></div>
  {/if}

  {#if view === "settings"}
    <div class="view">
      <SettingsView
        {sources}
        {languages}
        {models}
        {preferences}
        {t}
        locked={running}
        bind:source
        bind:asrModel
        bind:spokenLanguage
        bind:mtModel
        bind:targetLanguage
        bind:translateEnabled
        bind:showOriginal
        onRefreshSources={refreshSources}
        onModelsChanged={refreshModels}
      />
    </div>
  {/if}

  <!-- The bottom edge, in every view: what is running, and - while there is a
       transcript to clear - the button that clears it. -->
  <StatusBar
    {stages}
    {t}
    canClear={view === "transcript" && lineCount > 0}
    onClear={() => transcript?.clear()}
  />
</div>

<style>
  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .view {
    flex: 1 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .view[hidden] {
    display: none;
  }

  .error {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--gutter);
    background: var(--error-bg);
    border-bottom: 1px solid var(--error-line);
    color: var(--error-ink);
    font-size: var(--text-md);
    line-height: 1.5;
  }

  .error button {
    margin-left: auto;
    color: var(--error-ink);
  }
</style>
