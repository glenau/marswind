<script lang="ts">
  /**
   * The input level, drawn as the app icon with the tide coming in.
   *
   * A bar only ever said how wide the window was, and a dot that changed size
   * said nothing at all while it was quiet. This is the same picture as the
   * icon - a dark well, a pool of aurora light in it, two waves crossing the
   * surface - with one thing added: the waterline rises with whatever the
   * machine is playing.
   *
   * It is deliberately still recognisable as the mark when nothing is running.
   * The waterline moves within a band around where the icon draws it and never
   * reaches the top, because an orb filled to the brim stops looking like a
   * circle with something in it and starts looking like a circle that leaked.
   *
   * All of it is CSS - a gradient masked by a sine. Nothing here runs on the
   * main thread while the pipeline is busy recognizing speech, which is the
   * reason it is not a canvas.
   */
  let {
    level,
    running,
    title = "",
  }: { level: number; running: boolean; title?: string } = $props();

  /// Speech spends most of its time in the bottom third of a linear level, so
  /// the raw value is eased - otherwise the surface only moves for shouting.
  const fill = $derived(running ? Math.min(1, Math.max(0, level)) ** 0.55 : 0);

  /// Where the waterline sits, as a percentage down the orb. Silence rests just
  /// under halfway, which is where the icon draws it; the loudest peak is a
  /// quarter of the orb higher, and no further.
  const surface = $derived((1 - (0.48 + fill * 0.24)) * 100);
</script>

<div
  class="orb"
  class:running
  style="--surface: {surface}%; --fill: {fill}"
  role="img"
  aria-label={title}
  {title}
>
  <span class="halo"></span>
  <span class="well">
    <span class="wave back"></span>
    <span class="wave front"></span>
    <span class="crest"></span>
    <span class="gloss"></span>
  </span>
  <span class="rim"></span>
</div>

