<script lang="ts">
  /**
   * The band under the toolbar that says which view you are in.
   *
   * History and Settings used to start differently - one with a list, one with
   * a heading - so moving between them felt like moving between two apps. They
   * now open the same way: the name of the view with its mark beside it, and
   * underneath, whatever that view needs to divide itself into.
   */
  import Icon, { type IconName } from "./Icon.svelte";

  let {
    icon,
    title,
    tabs,
  }: {
    icon: IconName;
    title: string;
    /** The section switcher, for a view that has sections. */
    tabs?: import("svelte").Snippet;
  } = $props();
</script>

<header class="view-head">
  <div class="title">
    <span class="mark"><Icon name={icon} size="1.0625rem" /></span>
    <h2>{title}</h2>
  </div>

  {#if tabs}
    <nav class="tabs">{@render tabs()}</nav>
  {/if}
</header>

<style>
  .view-head {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--gutter);
    border-bottom: 1px solid var(--line);
  }

  .title {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  /* The mark sits in its own tile rather than loose against the text: at this
     size a bare stroke icon reads as a smudge next to a bold word. */
  .mark {
    display: grid;
    place-items: center;
    width: var(--control);
    height: var(--control);
    border-radius: var(--radius);
    background: var(--raised);
    border: 1px solid var(--line);
    color: var(--muted);
    flex: none;
  }

  h2 {
    margin: 0;
    font-size: var(--text-xl);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Sections sit at the far end of the same band, so the view never grows a
     second bar to hold them. */
  .tabs {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    margin-left: auto;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  /* The buttons are passed in, so they are styled from here. */
  .tabs :global(button) {
    height: var(--control-small);
    padding: 0 var(--space-3);
    font-size: var(--text-sm);
    font-weight: 500;
    background: transparent;
    border-color: transparent;
    color: var(--dim);
  }

  .tabs :global(button:hover:not(:disabled)) {
    background: var(--raised);
    border-color: var(--line);
    color: var(--text);
  }

  .tabs :global(button.on) {
    background: var(--accent-soft);
    border-color: var(--accent-line);
    color: var(--accent-ink-soft);
  }

  @media (max-width: 44rem) {
    .view-head {
      flex-direction: column;
      align-items: stretch;
      gap: var(--space-3);
    }

    .tabs {
      margin-left: 0;
    }
  }
</style>
