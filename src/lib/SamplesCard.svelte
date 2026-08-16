<script lang="ts">
  /**
   * The speech clips shipped with the app.
   *
   * Testing the app otherwise means finding a video, which makes "it does not
   * work" and "nothing is playing" indistinguishable. These play through the
   * normal system output, so they reach the tap exactly as any other sound
   * does - and each one comes with the words it contains, so what came out can
   * be compared with what went in without leaving the window.
   */
  import { onMount, onDestroy } from "svelte";
  import { listSamples, playSample, playingSample, stopSample, type SampleInfo } from "./api";
  import type { Translate } from "./i18n";

  let { t }: { t: Translate } = $props();

  let samples = $state<SampleInfo[]>([]);
  let playing = $state<string | null>(null);
  let shown = $state<string | null>(null);
  let error = $state("");
  let timer: ReturnType<typeof setInterval> | undefined;

  onMount(async () => {
    try {
      samples = await listSamples();
      playing = await playingSample();
    } catch (e) {
      error = String(e);
    }
    // A clip ends on its own, and the button has to stop saying Stop when it
    // does.
    timer = setInterval(async () => {
      playing = await playingSample().catch(() => null);
    }, 700);
  });

  // Only the polling stops here. Stopping the clip as well made the samples
  // useless: you play one to watch the transcript fill, and watching it means
  // leaving this view - which killed the audio the moment you did.
  onDestroy(() => clearInterval(timer));

  async function toggle(sample: SampleInfo) {
    error = "";
    try {
      if (playing === sample.id) {
        await stopSample();
        playing = null;
      } else {
        await playSample(sample.id);
        playing = sample.id;
      }
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if error}
  <p class="error">{error}</p>
{/if}

<ul>
  {#each samples as sample (sample.id)}
    <li>
      <div class="info">
        <div class="title">
          <span class="name">{sample.name}</span>
          {#if playing === sample.id}<span class="badge">{t("action.play")}</span>{/if}
        </div>
        <p class="note">{sample.note}</p>
        {#if shown === sample.id && sample.transcript}
          <p class="transcript">{sample.transcript}</p>
        {/if}
      </div>

      <div class="actions">
        {#if sample.transcript}
          <button class="quiet" onclick={() => (shown = shown === sample.id ? null : sample.id)}>
            {shown === sample.id ? t("action.hideText") : t("action.text")}
          </button>
        {/if}
        <button onclick={() => toggle(sample)}>
          {playing === sample.id ? t("action.stop") : t("action.play")}
        </button>
      </div>
    </li>
  {/each}
</ul>

<style>
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
    gap: var(--space-2);
  }

  .name {
    font-size: var(--text-md);
    font-weight: 500;
  }

  .badge {
    padding: 0.0625rem var(--space-2);
    border-radius: 999px;
    border: 1px solid var(--ok-line);
    font-size: var(--text-xs);
    color: var(--ok-ink);
  }

  .note {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    line-height: 1.5;
    color: var(--dim);
  }

  .transcript {
    margin: var(--space-3) 0 0;
    padding: var(--space-3);
    border-radius: var(--radius);
    background: var(--raised);
    border: 1px solid var(--line);
    font-size: var(--text-sm);
    line-height: 1.6;
    color: var(--muted);
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    flex: none;
  }

  .actions button {
    min-width: 6rem;
  }

  .error {
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
    color: var(--error-ink);
  }
</style>
