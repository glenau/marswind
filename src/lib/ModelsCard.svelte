<script lang="ts">
  /**
   * The model catalogue.
   *
   * Grouped by what a model is *for*, and every row says so on its badge. A
   * flat list of names - "Small", "Qwen3 4B", "Silero" - cannot tell anyone
   * which two of them they need to install before the app does anything, which
   * is the only question this list has to answer.
   *
   * Each row also names the license its weights come under, next to the button
   * that fetches them. Everything in the catalog is MIT or Apache-2.0, so today
   * that line is reassurance rather than a warning - but it is the app that
   * does the downloading, and the moment to see the terms is before a
   * two-gigabyte transfer rather than after.
   */
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import {
    cancelDownload,
    downloadModel,
    formatSize,
    listModels,
    modelsDiskUsage,
    removeModel,
    type ModelStatus,
    type ProgressEvent,
  } from "./api";
  import type { Translate } from "./i18n";

  let { t, onchange }: { t: Translate; onchange?: () => void } = $props();

  let models = $state<ModelStatus[]>([]);
  let diskUsage = $state(0);
  let progress = $state<Record<string, { done: number; total: number }>>({});
  let error = $state("");
  let unlisten: UnlistenFn | undefined;

  const GROUPS = [
    { kind: "asr", label: "models.forRecognition" },
    { kind: "mt", label: "models.forTranslation" },
    { kind: "vad", label: "models.forVad" },
  ] as const;

  const grouped = $derived(
    GROUPS.map((group) => ({
      ...group,
      models: models.filter((model) => model.kind === group.kind),
    })).filter((group) => group.models.length > 0),
  );

  export async function refresh() {
    models = await listModels();
    diskUsage = await modelsDiskUsage();
  }

  onMount(async () => {
    await refresh();
    unlisten = await listen<ProgressEvent>("models://progress", async (event) => {
      const { id, downloadedBytes, totalBytes, done, error: failure } = event.payload;

      if (failure) error = `${id}: ${failure}`;
      if (done) {
        const { [id]: _removed, ...rest } = progress;
        progress = rest;
        await refresh();
        onchange?.();
      } else {
        progress = { ...progress, [id]: { done: downloadedBytes, total: totalBytes } };
      }
    });
  });

  onDestroy(() => unlisten?.());

  async function download(model: ModelStatus) {
    error = "";
    progress = { ...progress, [model.id]: { done: 0, total: model.sizeBytes } };
    try {
      await downloadModel(model.id);
    } catch (e) {
      error = String(e);
      const { [model.id]: _removed, ...rest } = progress;
      progress = rest;
    }
    await refresh();
    onchange?.();
  }

  async function remove(model: ModelStatus) {
    error = "";
    try {
      await removeModel(model.id);
    } catch (e) {
      error = String(e);
    }
    await refresh();
    onchange?.();
  }
</script>

<div class="head">
  <h3>{t("models.title")}</h3>
  <span class="usage">{formatSize(diskUsage)} {t("models.onDisk")}</span>
</div>
<p class="lead">{t("models.note")}</p>

