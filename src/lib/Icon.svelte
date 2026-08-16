<script lang="ts">
  /**
   * The icon set, drawn rather than imported.
   *
   * A handful of shapes at one weight is not worth a dependency, and an icon
   * font would put a second thing on the page that has to load before the
   * window looks right. All of them are on one 24-unit grid, in `currentColor`,
   * so they take the colour of whatever they sit next to and scale with the
   * text-size setting like everything else.
   *
   * All but one are a single stroke width. The GitHub mark is the exception -
   * it is somebody else's drawing and is filled, which is the only way it is
   * recognisable at this size.
   */
  export type IconName =
    | "history"
    | "settings"
    | "general"
    | "models"
    | "interface"
    | "about"
    | "down"
    | "github";

  let { name, size = "1.125rem" }: { name: IconName; size?: string } = $props();
</script>

<svg
  class="icon"
  style="--size: {size}"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "history"}
    <!-- A clock with an arrow going back around it. -->
    <path d="M3.2 9.4A9 9 0 1 1 3 12" />
    <path d="M3 4.5V9.4h4.9" />
    <path d="M12 7.6V12l3 1.8" />
  {:else if name === "settings"}
    <!-- Three sliders: the app's settings are choices along a row, not a gear. -->
    <path d="M4 7h9M17 7h3M4 17h3M11 17h9" />
    <circle cx="15" cy="7" r="2.2" />
    <circle cx="9" cy="17" r="2.2" />
  {:else if name === "general"}
    <!-- A speaker facing waves: the pipeline, from what is playing to what is heard. -->
    <path d="M4 9.5h3l4-3.2v11.4l-4-3.2H4z" />
    <path d="M15 9.2a4 4 0 0 1 0 5.6" />
    <path d="M17.8 6.6a8 8 0 0 1 0 10.8" />
  {:else if name === "models"}
    <!-- A chip: a model is a thing on disk that the machine runs. -->
    <rect x="7" y="7" width="10" height="10" rx="2.2" />
    <path d="M10 3.5v3M14 3.5v3M10 17.5v3M14 17.5v3M3.5 10h3M3.5 14h3M17.5 10h3M17.5 14h3" />
  {:else if name === "interface"}
    <!-- A window with a title bar. -->
    <rect x="3.2" y="4.8" width="17.6" height="14.4" rx="2.4" />
    <path d="M3.2 9.2h17.6M6.6 7h.01M9.4 7h.01" />
  {:else if name === "about"}
    <circle cx="12" cy="12" r="8.8" />
    <path d="M12 11v5.2M12 7.9h.01" />
  {:else if name === "down"}
    <path d="M12 5.5v13M6.8 13.2 12 18.5l5.2-5.3" />
  {:else if name === "github"}
    <!-- The GitHub mark, filled rather than stroked: it is their drawing, and
         outlined it stops being the thing anyone recognises. -->
    <path
      fill="currentColor"
      stroke="none"
      d="M12 1.3a10.7 10.7 0 0 0-3.38 20.86c.53.1.73-.23.73-.51l-.01-1.82c-2.98.65-3.6-1.44-3.6-1.44-.49-1.24-1.19-1.57-1.19-1.57-.97-.66.07-.65.07-.65 1.08.08 1.64 1.11 1.64 1.11.96 1.64 2.5 1.17 3.11.89.1-.69.37-1.16.68-1.43-2.37-.27-4.87-1.19-4.87-5.29 0-1.17.42-2.12 1.1-2.87-.11-.27-.48-1.36.1-2.83 0 0 .9-.29 2.94 1.1a10.2 10.2 0 0 1 5.36 0c2.04-1.39 2.94-1.1 2.94-1.1.58 1.47.21 2.56.1 2.83.69.75 1.1 1.7 1.1 2.87 0 4.11-2.5 5.02-4.89 5.28.38.33.72.98.72 1.98l-.01 2.94c0 .28.19.62.73.51A10.7 10.7 0 0 0 12 1.3Z"
    />
  {/if}
</svg>

<style>
  .icon {
    width: var(--size);
    height: var(--size);
    flex: none;
    display: block;
  }
</style>
