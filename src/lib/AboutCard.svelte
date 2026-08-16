<script lang="ts">
  /**
   * What this is, what it is made of, and which build of it you are looking at.
   *
   * Centred on the mark rather than lined up along the left edge: this is the
   * one panel in the app that is read once, and the only thing on it that is
   * ever needed twice is the version. The version is read from the bundle
   * rather than written here, so there is one number - the one in
   * `src-tauri/tauri.conf.json` - and a screenshot of this panel is enough to
   * say which build a bug report is about.
   */
  import { base } from "$app/paths";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { HOMEPAGE, ISSUES, LICENSE, NOTICES, appVersion, runtimeVersion } from "./api";
  import type { Translate } from "./i18n";

  let { t }: { t: Translate } = $props();

  let version = $state("");
  let runtime = $state("");

  /// What the app is actually built out of, in the order a reader meets it:
  /// the window, then the code behind it, then the two runtimes that do the
  /// work. Deliberately without versions - those change every release and are
  /// nobody's business here; the two that matter are on the rows above.
  ///
  /// Nine names, where the binary is built from several hundred packages. The
  /// full inventory with every license in it is a generated file, and the
  /// Notices button below is the way to it - a panel read once cannot be the
  /// place that lists them.
  const TECHNOLOGIES = [
    "Tauri",
    "Rust",
    "Svelte",
    "TypeScript",
    "Vite",
    "whisper.cpp",
    "llama.cpp",
    "Silero VAD",
    "Metal",
  ];

  // Outside a Tauri window - `npm run dev` in a browser - these are the only
  // calls on this panel, and a panel that throws is worse than one that is
  // missing a line.
  appVersion()
    .then((v) => (version = v))
    .catch(() => {});
  runtimeVersion()
    .then((v) => (runtime = v))
    .catch(() => {});
</script>

<div class="about">
  <img src="{base}/favicon.png" alt="" width="96" height="96" />
  <strong>Marswind</strong>
  <p class="tagline">{t("about.tagline")}</p>

  <!-- The version is here and not in the name above it: three facts of the same
       weight, and the name carrying one of them made it the only one anybody
       read. One line rather than three rows - the panel is read once, and
       stacking them pushed everything worth reading below the fold. -->
  <dl>
    {#if version}
      <div><dt>{t("about.version")}</dt><dd>{version}</dd></div>
    {/if}
    {#if runtime}
      <div><dt>{t("about.runtime")}</dt><dd>Tauri {runtime}</dd></div>
    {/if}
    <div><dt>{t("about.license")}</dt><dd>MIT</dd></div>
  </dl>

  <div class="built">
    <h3>{t("about.built")}</h3>
    <ul>
      {#each TECHNOLOGIES as name (name)}
        <li>{name}</li>
      {/each}
    </ul>
  </div>

  <!-- The names above say what it is made of; this says what it does with them.
       "It listens to your Mac" is the one claim in the app that sounds like it
       needs a driver, a microphone or a server behind it, and the answer to all
       three is on this panel rather than in a repository nobody opens. -->
  <div class="how">
    <h3>{t("about.how")}</h3>
    <p>{t("about.howCapture")}</p>
    <p>{t("about.howPipeline")}</p>
  </div>

  <div class="links">
    <button onclick={() => openUrl(HOMEPAGE)}>{t("about.source")}</button>
    <button onclick={() => openUrl(ISSUES)}>{t("about.issues")}</button>
    <button onclick={() => openUrl(LICENSE)}>{t("about.licenseFile")}</button>
    <button onclick={() => openUrl(NOTICES)}>{t("about.notices")}</button>
  </div>
</div>

<style>
  /* One column down the middle, held to a readable measure. Full width, a
     paragraph of tagline runs to a line nobody finishes. */
  .about {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 34rem;
    margin: 0 auto;
    padding: var(--space-6) 0 var(--space-4);
  }

  img {
    border-radius: var(--radius-lg);
    /* The mark is the largest thing on the panel, and the only picture in the
       app - worth the room. */
    width: 6rem;
    height: 6rem;
  }

  strong {
    display: block;
    margin-top: var(--space-4);
    font-size: var(--text-2xl);
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .tagline {
    margin: var(--space-2) 0 0;
    font-size: var(--text-md);
    line-height: 1.6;
    color: var(--muted);
  }

  /* All three on one line, each still labelled. They wrap rather than shrink,
     so a narrow window breaks them between pairs instead of hyphenating
     "Version". */
  dl {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-1) var(--space-4);
    margin: var(--space-4) 0 0;
    font-size: var(--text-md);
  }

  dl div {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  /* A separator between the pairs and not around them: the gap alone left three
     labels and three values reading as one run of words. */
  dl div + div::before {
    content: "·";
    /* An ink colour rather than the line colour it looks like: at this size a
       separator drawn in the colour of a border is not there at all. */
    color: var(--dim);
  }

  dt {
    color: var(--dim);
  }

  dd {
    margin: 0;
    color: var(--muted);
  }

  .built,
  .how {
    width: 100%;
    margin-top: var(--space-5);
    padding-top: var(--space-5);
    border-top: 1px solid var(--line);
  }

  /* Ranged left, alone on this panel: these are paragraphs rather than labels,
     and centred prose of this length is read down its ragged left edge. */
  .how p {
    margin: 0 0 var(--space-3);
    text-align: left;
    font-size: var(--text-md);
    line-height: 1.6;
    color: var(--muted);
  }

  .how p:last-child {
    margin-bottom: 0;
  }

  h3 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--dim);
  }

  /* Names, not versions. A version here would be a second place to keep them
     up to date and the first one to go stale. */
  ul {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-2);
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    padding: 0.1875rem var(--space-3);
    border-radius: 999px;
    background: var(--raised);
    border: 1px solid var(--line);
    font-size: var(--text-xs);
    line-height: 1.4;
    color: var(--muted);
    white-space: nowrap;
  }

  .links {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-2);
    margin-top: var(--space-5);
  }

  /* The technology list is the one thing here that runs long. Narrow, it wraps
     to three or four short rows, which is fine; the panel itself never gets
     narrower than its own margins. */
  @media (max-width: 40rem) {
    .about {
      padding-top: var(--space-5);
    }
  }
</style>
