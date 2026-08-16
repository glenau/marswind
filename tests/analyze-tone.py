#!/usr/bin/env python3
"""Checks a captured recording of the 440 Hz reference tone.

Three things must hold for the capture path to be trustworthy:

* the amplitude matches the source, so nothing scales or clips the signal,
* the dominant frequency is still 440 Hz, so the resampler is not shifting it,
* the harmonics stay near zero, so downsampling is not folding energy back into
  the speech band as aliasing.

Usage: analyze-tone.py <captured.wav>
"""
import math
import struct
import sys
import wave

EXPECTED_FREQUENCY = 440.0
EXPECTED_AMPLITUDE = 0.35
AMPLITUDE_TOLERANCE = 0.03
FREQUENCY_TOLERANCE = 2.0
# Harmonic energy this far below the fundamental is inaudible and well under
# what a broken resample would produce.
MAX_HARMONIC_RATIO = 0.01


def goertzel(samples, frequency, rate):
    coefficient = 2 * math.cos(2 * math.pi * frequency / rate)
    s1 = s2 = 0.0
    for sample in samples:
        s0 = sample + coefficient * s1 - s2
        s2, s1 = s1, s0
    return s1 * s1 + s2 * s2 - coefficient * s1 * s2


def main():
    with wave.open(sys.argv[1]) as source:
        rate = source.getframerate()
        frames = source.getnframes()
        raw = source.readframes(frames)

    samples = [value / 32768 for value in struct.unpack(f"<{frames}h", raw)]
    if not samples:
        print("captured file is empty")
        return 1

    # Skip the first moment: playback and capture do not start on the same edge.
    steady = samples[rate // 2 : rate // 2 + 8192]
    peak = max(abs(sample) for sample in steady)

    fundamental = goertzel(steady, EXPECTED_FREQUENCY, rate)
    dominant = max(range(200, min(4000, rate // 2), 2), key=lambda f: goertzel(steady, f, rate))
    harmonics = max(
        goertzel(steady, EXPECTED_FREQUENCY * n, rate) / fundamental for n in (2, 3, 4)
    )

    print(f"    sample rate     {rate} Hz")
    print(f"    peak amplitude  {peak:.4f} (source {EXPECTED_AMPLITUDE})")
    print(f"    dominant tone   {dominant} Hz")
    print(f"    harmonic energy {100 * harmonics:.3f}% of the fundamental")

    failures = []
    if abs(peak - EXPECTED_AMPLITUDE) > AMPLITUDE_TOLERANCE:
        failures.append(f"amplitude {peak:.4f} differs from the source {EXPECTED_AMPLITUDE}")
    if abs(dominant - EXPECTED_FREQUENCY) > FREQUENCY_TOLERANCE:
        failures.append(f"dominant frequency {dominant} Hz, expected {EXPECTED_FREQUENCY}")
    if harmonics > MAX_HARMONIC_RATIO:
        failures.append(f"harmonic energy {100 * harmonics:.2f}% suggests aliasing")

    for failure in failures:
        print(f"    FAIL {failure}")
    if failures:
        return 1

    print("    pass")
    return 0


if __name__ == "__main__":
    sys.exit(main())
