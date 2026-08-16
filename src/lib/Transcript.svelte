<script lang="ts">
  /**
   * The transcript: the original on the left, its translation on the right, one
   * row per caption.
   *
   * Side by side rather than stacked because the two are read together - a
   * reader checking what a Russian phrase came from should find it level with
   * it, not have to count lines. Each row is its own grid, so the two cells
   * always start at the same baseline however long either one is.
   *
   * The rule between the columns is painted on the sheet, not on the rows, so
   * it runs the whole height of the window whether there is anything in it or
   * not. Drawn per row it appeared and vanished as the transcript filled, which
   * is what made the table look like it was assembling itself.
   */
  import { onMount, onDestroy } from "svelte";
  import { fade } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import { createCaptions, isIncomplete, isTranslated, translationOf } from "./captions.svelte";
  import type { Translate } from "./i18n";

  let {
    split = true,
    sourceName = "",
    targetName = "",
    t,
    onLines,
  }: {
    split?: boolean;
    sourceName?: string;
    targetName?: string;
    t: Translate;
    /** How many rows there are, for the Clear button that lives outside. */
    onLines?: (count: number) => void;
  } = $props();

  const captions = createCaptions();
  const lines = $derived(captions.lines);

  let scroller: HTMLElement | undefined = $state();
  /// Auto-scroll only while the user is at the bottom, so scrolling back to
  /// read something is not fought by every new caption.
  let following = $state(true);
  /// A smooth jump to the bottom is in flight. It reports every step of the way
  /// down as a scroll, and the last of those arrives before the final position
  /// does - read as user scrolling, that left the app convinced the reader had
  /// stopped following at the exact moment they asked to start again.
  let jumping = false;
  let jumpEnds: ReturnType<typeof setTimeout> | undefined;

  function onScroll() {
    if (!scroller || jumping) return;
    const distance = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
    following = distance < 48;
  }

  /// The reader taking the wheel ends the jump early, so a scroll away from the
  /// bottom while one is in flight is still obeyed. Attached from here rather
  /// than in the markup: these listen to the reader, they do not make the
  /// transcript something you interact with.
  $effect(() => {
    if (!scroller) return;
    const end = () => (jumping = false);
    const options = { passive: true } as const;
    scroller.addEventListener("wheel", end, options);
    scroller.addEventListener("touchmove", end, options);
    const element = scroller;
    return () => {
      element.removeEventListener("wheel", end);
      element.removeEventListener("touchmove", end);
    };
  });

  function follow() {
    if (!following) return;
    // On the frame, not on the microtask. A microtask runs before the engine
    // has laid the new caption out, so the scroll was measured against the
    // previous layout and the sticky heading above was moved against it too -
    // which is how a strip of the last frame's text ended up sitting above the
    // headings. A frame callback also collapses several arriving captions into
    // one scroll.
    requestAnimationFrame(() => scroller?.scrollTo({ top: scroller.scrollHeight }));
  }

  /** Back to the newest line, and following it again from then on.
   *
   * Smooth here and nowhere else: this is a jump the reader asked for and wants
   * to see happen, while the scrolling that keeps up with live captions has to
   * be instant or it never arrives before the next one. */
  function toBottom() {
    following = true;
    jumping = true;
    scroller?.scrollTo({ top: scroller.scrollHeight, behavior: "smooth" });
    // Longer than any smooth scroll the engine performs, and harmless if it is
    // late: the next real scroll simply re-reads where the reader is.
    clearTimeout(jumpEnds);
    jumpEnds = setTimeout(() => (jumping = false), 800);
  }

  onMount(() => captions.start());
  onDestroy(() => {
    captions.stop();
    clearTimeout(jumpEnds);
  });

  $effect(() => {
    captions.lines.length;
    captions.current?.text;
    captions.current?.segments.length;
    captions.current?.live?.text;
    follow();
  });

  $effect(() => onLines?.(lines.length));

  export function clear() {
    captions.clear();
    following = true;
  }
</script>