<style>
  .orb {
    position: relative;
    width: calc(var(--control) * 1.2);
    height: calc(var(--control) * 1.2);
    flex: none;
    /* At rest it is the mark, a shade calmer - not a greyed-out version of it.
       What says "not listening" is that the light around it has gone out and
       the tide is at its lowest, which is the same thing the icon shows. */
    filter: saturate(0.88) brightness(0.86);
    opacity: 0.92;
    transition:
      filter 400ms ease,
      opacity 400ms ease;
  }

  .orb.running {
    filter: none;
    opacity: 1;
  }

  /* The light the pool throws. Kept tight against the rim: a wide bloom reads
     as the orb having spilled rather than as the orb being bright. */
  .halo {
    position: absolute;
    inset: -18%;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(120, 160, 255, 0.55), transparent 62%);
    opacity: calc(var(--fill) * 0.75);
    filter: blur(0.25rem);
    transition: opacity 360ms ease;
    pointer-events: none;
  }

  /* Everything below this is cut to the circle, three ways.
     WebKit - which is what the app runs in, Chromium only what it is developed
     in - does not clip a child that has been promoted to its own compositing
     layer, and both waves have been: they animate a transform and one of them
     blends. `overflow: hidden` let the liquid out as a rectangle; adding
     `clip-path` still left a crescent of it past the rim on a loud peak.
     The mask is what actually holds. A clip is a test applied per layer, and a
     layer the compositor owns never gets asked; a mask makes the engine render
     the whole subtree into a buffer and cut *that*, so there is nothing left to
     escape. The other two stay because they cost nothing and are what other
     engines use.
     `isolation` keeps the blending below inside the circle rather than letting
     it reach the toolbar behind. */
  .well {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    overflow: hidden;
    -webkit-clip-path: circle(50% at 50% 50%);
    clip-path: circle(50% at 50% 50%);
    /* Hard to 97% and out by 100%: a sliver of a percent to antialias the edge,
       which at this size is about half a pixel. */
    -webkit-mask-image: radial-gradient(circle closest-side, #000 97%, transparent 100%);
    mask-image: radial-gradient(circle closest-side, #000 97%, transparent 100%);
    isolation: isolate;
    /* The same well the icon is drawn in, so the empty part above the water is
       a lit surface rather than a hole. */
    background: linear-gradient(180deg, #1c2333, #111621 55%, #0a0d14);
    /* The shimmer: the whole well swings through a slice of the spectrum. Long
       and gentle - at this size a fast hue sweep reads as a fault rather than
       as light. The wave layers only move the water, they do not colour it. */
    animation: shimmer 24s ease-in-out infinite alternate;
  }

  /* A gradient cut off at the top by a sine and solid below it. Every period is
     in the one image, stretched across the whole element, rather than one
     period tiled - a tiled mask antialiases the edge of each tile, and the two
     half-covered columns where two tiles met showed up as a hairline down the
     middle of the circle.
     The element is four circles wide with the spare three off to the left, so
     a wave can travel a full period without its own edge coming into view.
     28% is where the sine's baseline falls inside the image, so the waterline
     lands on `--surface` rather than somewhere under it. Both waves share the
     baseline, so they share the offset. */
  .wave,
  .crest {
    position: absolute;
    left: -150%;
    width: 400%;
    top: calc(var(--surface) - 28%);
    height: 200%;
    -webkit-mask-size: 100% 100%;
    mask-size: 100% 100%;
    -webkit-mask-repeat: no-repeat;
    mask-repeat: no-repeat;
    /* Peaks arrive quickly and drain slowly, which is what makes a level
       readable rather than a strobe. Slow enough to be a rise rather than a
       jump: the meter is 40 pixels across and a step in it is visible. The
       level itself is eased before it gets here, so this is the second half of
       the smoothing rather than all of it. */
    transition: top 260ms cubic-bezier(0.25, 0.6, 0.25, 1);
    pointer-events: none;
  }

  /* Half again as many periods as the back one, so the crests drift in and out
     of step instead of travelling together. Both are drawn much shallower and
     longer than a meter would have them - this is the icon's water, which
     swells rather than ripples. */
  .wave.front,
  .crest {
    -webkit-mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 100' preserveAspectRatio='none'%3E%3Cpath d='M0,14 Q25,9 50,14 T100,14 T150,14 T200,14 T250,14 T300,14 T350,14 T400,14 T450,14 T500,14 T550,14 T600,14 L600,100 L0,100 Z' fill='%23fff'/%3E%3C/svg%3E");
    mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 100' preserveAspectRatio='none'%3E%3Cpath d='M0,14 Q25,9 50,14 T100,14 T150,14 T200,14 T250,14 T300,14 T350,14 T400,14 T450,14 T500,14 T550,14 T600,14 L600,100 L0,100 Z' fill='%23fff'/%3E%3C/svg%3E");
    animation: flow 11s linear infinite;
  }

  .wave.front {
    /* Depth over colour: darker the further down, so the crest reads as the top
       of something rather than as a coloured stripe. Two sweeps of the ramp
       across the element - one every two circles - and the ramp is a palindrome
       so the repeats meet without a seam. */
    background-image:
      linear-gradient(180deg, rgba(5, 7, 12, 0) 20%, rgba(5, 7, 12, 0.5)),
      linear-gradient(
        90deg,
        #2ee6d4,
        #38aaff,
        #7b6bff,
        #c15cff,
        #ff5fb0,
        #c15cff,
        #7b6bff,
        #38aaff,
        #2ee6d4
      );
    background-size:
      100% 100%,
      50% 100%;
    /* The colour travels through the water at its own pace, rather than being
       nailed to the shape of it. This is what makes the change of colour read
       as light moving over a surface instead of a texture sliding past. */
    animation:
      flow 11s linear infinite,
      tint 31s linear infinite;
  }

  .wave.back {
    /* Warm against the front wave's cool sweep, at the same pitch of colour:
       amber where that one is mint, orange where it is cyan. Two greens-to-
       blues crossing each other only ever read as one wave with a fold in it -
       the point of the second wave is that you can see it is a second wave. */
    /* The icon's ramp, to the digit. Deepening it to compensate for the screen
       blend was the obvious move and the wrong one: the well swings its hue
       either side of where it sits, and an orange with the light taken out of
       it goes olive at one end of that swing. Bright, it stays orange
       throughout. */
    background-image: linear-gradient(90deg, #ffd166, #ffa63c, #ff8348, #ffa63c, #ffd166);
    background-size: 50% 100%;
    /* Standing clear of the front wave it is against the dark well rather than
       against colour, and at the old three quarters it went grey there - the
       same reason the icon's back wave was lifted from 0.7 to 0.8. */
    opacity: 0.85;
    /* Where the two overlap they add up, so the crossings are the brightest
       part of the orb - which is what a real meter does at the peak. */
    mix-blend-mode: screen;
    /* Its trough where the front has its crest, a longer period, and a baseline
       four points above the front's - so it crosses rather than runs parallel,
       and stands clear of the front wave instead of hiding behind it. */
    -webkit-mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 100' preserveAspectRatio='none'%3E%3Cpath d='M0,10 Q37.5,16 75,10 T150,10 T225,10 T300,10 T375,10 T450,10 T525,10 T600,10 L600,100 L0,100 Z' fill='%23fff'/%3E%3C/svg%3E");
    mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 100' preserveAspectRatio='none'%3E%3Cpath d='M0,10 Q37.5,16 75,10 T150,10 T225,10 T300,10 T375,10 T450,10 T525,10 T600,10 L600,100 L0,100 Z' fill='%23fff'/%3E%3C/svg%3E");
    animation: ebb 17s linear infinite;
  }

  /* The line of light along the crest - the same one the icon has, and the
     single detail that most makes the two read as one drawing. It is the front
     wave's own path stroked instead of filled, so it tracks it exactly. */
  .crest {
    background: #ffffff;
    opacity: 0.42;
    -webkit-mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 100' preserveAspectRatio='none'%3E%3Cpath d='M0,14 Q25,9 50,14 T100,14 T150,14 T200,14 T250,14 T300,14 T350,14 T400,14 T450,14 T500,14 T550,14 T600,14' fill='none' stroke='%23fff' stroke-width='2.4'/%3E%3C/svg%3E");
    mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 100' preserveAspectRatio='none'%3E%3Cpath d='M0,14 Q25,9 50,14 T100,14 T150,14 T200,14 T250,14 T300,14 T350,14 T400,14 T450,14 T500,14 T550,14 T600,14' fill='none' stroke='%23fff' stroke-width='2.4'/%3E%3C/svg%3E");
  }

  /* One highlight, up and to the left, so the circle reads as glass. */
  .gloss {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: radial-gradient(circle at 32% 24%, rgba(255, 255, 255, 0.22), transparent 46%);
    pointer-events: none;
  }

  .rim {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 1px solid var(--line-strong);
    /* Brightens with the level, so the whole shape swells a little rather than
       only the water inside it. */
    box-shadow: 0 0 0 calc(var(--fill) * 0.125rem) rgba(91, 140, 255, 0.16);
    transition: box-shadow 360ms ease;
    pointer-events: none;
  }

  /* Each of these moves the mask by exactly one period, so the loop is seamless
     and the element - four times the width of the circle - never shows an
     edge. */
  @keyframes flow {
    to {
      transform: translateX(16.6667%);
    }
  }

  @keyframes ebb {
    to {
      transform: translateX(-25%);
    }
  }

  /* One full width of the colour ramp, which is its own repeat: the loop closes
     on itself and the hue never jumps. The depth layer above it does not move,
     hence the first pair. */
  @keyframes tint {
    from {
      background-position:
        0 0,
        0 0;
    }
    to {
      background-position:
        0 0,
        50% 0;
    }
  }

  @keyframes shimmer {
    from {
      filter: hue-rotate(-14deg);
    }
    to {
      filter: hue-rotate(18deg);
    }
  }

  /* The level is still readable standing still - it is the height of the water,
     not the motion, that carries it. */
  @media (prefers-reduced-motion: reduce) {
    .well,
    .wave,
    .crest {
      animation: none;
    }
  }
</style>
