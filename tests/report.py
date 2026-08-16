#!/usr/bin/env python3
"""Turns the JSON scores in a run directory into one table.

Usage: report.py <directory> [<directory> ...]

With several directories the columns are grouped by label, which is how a
change is judged: the same clips, before and after.
"""
import glob
import json
import os
import sys


def median(values):
    values = [v for v in values if v is not None]
    if not values:
        return None
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def load(directory):
    runs = {}
    for path in sorted(glob.glob(os.path.join(directory, "*.json"))):
        label = os.path.basename(path).split("-")[0]
        with open(path) as handle:
            score = json.load(handle)
        runs.setdefault((label, score["clip"]), []).append(score)
    return runs


def spread(values):
    """Every WER behind a median. One number hides how wide the range is, and on
    this corpus the range is the story."""
    if len(values) < 2:
        return ""
    return "   (" + ", ".join(f"{value * 100:.1f}%" for value in sorted(values)) + ")"


def cell(value, scale=1.0, unit="", width=7):
    if value is None:
        return "-".rjust(width)
    return f"{value * scale:.1f}{unit}".rjust(width)


def main():
    runs = {}
    for directory in sys.argv[1:]:
        for key, scores in load(directory).items():
            runs.setdefault(key, []).extend(scores)

    if not runs:
        print("no scores found")
        return 1

    header = (
        f"{'LABEL':<10} {'CLIP':<20} {'WER':>7} {'chrF':>7} {'1st sub':>8} "
        f"{'refresh':>8} {'wait':>8} {'tail':>8} {'e2e 1st':>8} {'e2e all':>8} {'units':>6}"
    )
    print(header)
    print("-" * len(header))

    for (label, clip), scores in sorted(runs.items()):
        print(
            f"{label:<10} {clip:<20} "
            f"{cell(median([s['wer'] for s in scores]), 100, '%')} "
            f"{cell(median([s.get('chrf') for s in scores]), 100)} "
            f"{cell(median([s.get('first_subtitle') for s in scores]), 1, 's', 8)} "
            f"{cell(median([s.get('refresh_median') for s in scores]), 1, 's', 8)} "
            f"{cell(median([s.get('first_word_ms_median') for s in scores]), 0.001, 's', 8)} "
            f"{cell(median([s.get('tail_lag') for s in scores]), 1, 's', 8)} "
            f"{cell(median([s.get('e2e_first_word_median') for s in scores]), 1, 's', 8)} "
            f"{cell(median([s.get('e2e_complete_median') for s in scores]), 1, 's', 8)} "
            f"{median([s['translated_units'] for s in scores]):>6}"
            + spread([s["wer"] for s in scores])
        )

    for label in sorted({label for label, _ in runs}):
        scores = [s for (l, _), group in runs.items() if l == label for s in group]
        print(
            f"\n{label}: WER median {median([s['wer'] for s in scores]) * 100:.1f}%, "
            f"chrF median {median([s.get('chrf') for s in scores]) * 100:.1f}, "
            f"first subtitle {median([s.get('first_subtitle') for s in scores]):.1f}s into the clip, "
            f"refreshing every {median([s.get('refresh_median') for s in scores]):.1f}s, "
            f"{median([s.get('first_word_ms_median') for s in scores]) / 1000:.1f}s from a finished "
            f"line to its first translated word"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
