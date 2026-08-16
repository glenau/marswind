<script lang="ts">
  /**
   * Past sessions: the list on the left, the session you picked on the right.
   *
   * Everything a session recorded is here, including the numbers behind it -
   * how long recognition took on a row, how long its translation took, and
   * whether any of it was dropped. That is what makes a session useful after
   * the fact rather than just a transcript.
   */
  import { onMount } from "svelte";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import ViewHeader from "./ViewHeader.svelte";
  import {
    exportSession,
    formatDuration,
    listSessions,
    readSession,
    removeSession,
    type ExportFormat,
    type Session,
    type SessionSummary,
  } from "./api";
  import type { Translate } from "./i18n";

  let { t }: { t: Translate } = $props();

  let sessions = $state<SessionSummary[]>([]);
  let selected = $state<Session | null>(null);
  let loading = $state(true);
  let error = $state("");
  let note = $state("");

  const FORMATS: { id: ExportFormat; label: "history.exportText" }[] = [
    { id: "text", label: "history.exportText" },
    { id: "srt", label: "history.exportSrt" as "history.exportText" },
    { id: "json", label: "history.exportJson" as "history.exportText" },
  ];

  onMount(refresh);

  async function refresh() {
    loading = true;
    try {
      sessions = await listSessions();
      if (sessions.length > 0) await open(sessions[0].id);
      else selected = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function open(id: string) {
    note = "";
    try {
      selected = await readSession(id);
    } catch (e) {
      error = String(e);
    }
  }

  async function save(format: ExportFormat) {
    if (!selected) return;
    try {
      const path = await exportSession(selected.id, format);
      note = `${t("history.saved")} ${path}`;
      await revealItemInDir(path);
    } catch (e) {
      error = String(e);
    }
  }

  async function remove(id: string) {
    try {
      await removeSession(id);
      if (selected?.id === id) selected = null;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<ViewHeader icon="history" title={t("action.history")} />

<div class="history">
  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="body">
    <nav>
      {#if loading}
        <p class="quiet">{t("history.loading")}</p>
      {:else if sessions.length === 0}
        <p class="quiet">{t("history.empty")}</p>
      {:else}
        <ul>
          {#each sessions as session (session.id)}
            <li>
              <button
                class="quiet entry"
                class:on={selected?.id === session.id}
                onclick={() => open(session.id)}
              >
                <span class="when">{session.startedAt}</span>
                <span class="meta">
                  {formatDuration(session.durationSeconds)} · {session.rows}
                  {t("history.rows")} · {session.words}
                  {t("history.words")}
                </span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </nav>

    <section class="detail">
      {#if selected}
        <div class="detail-head">
          <div>
            <h3>{selected.startedAt}</h3>
            <p class="quiet">
              {selected.asrModel || t("history.noModel")}
              {#if selected.mtModel}· {selected.mtModel} → {selected.targetLanguage}{/if}
              · {formatDuration(selected.durationSeconds)}
            </p>
          </div>
          <button onclick={() => remove(selected!.id)}>{t("action.delete")}</button>
        </div>

        <div class="exports">
          {#each FORMATS as format (format.id)}
            <button onclick={() => save(format.id)}>{t(format.label)}</button>
          {/each}
        </div>
        {#if note}<p class="note">{note}</p>{/if}

        <div class="rows">
          {#each selected.rows as row (row.line)}
            <article class="row">
              <span class="at">{formatDuration(row.at)}</span>
              <div class="texts">
                <p class="source">{row.source}</p>
                {#if row.translation}<p class="target">{row.translation}</p>{/if}
                {#if row.skippedSegments > 0}
                  <p class="gap">
                    {row.skippedSegments}
                    {row.skippedSegments === 1 ? t("history.segment") : t("history.segments")}
                    · {t("history.notTranslated")}
                  </p>
                {/if}
              </div>
              <span class="timing">
                {row.recognitionMs} ms
                {#if row.translationMs > 0}· {row.translationMs} ms{/if}
              </span>
            </article>
          {/each}
        </div>
      {:else if !loading}
        <p class="quiet centered">{t("history.pick")}</p>
      {/if}
    </section>
  </div>
</div>

<style>
  /* The header above is a sibling in the same column, so this takes what is
     left rather than the full height of the window. */
  .history {
    flex: 1 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 19rem 1fr;
  }

  /* The list is inset by half the gutter and its entries carry the other half,
     so the text inside them starts on the same margin as the view title above
     and the session on the right. */
  nav {
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    border-right: 1px solid var(--line);
    padding: var(--space-2);
  }

  /* The empty message has no entry to sit in, so it carries the other half
     itself. */
  nav > .quiet {
    padding: var(--space-2);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .entry {
    width: 100%;
    height: auto;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-1);
    padding: var(--space-3) var(--space-2);
    text-align: left;
  }

  .when {
    font-size: var(--text-md);
    font-weight: 500;
    color: var(--text);
  }

  .meta {
    font-size: var(--text-sm);
    color: var(--dim);
    font-variant-numeric: tabular-nums;
  }

  .detail {
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    padding: var(--space-4) var(--gutter);
  }

  .detail-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  h3 {
    margin: 0 0 var(--space-1);
    font-size: var(--text-lg);
    font-weight: 600;
  }

  .exports {
    display: flex;
    gap: var(--space-2);
    margin: var(--space-4) 0 0;
  }

  .note {
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  .rows {
    margin-top: var(--space-4);
    border-top: 1px solid var(--line);
  }

  .row {
    display: grid;
    grid-template-columns: 3.5rem 1fr 8rem;
    gap: var(--space-4);
    align-items: start;
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .at,
  .timing {
    font-size: var(--text-sm);
    color: var(--dim);
    font-variant-numeric: tabular-nums;
    padding-top: 0.125rem;
  }

  .timing {
    text-align: right;
  }

  .texts p {
    margin: 0;
    font-size: var(--text-md);
    line-height: 1.55;
    overflow-wrap: break-word;
  }

  .source {
    color: var(--muted);
  }

  .target {
    color: var(--text);
  }

  .gap {
    margin-top: var(--space-1);
    font-size: var(--text-sm);
    color: var(--warn);
  }

  .quiet {
    margin: 0;
    font-size: var(--text-md);
    line-height: 1.55;
    color: var(--dim);
  }

  .centered {
    text-align: center;
    padding-top: 12vh;
  }

  .error {
    flex: none;
    margin: 0;
    padding: var(--space-3) var(--gutter);
    background: var(--error-bg);
    border-bottom: 1px solid var(--error-line);
    color: var(--error-ink);
    font-size: var(--text-md);
  }

  @media (max-width: 52rem) {
    .body {
      grid-template-columns: 1fr;
      grid-template-rows: 11rem 1fr;
    }

    nav {
      border-right: none;
      border-bottom: 1px solid var(--line);
    }

    .row {
      grid-template-columns: 3rem 1fr;
    }

    .timing {
      display: none;
    }
  }
</style>
