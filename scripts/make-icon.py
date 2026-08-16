#!/usr/bin/env python3
"""Draw the application icon.

The icon is the toolbar's level indicator at rest: a dark well with a pool of
light in it, its surface crossed by two waves. Everything about it is computed
rather than drawn by hand, so the shape can be argued with - the corner is a
real superellipse and the waves are real sines.

    python3 scripts/make-icon.py

writes assets/icon.svg. `scripts/build-icons.sh` turns that into the PNG, the
.icns and the .ico the bundle actually ships.

Standard library only, on purpose: an icon that cannot be regenerated without a
package manager is an icon nobody regenerates.
"""

from __future__ import annotations

import math
from pathlib import Path

CANVAS = 1024
# macOS leaves the icon body short of the canvas edge and lets the system add
# the shadow. 832 of 1024 is the proportion Apple's own templates use.
BODY = 832
MARGIN = (CANVAS - BODY) / 2

# The exponent of the superellipse |x|^n + |y|^n = 1. Around four and a half is
# the continuous corner macOS has used since Big Sur - a circular corner (which
# is what an SVG `rx` gives) meets the straight edge at a visible crease, and
# that is the whole reason the shape is computed here rather than written out.
SQUIRCLE_N = 4.4
SQUIRCLE_SAMPLES = 256

# Where the surface of the pool sits, and how far the crests reach above it.
# Two waves at wavelengths that do not divide into each other, so the icon never
# looks like a repeating pattern.
#
# The back one rides a good deal higher than the front. Half a wavelength apart
# they would only cross, and at the sizes an icon is actually seen at - 32
# points in a dock, 16 in a menu - the green was a sliver behind the front wave
# rather than the second colour in the mark.
WAVES = [
    # baseline, amplitude, wavelength, phase
    (430.0, 56.0, 700.0, 0.62),
    (528.0, 38.0, 470.0, 0.12),
]


def squircle(cx: float, cy: float, radius: float) -> str:
    """The rounded-square body, as a closed path."""
    points = []
    for i in range(SQUIRCLE_SAMPLES):
        t = 2.0 * math.pi * i / SQUIRCLE_SAMPLES
        cos_t, sin_t = math.cos(t), math.sin(t)
        x = cx + radius * math.copysign(abs(cos_t) ** (2.0 / SQUIRCLE_N), cos_t)
        y = cy + radius * math.copysign(abs(sin_t) ** (2.0 / SQUIRCLE_N), sin_t)
        points.append(f"{x:.1f},{y:.1f}")
    return "M" + "L".join(points) + "Z"


def wave(baseline: float, amplitude: float, wavelength: float, phase: float) -> str:
    """One sine across the canvas, closed off at the bottom so it holds liquid."""
    points = []
    steps = 128
    for i in range(steps + 1):
        x = CANVAS * i / steps
        y = baseline - amplitude * math.sin(2.0 * math.pi * (x / wavelength + phase))
        points.append(f"{x:.1f},{y:.1f}")
    return "M" + "L".join(points) + f"L{CANVAS},{CANVAS}L0,{CANVAS}Z"


def build() -> str:
    body = squircle(CANVAS / 2, CANVAS / 2, BODY / 2)
    back = wave(*WAVES[0])
    front = wave(*WAVES[1])
    # The highest crest, so the glow above the water starts where the water does.
    crest = WAVES[0][0] - WAVES[0][1]

    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {CANVAS} {CANVAS}" width="{CANVAS}" height="{CANVAS}">
  <defs>
    <clipPath id="body">
      <path d="{body}"/>
    </clipPath>

    <!-- The well: not black, so the icon still has depth on a dark desktop. -->
    <linearGradient id="well" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#1c2333"/>
      <stop offset="0.55" stop-color="#111621"/>
      <stop offset="1" stop-color="#0a0d14"/>
    </linearGradient>

    <!-- One sweep of colour across the width, the same one the level indicator
         in the toolbar runs through. -->
    <linearGradient id="aurora" x1="0" y1="0.1" x2="1" y2="0.9">
      <stop offset="0" stop-color="#2ee6d4"/>
      <stop offset="0.26" stop-color="#38aaff"/>
      <stop offset="0.56" stop-color="#7b6bff"/>
      <stop offset="0.8" stop-color="#c15cff"/>
      <stop offset="1" stop-color="#ff5fb0"/>
    </linearGradient>

    <!-- The back wave runs warm against the front one's cool sweep, at the same
         pitch of colour: amber where that one is mint, orange where it is cyan.
         Two greens-to-blues crossing each other read as one wave with a fold in
         it - the point of the second is that you can see it is a second. -->
    <linearGradient id="aurora-back" x1="0" y1="0" x2="1" y2="0.7">
      <stop offset="0" stop-color="#ffd166"/>
      <stop offset="0.45" stop-color="#ffa63c"/>
      <stop offset="1" stop-color="#ff8348"/>
    </linearGradient>

    <!-- The light the water throws up the inside of the well, centred on the
         waterline so it reads as coming off the surface. -->
    <radialGradient id="glow" cx="0.5" cy="{crest / CANVAS:.3f}" r="0.62">
      <stop offset="0" stop-color="#63b4ff" stop-opacity="0.4"/>
      <stop offset="0.5" stop-color="#7a8bff" stop-opacity="0.13"/>
      <stop offset="1" stop-color="#7a8bff" stop-opacity="0"/>
    </radialGradient>

    <!-- One highlight, up and to the left, so the face reads as glass. -->
    <linearGradient id="gloss" x1="0.05" y1="0" x2="0.7" y2="1">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.2"/>
      <stop offset="0.4" stop-color="#ffffff" stop-opacity="0.03"/>
      <stop offset="1" stop-color="#ffffff" stop-opacity="0"/>
    </linearGradient>

    <!-- Depth in the water: darker where it is deep, so the crests read as the
         top of something rather than as a coloured stripe. -->
    <linearGradient id="depth" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#000000" stop-opacity="0"/>
      <stop offset="1" stop-color="#05070c" stop-opacity="0.45"/>
    </linearGradient>
  </defs>

  <g clip-path="url(#body)">
    <rect width="{CANVAS}" height="{CANVAS}" fill="url(#well)"/>

    <rect width="{CANVAS}" height="{CANVAS}" fill="url(#glow)"/>

    <path d="{back}" fill="url(#aurora-back)" opacity="0.9"/>
    <path d="{front}" fill="url(#aurora)"/>
    <path d="{front}" fill="url(#depth)"/>
    <!-- A line of light along the crest. Without it the front wave is a shape
         with colour in it rather than the top of something. The rest of this
         path runs outside the body and is clipped away. -->
    <path d="{front}" fill="none" stroke="#ffffff" stroke-opacity="0.3" stroke-width="4"/>

    <rect width="{CANVAS}" height="{CANVAS}" fill="url(#gloss)"/>
  </g>

  <!-- A hairline along the body, which is what keeps the icon from dissolving
       into a dark dock. -->
  <path d="{body}" fill="none" stroke="#ffffff" stroke-opacity="0.14" stroke-width="3"/>
</svg>
"""


def main() -> None:
    out = Path(__file__).resolve().parent.parent / "assets" / "icon.svg"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(build(), encoding="utf-8")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
