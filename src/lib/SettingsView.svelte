<script lang="ts">
  /**
   * Settings, as a view filling the window rather than a dialog floating over
   * it. There is nothing to see behind it while it is open, so a modal only
   * added a second set of edges and a smaller area to put things in.
   *
   * It is four sections rather than one long column. Everything that decides
   * what the app does - the source, the two models it runs, the clips to try it
   * with - is in the first; the catalogue, the look of the window and the
   * credits each get their own, because none of them is something you visit
   * twice. Scrolling past the whole model catalogue to reach the text size was
   * the thing this replaces.
   *
   * Every section inside is still the same shape: a title, one line saying what
   * it is for, then the controls.
   */
  import AboutCard from "./AboutCard.svelte";
  import ModelsCard from "./ModelsCard.svelte";
  import SamplesCard from "./SamplesCard.svelte";
  import ViewHeader from "./ViewHeader.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import { LOCALES, SPOKEN_LANGUAGES, type Locale, type Translate } from "./i18n";
  import { SIZES, THEMES, type Preferences, type SizeId, type ThemeId } from "./preferences.svelte";
  import type { Language, ModelStatus, SourceInfo } from "./api";

  let {
    sources,
    languages,
    models,
    locked,
    preferences,
    t,
    source = $bindable(),
    asrModel = $bindable(),
    spokenLanguage = $bindable(),
    mtModel = $bindable(),
    targetLanguage = $bindable(),
    translateEnabled = $bindable(),
    showOriginal = $bindable(),
    onRefreshSources,
    onModelsChanged,
  }: {
    sources: SourceInfo[];
    languages: Language[];
    models: ModelStatus[];
    locked: boolean;
    preferences: Preferences;
    t: Translate;
    source: string;
    asrModel: string;
    spokenLanguage: string;
    mtModel: string;
    targetLanguage: string;
    translateEnabled: boolean;
    showOriginal: boolean;
    onRefreshSources: () => void;
    onModelsChanged: () => void;
  } = $props();

  type Section = "general" | "models" | "interface" | "about";
  type Key = Parameters<Translate>[0];

  const SECTIONS: { id: Section; label: Key; icon: IconName }[] = [
    { id: "general", label: "settings.general", icon: "general" },
    { id: "models", label: "settings.models", icon: "models" },
    { id: "interface", label: "settings.interface", icon: "interface" },
    { id: "about", label: "settings.about", icon: "about" },
  ];

  let section = $state<Section>("general");

  const installedAsr = $derived(models.filter((m) => m.kind === "asr" && m.installed));
  const installedMt = $derived(models.filter((m) => m.kind === "mt" && m.installed));
  const vadInstalled = $derived(models.some((m) => m.kind === "vad" && m.installed));
  const selectedSource = $derived(sources.find((s) => s.id === source));
</script>