{#if error}
  <p class="error">{error}</p>
{/if}

{#each grouped as group (group.kind)}
  <div class="group">
    <p class="group-label">{t(group.label)}</p>
    <ul>
      {#each group.models as model (model.id)}
        {@const active = progress[model.id]}
        <li>
          <div class="info">
            <div class="title">
              <span class="name">{model.name}</span>
              <span class="badge purpose">{t(group.label)}</span>
              {#if model.kind === "vad"}
                <span class="badge required">{t("models.required")}</span>
              {:else if model.recommended}
                <span class="badge recommended">{t("models.recommended")}</span>
              {/if}
              {#if model.installed}
                <span class="badge installed">{t("models.installed")}</span>
              {/if}
            </div>
            <p class="note">
              {model.note}
              <button class="license" onclick={() => openUrl(model.licenseUrl)}>
                {model.license}
              </button>
            </p>
            {#if active}
              <div class="bar">
                <div
                  class="fill"
                  style="width: {active.total ? (100 * active.done) / active.total : 0}%"
                ></div>
              </div>
              <p class="note">
                {formatSize(active.done)}
                {t("models.of")}
                {formatSize(active.total)}
              </p>
            {/if}
          </div>

          <div class="actions">
            <span class="size">{formatSize(model.sizeBytes)}</span>
            {#if active}
              <button onclick={() => cancelDownload(model.id)}>{t("action.cancel")}</button>
            {:else if model.installed}
              <button onclick={() => remove(model)}>{t("action.remove")}</button>
            {:else}
              <button class="primary" onclick={() => download(model)}>{t("action.install")}</button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  </div>
{/each}

<style>
  /* Title and the one number that belongs beside it, on the same line - the
     same row, at the same height, as every other section title in the
     settings. */
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    min-height: var(--control-small);
    margin-bottom: var(--space-2);
  }

  /* What this section is for, in a sentence, exactly as every other section in
     the settings opens. */
  .lead {
    margin: 0;
    font-size: var(--text-md);
    line-height: 1.55;
    color: var(--muted);
  }

  h3 {
    margin: 0;
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--dim);
  }

  .usage {
    font-size: var(--text-sm);
    color: var(--dim);
  }

  .group {
    margin-top: var(--space-4);
  }

  .group-label {
    margin: 0 0 var(--space-1);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--muted);
    text-transform: capitalize;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--line);
  }

  li:last-child {
    border-bottom: none;
  }

  .info {
    min-width: 0;
  }

  .title {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .name {
    font-size: var(--text-md);
    font-weight: 500;
  }

  /* Badges say what a model is for and whether it is here. Same shape, colour
     carries the meaning. */
  .badge {
    padding: 0.0625rem var(--space-2);
    border-radius: 999px;
    border: 1px solid var(--line-strong);
    font-size: var(--text-xs);
    color: var(--dim);
    white-space: nowrap;
  }

  .badge.purpose {
    color: var(--muted);
    text-transform: capitalize;
  }

  .badge.recommended {
    color: var(--info-ink);
    border-color: var(--info-line);
  }

  .badge.required {
    color: var(--warn);
    border-color: var(--warn-line);
  }

  .badge.installed {
    color: var(--ok-ink);
    border-color: var(--ok-line);
  }

  .note {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    line-height: 1.5;
    color: var(--dim);
  }

  /* A link at the end of the note rather than a row of its own: the terms
     matter, and on twelve of the thirteen models they are the answer nobody
     needed. Stripped back to text so it reads as the end of the sentence it
     sits in - it inherits the note's size and colour and is told apart by the
     underline alone. */
  .license {
    all: unset;
    cursor: pointer;
    white-space: nowrap;
    text-decoration: underline;
    text-decoration-color: var(--line-strong);
    text-underline-offset: 0.15em;
  }

  .license:hover {
    color: var(--muted);
    text-decoration-color: currentColor;
  }

  .license:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: var(--radius);
  }

  .bar {
    margin-top: var(--space-2);
    height: 0.25rem;
    border-radius: 999px;
    background: var(--raised);
    overflow: hidden;
  }

  .bar .fill {
    height: 100%;
    background: var(--accent);
    transition: width 200ms ease;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: none;
  }

  .size {
    font-size: var(--text-sm);
    color: var(--dim);
    font-variant-numeric: tabular-nums;
  }

  /* A minimum rather than a width: every button in this column lines up at the
     same size in most languages, and the ones with a longer word for "Install"
     grow instead of clipping it. */
  .actions button {
    min-width: 6rem;
  }

  .error {
    margin: var(--space-2) 0 0;
    font-size: var(--text-sm);
    color: var(--error-ink);
  }
</style>
