<script lang="ts">
  /**
   * The strip along the bottom of the window: which parts of the pipeline are
   * up, and the one action that belongs to the transcript rather than to the
   * app.
   *
   * These indicators used to sit in the toolbar, beside the button that starts
   * everything. They are not something anyone acts on - they are something you
   * check when a line does not appear - and the bottom edge is where that kind
   * of thing belongs. It also gave the toolbar its middle back.
   *
   * The mark in the corner opens the project. It belongs here rather than in
   * the toolbar for the same reason the indicators do: it is where you look
   * when you want to know what this thing is, not when you want it to do
   * something. It is also in About, at length; this is the version for someone
   * who already knows what they are looking for.
   */
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "./Icon.svelte";
  import { HOMEPAGE } from "./api";
  import type { Translate } from "./i18n";

  export type Stage = { label: string; running: boolean; detail: string };

  let {
    stages,
    canClear = false,
    t,
    onClear,
  }: {
    stages: Stage[];
    canClear?: boolean;
    t: Translate;
    onClear: () => void;
  } = $props();
</script>

<footer>
  <button
    class="quiet github"
    onclick={() => openUrl(HOMEPAGE)}
    title={t("action.github")}
    aria-label={t("action.github")}
  >
    <Icon name="github" size="1.0625rem" />
  </button>

  <div class="stages">
    {#each stages as stage (stage.label)}
      <span class="pill" class:on={stage.running} title={stage.detail}>
        <span class="dot"></span>
        {stage.label}
      </span>
    {/each}
  </div>

  <!-- Only there when there is something to clear. A permanently disabled
       button is a permanent question about why it is disabled. -->
  {#if canClear}
    <button class="quiet clear" onclick={onClear}>{t("action.clear")}</button>
  {/if}
</footer>

<style>
  /* One height, whatever is in it. The pills are shorter than a button, so a
     bar sized by its contents grew by a few pixels the moment Clear appeared -
     and the whole transcript above it moved up to make room. The height is the
     tallest thing that can be in here, set once. */
  footer {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    /* The same margin above and below as the one down the sides, so the strip
       is inset evenly rather than being a line of pills pressed against the
       bottom edge. The height is set rather than left to the contents: the
       pills are shorter than a button, and a bar sized by what is in it grew
       the moment Clear appeared - taking the transcript up with it. Boxes are
       border-box here, so the hairline has to be in the number too. */
    min-height: calc(var(--control-small) + var(--gutter) * 2 + 1px);
    padding: var(--gutter);
    border-top: 1px solid var(--line);
  }

  /* The mark, and nothing else in the corner. Square and quiet: at the size of
     the strip it is a piece of punctuation, not a control competing with the
     stages beside it. */
  .github {
    flex: none;
    width: var(--control-small);
    height: var(--control-small);
    padding: 0;
    color: var(--dim);
  }

  .github:hover:not(:disabled) {
    background: transparent;
    border-color: transparent;
    color: var(--text);
  }

  .stages {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    min-width: 0;
  }

  /* Only the dot changes when a stage comes up, so nothing on the row moves. */
  .pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    /* No `height`: the padding sets it, which is what keeps the badge in
       proportion to its own text at every text-size setting. */
    padding: 0.1875rem var(--space-3);
    border-radius: 999px;
    background: var(--raised);
    border: 1px solid var(--line);
    font-size: var(--text-xs);
    line-height: 1.4;
    color: var(--dim);
    white-space: nowrap;
    transition: color 160ms ease;
  }

  .pill.on {
    color: var(--text);
  }

  /* Dot and halo are painted as one gradient from one centre rather than as a
     box with a shadow around it. A 6px box and a 3px spread both land on
     fractions of a pixel once the text-size setting scales them, and the two
     were rounded independently - which is what put the green core off-centre in
     its own glow. One paint cannot be off-centre from itself.
     The box is the size of the halo, so nothing on the row moves when a stage
     comes up. */
  .dot {
    position: relative;
    width: 0.75rem;
    height: 0.75rem;
    flex: none;
    background: radial-gradient(
      circle at 50% 50%,
      var(--dot-off) 0 0.1875rem,
      transparent calc(0.1875rem + 0.5px)
    );
  }

  /* The lit state is a second layer over the first, so it can be faded in
     rather than swapped - a gradient is not a value CSS can transition. */
  .dot::after {
    content: "";
    position: absolute;
    inset: 0;
    opacity: 0;
    background: radial-gradient(
      circle at 50% 50%,
      var(--ok) 0 0.1875rem,
      rgba(72, 192, 108, 0.16) calc(0.1875rem + 0.5px) 0.375rem,
      transparent calc(0.375rem + 0.5px)
    );
    transition: opacity 200ms ease;
  }

  .pill.on .dot::after {
    opacity: 1;
  }

  /* Text, not a control. Everything else on this strip is something to read,
     and a button with a panel behind it in the corner of the window pulls more
     than a word that clears the transcript is worth. It answers by brightening
     instead. */
  .clear {
    margin-left: auto;
    flex: none;
    height: var(--control-small);
    padding: 0 var(--space-2);
    font-size: var(--text-sm);
    color: var(--dim);
  }

  .clear:hover:not(:disabled) {
    background: transparent;
    border-color: transparent;
    color: var(--text);
  }
</style>
