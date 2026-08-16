<script lang="ts">
  /**
   * The line beside the level orb.
   *
   * The toolbar had a lot of empty middle and nothing to say in it. This says
   * something - one sentence at a time, in the language the interface is in,
   * changing every few seconds - and it changes character when the pipeline
   * starts, which is a second, quieter confirmation that the app is listening.
   *
   * It is set in its own layer and crossfaded rather than replaced, so nothing
   * around it moves when the sentence changes.
   */
  import { fade } from "svelte/transition";
  import { phrases, type Locale } from "./i18n";

  let { locale, running }: { locale: Locale; running: boolean } = $props();

  /// Long enough to read twice without noticing it is on a timer.
  const EVERY = 7000;

  const pool = $derived(phrases(locale, running));
  /// Where in the pool this window happens to start, so two launches do not
  /// open on the same greeting.
  let step = $state(Math.floor(Math.random() * 10));
  const phrase = $derived(pool[step % pool.length]);

  $effect(() => {
    // Not while the window is hidden. Nobody is reading it, and the crossfades
    // that would have run in the meantime are all waiting on the same first
    // frame after it comes back.
    const timer = setInterval(() => {
      if (!document.hidden) step += 1;
    }, EVERY);
    return () => clearInterval(timer);
  });
</script>

<p class="phrase" class:running aria-live="polite">
  {#key phrase}
    <!-- The outgoing line leaves before the incoming one arrives; overlapping
         them reads as a smear rather than a change. -->
    <span in:fade={{ duration: 420, delay: 220 }} out:fade={{ duration: 200 }}>{phrase}</span>
  {/key}
</p>

<style>
  .phrase {
    position: relative;
    margin: 0;
    min-width: 0;
    flex: 1 1 auto;
    /* Its own height, so the toolbar does not breathe with the length of the
       sentence in it. */
    height: 1.5em;
    font-size: var(--text-md);
    line-height: 1.5;
    color: var(--dim);
    transition: color 400ms ease;
  }

  /* While it is listening the line is part of the running state, not an idle
     decoration. */
  .phrase.running {
    color: var(--muted);
  }

  span {
    position: absolute;
    inset: 0;
    display: block;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  @media (prefers-reduced-motion: reduce) {
    span {
      transition: none;
    }
  }
</style>
