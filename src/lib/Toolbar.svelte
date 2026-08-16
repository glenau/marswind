<script lang="ts">
  /**
   * The one row of controls: the level, a word about what the app is doing, and
   * the button that matters. History and Settings are views rather than
   * dialogs, so their buttons behave as tabs - pressing the one that is already
   * open goes back.
   *
   * The stage indicators used to sit here and now live along the bottom of the
   * window, with the rest of the machinery. What is at the top is what the
   * reader acts on; what is at the bottom is what they glance at.
   */
  import LevelOrb from "./LevelOrb.svelte";
  import Phrase from "./Phrase.svelte";
  import type { Locale, Translate } from "./i18n";

  export type View = "transcript" | "history" | "settings";

  let {
    level,
    busy,
    running,
    view,
    locale,
    t,
    onToggleRun,
    onShow,
  }: {
    level: number;
    busy: boolean;
    running: boolean;
    view: View;
    locale: Locale;
    t: Translate;
    onToggleRun: () => void;
    onShow: (view: View) => void;
  } = $props();

  function toggle(target: View) {
    onShow(view === target ? "transcript" : target);
  }

  /// What the run button currently says. The button holds all three labels so
  /// it cannot change width under the cursor, and the two it is not saying are
  /// `visibility: hidden` - which takes them out of the accessible tree, and
  /// took the button's name with them. This is the name.
  const runLabel = $derived(
    busy ? t("action.working") : running ? t("action.stop") : t("action.start"),
  );
</script>

<header>
  <!-- The level and the line about it read as one thing, which is why they sit
       together against the left edge rather than floating in the middle. -->
  <div class="status">
    <LevelOrb {level} {running} title={t("meter.level")} />
    <Phrase {locale} {running} />
  </div>

  <div class="actions">
    <!-- All three labels are in the button and only one of them is visible, so
         it is as wide as the longest thing it can say and no wider. A fixed
         width was the same idea in one number, and one number cannot be right
         for thirteen languages: "Stop" and "Commencer à écouter" are both this
         button. -->
    <button
      class="primary run"
      class:destructive={running}
      onclick={onToggleRun}
      disabled={busy}
      aria-label={runLabel}
    >
      <span class="labels" aria-hidden="true">
        <span class:shown={busy}>{t("action.working")}</span>
        <span class:shown={!busy && running}>{t("action.stop")}</span>
        <span class:shown={!busy && !running}>{t("action.start")}</span>
      </span>
    </button>

    <button class="tab" class:on={view === "history"} onclick={() => toggle("history")}>
      {t("action.history")}
    </button>
    <button class="tab" class:on={view === "settings"} onclick={() => toggle("settings")}>
      {t("action.settings")}
    </button>
  </div>
</header>

<style>
  /* Two tracks: everything that reports sits in the first and takes the spare
     width, everything that acts sits in the second and keeps its size.
     `minmax(0, 1fr)` rather than `1fr` - a track that cannot shrink below its
     content is what let the buttons slide off the edge of a narrow window. */
  header {
    flex: none;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--gutter);
    border-bottom: 1px solid var(--line);
  }

  .status {
    grid-column: 1;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .actions {
    grid-column: 2;
    justify-self: end;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  /* The three labels on one grid cell: the button is sized by the widest of
     them, so pressing it never changes its own width - which is the jitter a
     fixed width was there to stop - and no translation is clipped.
     `visibility` rather than `display`, so the hidden two hold the cell open
     and stay out of the accessible name. */
  .run {
    min-width: 8.5rem;
  }

  .labels {
    display: grid;
  }

  .labels > span {
    grid-area: 1 / 1;
    visibility: hidden;
  }

  .labels > span.shown {
    visibility: visible;
  }

  /* These labels never change while the window is open, so a minimum is enough:
     the row of buttons keeps its rhythm in English and grows for the languages
     that need the room. */
  .tab {
    min-width: 6rem;
  }

  /* The sentence goes first when there is no room for it: it is company, not
     information. The orb stays - it is the only thing here that is live. */
  @media (max-width: 52rem) {
    :global(.status .phrase) {
      display: none;
    }
  }
</style>
