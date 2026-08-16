<script lang="ts">
  let { children } = $props();
</script>

{@render children()}

<style>
  /* One surface, one scale.
   *
   * Every size in the app is a rem, and the root font size is a single number
   * multiplied by the user's text-size setting - so changing that setting moves
   * text, control heights, padding and radii together instead of leaving
   * buttons the size they were around bigger words.
   *
   * There is also only one background colour. Panels, toolbars and dialogs used
   * to each have their own, which is what made the window read as a stack of
   * unrelated boxes; separation is done with hairlines and, where something is
   * genuinely raised, one step of lift. */
  :global(html) {
    font-size: calc(16px * var(--ui-scale, 1));
  }

  :global(html),
  :global(body) {
    height: 100%;
    margin: 0;
    overflow: hidden;
  }

  :global(:root) {
    /* Everything the engine draws itself - scrollbars, dropdown menus, the
       checkbox, focus rings - follows this. Without it the page is assumed to
       be light, and the native scrollbar comes back as a dark thumb meant for a
       white background, which on this one reads as a black bar. It is set with
       the theme below, and the dark values here are what the window opens in
       before the preference has been read. */
    color-scheme: dark;

    --ui-scale: 1;

    /* Surfaces */
    --bg: #101216;
    --raised: #191c22;
    --raised-hover: #1f232a;
    --selected: #232935;

    /* Lines. Two weights: one you are not meant to notice, one you are. */
    --line: #21252d;
    --line-strong: #2e343e;

    /* Text */
    --text: #edeff3;
    --muted: #99a1af;
    --dim: #666e7c;

    --accent: #4c7dfd;
    --accent-hover: #6a92ff;
    --accent-ink: #ffffff;
    --danger: #d0524e;
    --danger-hover: #de625e;
    --warn: #c39a5e;
    --ok: #48c06c;

    /* The tinted states, as properties rather than as an `rgba()` written into
       whichever component needed it. There are only three of these and every
       one of them has to change with the theme - a translucent white over a
       dark panel becomes a translucent white over a white one, which is
       nothing at all. */
    --accent-soft: rgba(76, 125, 253, 0.16);
    --accent-soft-hover: rgba(76, 125, 253, 0.24);
    --accent-line: rgba(76, 125, 253, 0.55);
    --accent-line-hover: rgba(76, 125, 253, 0.7);
    --accent-ink-soft: #b7cbff;

    /* Badges and the error strip: an ink and a line each, one pair per meaning.
       Named for what they say rather than for their colour, so the light theme
       can pick a different shade of the same idea. */
    --info-ink: #9fc0ff;
    --info-line: #2e4270;
    --warn-line: #4a3d27;
    --ok-ink: #a9d8b8;
    --ok-line: #274a33;
    --error-bg: #2a1a1a;
    --error-line: #4a2626;
    --error-ink: #f0b4a8;

    /* The unlit stage indicator, and the one shadow in the app. */
    --dot-off: #3a4049;
    --lift: 0 0.5rem 1.25rem rgba(0, 0, 0, 0.45);
  }

  /* The light theme.
   *
   * The same interface, not a second one: every rule in the app reads these
   * properties, so this block is the whole of it. Surfaces run the other way -
   * the page is the lightest thing and a raised panel is a shade darker, which
   * is what keeps a hairline visible without turning every card into a box. */
  :global(:root[data-theme="light"]) {
    color-scheme: light;

    --bg: #ffffff;
    --raised: #f4f5f7;
    --raised-hover: #eceef2;
    --selected: #e3e9f6;

    --line: #e2e5ea;
    --line-strong: #cdd2da;

    --text: #16181d;
    --muted: #565d6b;
    --dim: #7c8492;

    --accent: #2f62e8;
    --accent-hover: #234fc8;
    --accent-ink: #ffffff;
    --danger: #c53a36;
    --danger-hover: #ad2f2c;
    --warn: #9a6b1f;
    --ok: #2b9a4f;

    --accent-soft: rgba(47, 98, 232, 0.1);
    --accent-soft-hover: rgba(47, 98, 232, 0.16);
    --accent-line: rgba(47, 98, 232, 0.45);
    --accent-line-hover: rgba(47, 98, 232, 0.6);
    --accent-ink-soft: #23479c;

    --info-ink: #2f5bbf;
    --info-line: #bcd0f5;
    --warn-line: #e3d0a8;
    --ok-ink: #1f7a41;
    --ok-line: #b8dfc4;
    --error-bg: #fdf0ee;
    --error-line: #f2cec7;
    --error-ink: #9c3a2c;

    --dot-off: #c2c7d0;
    --lift: 0 0.5rem 1.25rem rgba(20, 24, 33, 0.16);
  }

  /* Everything below is the same in both themes: sizes, not colours. */
  :global(:root) {
    /* One spacing scale. Anything not on it is a mistake. */
    --space-1: 0.25rem;
    --space-2: 0.5rem;
    --space-3: 0.75rem;
    --space-4: 1rem;
    --space-5: 1.5rem;
    --space-6: 2rem;

    /* One type scale. */
    --text-xs: 0.6875rem; /* labels, badges */
    --text-sm: 0.8125rem; /* secondary text */
    --text-md: 0.875rem; /* controls, body of dense lists */
    --text-lg: 1rem; /* the transcript, headings inside a view */
    --text-xl: 1.125rem; /* the name of the view you are in */
    --text-2xl: 1.375rem; /* the About panel's name, and nothing else */

    /* One control height and one radius, so a row of anything lines up. */
    --control: 2.125rem;
    --control-small: 1.75rem;
    --radius: 0.5rem;
    --radius-lg: 0.75rem;

    /* The page margin, shared by every header, row and footer so their edges
       line up down the window. */
    /* One margin for everything: toolbar, column headers, rows, footer and the
       settings body all start and end on it, so their edges line up down the
       window. */
    --gutter: var(--space-4);
    /* Twice the gutter, so that split down the middle it leaves each column the
       same margin against the rule as the window gives it against the edge. A
       single gutter's worth put the two texts closer to each other than either
       was to the outside, and the pair read as one crowded block. */
    --column-gap: calc(var(--gutter) * 2);
  }

  :global(body) {
    background: var(--bg);
    color: var(--text);
    font-family:
      -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", system-ui, sans-serif;
    font-size: var(--text-md);
    -webkit-font-smoothing: antialiased;
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  /* One button. Variants change colour, never size. */
  :global(button) {
    font: inherit;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: var(--control);
    padding: 0 var(--space-2);
    font-size: var(--text-md);
    font-weight: 500;
    color: var(--text);
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    cursor: pointer;
    white-space: nowrap;
    transition:
      background 120ms ease,
      border-color 120ms ease,
      color 120ms ease;
  }

  :global(button:hover:not(:disabled)) {
    background: var(--raised-hover);
    border-color: var(--line-strong);
  }

  :global(button:disabled) {
    opacity: 0.4;
    cursor: default;
  }

  :global(button.primary) {
    background: var(--accent);
    border-color: transparent;
    color: var(--accent-ink);
  }

  :global(button.primary:hover:not(:disabled)) {
    background: var(--accent-hover);
    border-color: transparent;
  }

  :global(button.destructive) {
    background: var(--danger);
    border-color: transparent;
    color: var(--accent-ink);
  }

  :global(button.destructive:hover:not(:disabled)) {
    background: var(--danger-hover);
    border-color: transparent;
  }

  /* A button that reads as text until you need it. */
  :global(button.quiet) {
    background: transparent;
    border-color: transparent;
    color: var(--muted);
  }

  :global(button.quiet:hover:not(:disabled)) {
    background: var(--raised);
    border-color: var(--line);
    color: var(--text);
  }

  /* The view you are looking at. Unmistakable rather than tasteful: a tab that
     is only slightly lighter than the others is a tab nobody can find. */
  :global(button.on) {
    background: var(--accent-soft);
    border-color: var(--accent-line);
    color: var(--accent-ink-soft);
  }

  :global(button.on:hover:not(:disabled)) {
    background: var(--accent-soft-hover);
    border-color: var(--accent-line-hover);
  }

  :global(select) {
    font: inherit;
    height: var(--control);
    padding: 0 var(--space-3);
    font-size: var(--text-md);
    color: var(--text);
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--radius);
  }

  :global(select:disabled) {
    opacity: 0.5;
  }

  :global(input[type="checkbox"]) {
    width: 0.875rem;
    height: 0.875rem;
    margin: 0;
    accent-color: var(--accent);
  }

  /* The scrollbar is deliberately left unstyled.
   *
   * Any `::-webkit-scrollbar` rule opts the page into the classic scrollbar,
   * which takes its width out of the content - that is what was pushing the
   * right-hand side of every panel inwards. Left alone, macOS uses its overlay
   * scrollbar, which floats above the content and takes nothing. Scrolling
   * areas also ask for `scrollbar-gutter: stable both-edges`, which does
   * nothing while the scrollbars are overlay and keeps the content centred if
   * the system is set to show them always. */
</style>