<ViewHeader icon="settings" title={t("action.settings")}>
  {#snippet tabs()}
    {#each SECTIONS as item (item.id)}
      <button class:on={section === item.id} onclick={() => (section = item.id)}>
        <Icon name={item.icon} size="0.9375rem" />
        {t(item.label)}
      </button>
    {/each}
  {/snippet}
</ViewHeader>

<div class="settings">
  {#if section === "general"}
    {#if locked}
      <p class="banner">{t("settings.locked")}</p>
    {/if}

    <section>
      <div class="head"><h3>{t("settings.audioTitle")}</h3></div>
      <p class="lead">{t("settings.audioNote")}</p>
      <div class="controls source">
        <label class="field">
          <span>{t("settings.audio")}</span>
          <select bind:value={source} disabled={locked}>
            {#each sources as option (option.id)}
              <option value={option.id}>{option.name}</option>
            {/each}
          </select>
        </label>
        <button onclick={onRefreshSources} disabled={locked}>{t("action.refresh")}</button>
      </div>
      {#if selectedSource?.detail}
        <p class="hint">{selectedSource.detail}</p>
      {/if}
    </section>

    <section>
      <div class="head"><h3>{t("settings.recognition")}</h3></div>
      <p class="lead">{t("settings.recognitionNote")}</p>
      <div class="controls pair">
        <label class="field">
          <span>{t("settings.model")}</span>
          <select bind:value={asrModel} disabled={locked}>
            {#if installedAsr.length === 0}
              <option value="">{t("settings.noModel")}</option>
            {/if}
            {#each installedAsr as model (model.id)}
              <option value={model.id}>{model.name}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span>{t("settings.language")}</span>
          <select bind:value={spokenLanguage} disabled={locked}>
            {#each SPOKEN_LANGUAGES as option (option.code)}
              <option value={option.code}>{option.name || t("settings.detect")}</option>
            {/each}
          </select>
        </label>
      </div>
      {#if !vadInstalled}
        <p class="hint warn">{t("settings.needVad")}</p>
      {:else if installedAsr.length === 0}
        <p class="hint warn">{t("settings.needAsr")}</p>
      {/if}
    </section>

    <section>
      <div class="head">
        <h3>{t("settings.translation")}</h3>
        <label class="check">
          <input type="checkbox" bind:checked={translateEnabled} disabled={locked} />
          <span>{t("settings.on")}</span>
        </label>
      </div>
      <p class="lead">{t("settings.translationNote")}</p>
      <div class="controls pair">
        <label class="field">
          <span>{t("settings.model")}</span>
          <select bind:value={mtModel} disabled={locked || !translateEnabled}>
            {#if installedMt.length === 0}
              <option value="">{t("settings.noModel")}</option>
            {/if}
            {#each installedMt as model (model.id)}
              <option value={model.id}>{model.name}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span>{t("settings.language")}</span>
          <select bind:value={targetLanguage} disabled={locked || !translateEnabled}>
            {#each languages as option (option.code)}
              <option value={option.code}>{option.endonym}</option>
            {/each}
          </select>
        </label>
      </div>
      {#if translateEnabled && installedMt.length === 0}
        <p class="hint warn">{t("settings.needMt")}</p>
      {:else}
        <label class="check spaced">
          <input type="checkbox" bind:checked={showOriginal} disabled={!translateEnabled} />
          <span>{t("settings.showOriginal")}</span>
        </label>
      {/if}
    </section>

    <section>
      <div class="head"><h3>{t("settings.samples")}</h3></div>
      <p class="lead">{t("settings.samplesNote")}</p>
      <SamplesCard {t} />
    </section>
  {:else if section === "models"}
    <section>
      <ModelsCard {t} onchange={onModelsChanged} />
    </section>
  {:else if section === "interface"}
    <section>
      <div class="head"><h3>{t("settings.interface")}</h3></div>
      <p class="lead">{t("settings.interfaceNote")}</p>
      <div class="controls pair">
        <label class="field">
          <span>{t("settings.language")}</span>
          <select
            value={preferences.language}
            onchange={(e) => (preferences.language = e.currentTarget.value as Locale)}
          >
            {#each LOCALES as locale (locale.code)}
              <option value={locale.code}>{locale.name}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span>{t("settings.theme")}</span>
          <select
            value={preferences.theme}
            onchange={(e) => (preferences.theme = e.currentTarget.value as ThemeId)}
          >
            {#each THEMES as theme (theme)}
              <option value={theme}>{t(`theme.${theme}` as "theme.dark")}</option>
            {/each}
          </select>
        </label>
      </div>
      <div class="controls pair spaced">
        <label class="field">
          <span>{t("settings.textSize")}</span>
          <select
            value={preferences.size}
            onchange={(e) => (preferences.size = e.currentTarget.value as SizeId)}
          >
            {#each SIZES as size (size.id)}
              <option value={size.id}>{t(`size.${size.id}` as "size.small")}</option>
            {/each}
          </select>
        </label>
      </div>
    </section>
  {:else}
    <AboutCard {t} />
  {/if}
</div>

<style>
  .settings {
    flex: 1 1 0;
    min-height: 0;
    overflow-y: auto;
    /* Reserved whether or not it is there, so the controls do not jump sideways
       when the list grows past the window. */
    scrollbar-gutter: stable both-edges;
    padding: 0 var(--gutter) var(--space-4);
  }

  section {
    padding: var(--space-4) 0;
    border-bottom: 1px solid var(--line);
  }

  section:last-child {
    border-bottom: none;
  }

  /* Every section opens with this row, whether or not it has a control in it.
     One height for all of them: sized by its own contents, the Translation
     section - the only one with a switch on the title line - started a few
     pixels lower than the rest, and that is exactly the kind of difference you
     see without being able to name. */
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    min-height: var(--control-small);
    margin-bottom: var(--space-2);
  }

  h3 {
    margin: 0;
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--dim);
  }

  /* What this section is for, in a sentence. */
  .lead {
    margin: 0 0 var(--space-3);
    font-size: var(--text-md);
    line-height: 1.55;
    color: var(--muted);
  }

  /* Every control on one grid: paired dropdowns split the width evenly, and a
     trailing button keeps its own column. Nothing is sized by its content. */
  .controls {
    display: grid;
    gap: var(--space-3);
  }

  .controls.pair {
    grid-template-columns: 1fr 1fr;
  }

  /* The button keeps its own column and is sized by its word, with a floor so
     "Refresh" is not a button the width of the word. */
  .controls.source {
    grid-template-columns: 1fr auto;
    align-items: end;
  }

  .controls.source button {
    min-width: 7rem;
  }

  /* A second row of the same grid rather than a third column: three dropdowns
     across are narrower than the words in them, and the text size keeping the
     width of a language name is what makes the two rows line up. */
  .controls.spaced {
    margin-top: var(--space-3);
  }

  /* Every control says what it is. Two dropdowns side by side with only their
     current value to go on is a guessing game - which of them was the model and
     which the language was the question the settings never answered. */
  .field {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  .field span {
    font-size: var(--text-sm);
    color: var(--dim);
  }

  .field select,
  .controls button {
    width: 100%;
    min-width: 0;
  }

  .hint {
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
    color: var(--dim);
  }

  .hint.warn {
    color: var(--warn);
  }

  .banner {
    margin: var(--space-4) 0 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius);
    background: var(--raised);
    border: 1px solid var(--line);
    font-size: var(--text-md);
    color: var(--muted);
  }

  .check {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-md);
    color: var(--muted);
    cursor: pointer;
  }

  .check.spaced {
    margin-top: var(--space-3);
  }

  @media (max-width: 40rem) {
    .controls.pair,
    .controls.source {
      grid-template-columns: 1fr;
    }
  }
</style>
