#!/usr/bin/env python3
"""Scores a self-test log against a reference transcript.

Prints the word error rate to stdout as a bare number so shell scripts can use
it, and a readable breakdown to stderr.

Usage: wer.py <reference.txt> <selftest.log>
"""
import re
import sys


def normalize(text):
    """Lowercase words only. Punctuation and casing are whisper's choice, not
    evidence about what it heard."""
    text = text.lower().replace("-", " ")
    return re.sub(r"[^a-z0-9\s]", " ", text).split()


def edit_distance(reference, hypothesis):
    previous = list(range(len(hypothesis) + 1))
    for i, ref_word in enumerate(reference, start=1):
        current = [i] + [0] * len(hypothesis)
        for j, hyp_word in enumerate(hypothesis, start=1):
            cost = 0 if ref_word == hyp_word else 1
            current[j] = min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + cost)
        previous = current
    return previous[-1]


def main():
    reference = normalize(open(sys.argv[1]).read())
    lines = []
    for line in open(sys.argv[2]):
        match = re.search(r"SELFTEST phrase t=[\d.]+ \[(\d+) ms\] (.*)", line)
        if match:
            lines.append((int(match.group(1)), match.group(2)))

    hypothesis = normalize(" ".join(text for _, text in lines))
    if not reference:
        print("reference is empty", file=sys.stderr)
        return 2

    errors = edit_distance(reference, hypothesis)
    wer = errors / len(reference)

    print(f"{wer:.4f}")
    print(
        f"  {errors} errors in {len(reference)} words, {len(lines)} caption lines",
        file=sys.stderr,
    )
    for inference_ms, text in lines:
        print(f"    [{inference_ms:4d} ms] {text}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