<section class="transcript" class:split>
  <main bind:this={scroller} onscroll={onScroll}>
    <div class="sheet">
      <!-- Each column says what is in it, in the language it is in: the left is
           whatever is being spoken, the right is what it is being turned into.
           "Original" told a reader nothing they could not already see.

           It lives inside the scroller and sticks to its top: outside it, the
           header kept the full width while the rows lost the scrollbar's, and
           the two stopped lining up as soon as the transcript filled. -->
      <header class="columns">
        <span class="from">{sourceName || t("transcript.original")}</span>
        {#if split}
          <span class="to">{t("transcript.into")} {targetName || t("transcript.translation")}</span>
          <!-- Narrow, the two columns become one and the pair of headings
               becomes one heading. "English" over a column of Russian said the
               wrong thing entirely. -->
          <span class="both">
            {sourceName || t("transcript.original")}
            {t("transcript.toLanguage")}
            {targetName || t("transcript.translation")}
          </span>
        {/if}
      </header>

      {#if lines.length === 0}
        <!-- One message per column rather than one across both. Centred over
             the whole width it sat on top of the rule and told the reader
             nothing about which side was which. -->
        <div class="rows">
          <div class="row empty">
            <p class="empty-text">{t("transcript.emptySource")}</p>
            {#if split}<p class="empty-text">{t("transcript.emptyTarget")}</p>{/if}
          </div>
        </div>
      {:else}
        <div class="rows">
          {#each lines as line (line.id)}
            {@const translation = translationOf(line)}
            {@const settled = isTranslated(line)}
            <article class="row">
              <p class="source">
                <!-- The space belongs to the expression: as markup between two
                     tags it is trimmed away and the words run together. -->
                {line.text}{#if line.tentative}<span class="tentative">{` ${line.tentative}`}</span
                  >{/if}
              </p>

              {#if split}
                <p class="target">
                  {#if translation}
                    {translation}{#if !settled}<span class="cursor" aria-hidden="true"></span>{/if}
                  {:else if settled}
                    <span class="dropped">{t("transcript.notTranslated")}</span>
                  {:else}
                    <span class="waiting">{t("transcript.translating")}</span>
                  {/if}
                  {#if settled && translation && isIncomplete(line)}
                    <span class="dropped">…</span>
                  {/if}
                </p>
              {/if}
            </article>
          {/each}
        </div>
      {/if}
    </div>
  </main>

  <!-- Only while the reader has scrolled away from the newest line. Standing at
       the bottom, it would be a button that does nothing, sitting on top of the
       text it does nothing to. -->
  {#if !following && lines.length > 0}
    <button
      class="to-bottom"
      onclick={toBottom}
      title={t("transcript.toBottom")}
      aria-label={t("transcript.toBottom")}
      transition:fade={{ duration: 140 }}
    >
      <Icon name="down" size="1.125rem" />
    </button>
  {/if}
</section>

<style>
  .transcript {
    position: relative;
    flex: 1 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  main {
    flex: 1 1 0;
    min-height: 0;
    overflow-y: auto;
    /* Reserved on both sides, whether or not there is a scrollbar: `stable`
       alone keeps the layout from jumping but leaves the content sitting
       further from the right edge than the left, and the rule between the
       columns off the true centre. Has no effect at all when the system is
       using overlay scrollbars, which is the usual case. */
    scrollbar-gutter: stable both-edges;
  }

  .sheet {
    /* Full height even when empty, which is what lets the rule run the whole
       way down. */
    min-height: 100%;
  }

  /* The rule sits at the middle of the sheet. Header and rows carry the same
     horizontal padding, so the middle of the sheet is also the middle of both
     of their grids. */
  .split .sheet {
    background: linear-gradient(
      to right,
      transparent calc(50% - 0.5px),
      var(--line) calc(50% - 0.5px),
      var(--line) calc(50% + 0.5px),
      transparent calc(50% + 0.5px)
    );
  }

  .columns {
    position: sticky;
    top: 0;
    z-index: 1;
    /* Promoted to its own compositing layer, deliberately.
       The transcript scrolls itself on every new caption, and WebKit moves a
       sticky element by repainting the strip it used to occupy - which, while
       the rows underneath are also changing, it does not always get to. The
       leftover was a band of the previous frame's text sitting above the
       column headings, which reads as a gap between them and the toolbar. A
       layer the compositor owns is moved rather than repainted, so there is no
       strip to leave behind. */
    transform: translateZ(0);
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--column-gap);
    padding: var(--space-3) var(--gutter);
    background: var(--bg);
    /* The heavier of the two weights, unlike the rules between rows. This one
       is the edge the transcript scrolls under rather than a division inside
       it, and it is the one line in the app that has to hold its own against a
       fade sitting directly beneath it. */
    border-bottom: 1px solid var(--line-strong);
    /* A long label wraps at middling widths, and without this the second line
       ended up touching the first row of the transcript. */
    line-height: 1.5;
    font-size: var(--text-xs);
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--dim);
  }

  /* A short fade below the headings, so a row passing under them thins out
     instead of being sliced through the middle of its letters. It hangs off the
     sticky element, so it travels with it.
     Below the hairline, not on it: a percentage offset on an absolutely
     positioned box resolves against the containing block's *padding* box, so
     `top: 100%` starts the fade at the top edge of the bottom border - and the
     fade opens at full `--bg`, which painted the rule out. The extra pixel is
     the border's own width. */
  .columns::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: calc(100% + 1px);
    height: var(--space-3);
    background: linear-gradient(var(--bg), transparent);
    pointer-events: none;
  }

  /* The rule between the columns is painted on the sheet, underneath this fade,
     and the fade opens at full `--bg` - so the first `--space-3` of it was
     painted out and the vertical rule appeared to start a gap below the
     horizontal one. It is redrawn here on top of the fade: same colour, same
     place, so the two rules meet and the join is a corner rather than a
     shadow. */
  .split .columns::after {
    background-image:
      linear-gradient(
        to right,
        transparent calc(50% - 0.5px),
        var(--line) calc(50% - 0.5px),
        var(--line) calc(50% + 0.5px),
        transparent calc(50% + 0.5px)
      ),
      linear-gradient(var(--bg), transparent);
  }

  .split .columns {
    grid-template-columns: 1fr 1fr;
  }

  /* The one-column heading only exists below the breakpoint at the bottom of
     this file. */
  .both {
    display: none;
  }

  .rows {
    padding: 0 var(--gutter);
  }

  .row {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--column-gap);
    align-items: start;
    padding: var(--space-4) 0;
    border-bottom: 1px solid var(--line);
  }

  .split .row {
    grid-template-columns: 1fr 1fr;
  }

  .row:last-child {
    border-bottom: none;
  }

  p {
    margin: 0;
    font-size: var(--text-lg);
    line-height: 1.55;
    /* Long recognized runs without spaces should break rather than push the
       column wider than the window. */
    overflow-wrap: break-word;
  }

  .source {
    color: var(--text);
  }

  /* Beside a translation the original is supporting material. */
  .split .source {
    color: var(--muted);
  }

  .tentative {
    color: var(--dim);
  }

  .waiting {
    color: var(--dim);
    font-size: var(--text-sm);
    font-style: italic;
    animation: breathe 1.4s ease-in-out infinite;
  }

  /* The translator was too far behind and this line never reached it. Saying so
     is the point: this used to be an indicator that spun forever. */
  .dropped {
    color: var(--warn);
    font-size: var(--text-sm);
  }

  /* The translation is still being written. A caret says so without moving the
     text that is already there. */
  .cursor {
    display: inline-block;
    width: 2px;
    height: 1em;
    margin-left: 0.1875rem;
    vertical-align: -0.15em;
    background: var(--accent);
    animation: blink 1s steps(2, start) infinite;
  }

  @keyframes blink {
    to {
      visibility: hidden;
    }
  }

  @keyframes breathe {
    50% {
      opacity: 0.45;
    }
  }

  .empty {
    border-bottom: none;
  }

  .empty-text {
    font-size: var(--text-md);
    line-height: 1.6;
    color: var(--dim);
  }

  /* Over the transcript at its bottom-right corner, clear of the last line of
     text rather than on top of it. Round, because it is the only control in the
     app that floats - nothing else it could be confused with. */
  .to-bottom {
    position: absolute;
    right: var(--gutter);
    bottom: var(--space-4);
    width: var(--control);
    height: var(--control);
    padding: 0;
    border-radius: 50%;
    background: var(--raised);
    border: 1px solid var(--line-strong);
    color: var(--muted);
    /* Lifted off the text underneath, which is the whole reason it reads as
       floating rather than as part of the last row. */
    box-shadow: var(--lift);
  }

  .to-bottom:hover:not(:disabled) {
    background: var(--raised-hover);
    color: var(--text);
  }

  /* Two columns need width. Below this the pair stacks, translation first,
     which is the same information in the order a reader wants it. */
  @media (max-width: 46rem) {
    .split .sheet {
      background: none;
    }

    /* One column, so there is no rule to carry through the fade. */
    .split .columns::after {
      background-image: linear-gradient(var(--bg), transparent);
    }

    .split .row {
      grid-template-columns: 1fr;
      gap: var(--space-1);
    }

    .split .columns {
      grid-template-columns: 1fr;
    }

    .split .columns .from,
    .split .columns .to {
      display: none;
    }

    .split .columns .both {
      display: block;
    }

    .split .source {
      order: 2;
      font-size: var(--text-md);
    }
  }
</style>
