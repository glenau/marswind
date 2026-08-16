#!/usr/bin/env python3
"""Scores one self-test log: recognition accuracy, translation quality, latency.

Recognition is scored with word error rate against the exact reference
transcript. Translation is scored with chrF - character n-gram F-score, the
standard metric when there is one reference and the target is a
morphologically rich language, where word-level metrics punish a correct
translation for choosing a different case ending.

Latency is reported against an estimate of when each line was spoken. The
fixtures are synthesised at a constant rate, so the position of a line's words
in the transcript maps to a time in the clip; that estimate is what makes an
end-to-end number possible at all, since the log only knows when text appeared.

Prints a JSON object to stdout and a readable breakdown to stderr.

Usage: score.py <name> <selftest.log> [--json]
"""
import json
import os
import re
import sys
import wave

FIXTURES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures")
# How far ahead in the reference a caption's next word may be looked for.
LOOKAHEAD = 12


# ---------------------------------------------------------------- recognition

def words_of(text):
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


# ---------------------------------------------------------------- translation

def chrf(reference, hypothesis, max_n=6, beta=2.0):
    """Character n-gram F-score, the sentence-level chrF of Popović 2015.

    Recall is weighted beta times precision: a translation that drops half the
    sentence should score far worse than one that adds a word.
    """
    reference = " ".join(reference.split())
    hypothesis = " ".join(hypothesis.split())
    if not reference or not hypothesis:
        return 0.0

    precisions, recalls = [], []
    for n in range(1, max_n + 1):
        ref_grams = ngrams(reference, n)
        hyp_grams = ngrams(hypothesis, n)
        if not ref_grams or not hyp_grams:
            continue
        overlap = sum(min(count, ref_grams.get(gram, 0)) for gram, count in hyp_grams.items())
        precisions.append(overlap / sum(hyp_grams.values()))
        recalls.append(overlap / sum(ref_grams.values()))

    if not precisions:
        return 0.0
    precision = sum(precisions) / len(precisions)
    recall = sum(recalls) / len(recalls)
    if precision + recall == 0:
        return 0.0
    return (1 + beta**2) * precision * recall / (beta**2 * precision + recall)


def ngrams(text, n):
    counts = {}
    for i in range(len(text) - n + 1):
        gram = text[i:i + n]
        counts[gram] = counts.get(gram, 0) + 1
    return counts


def cyrillic_ratio(text):
    letters = [c for c in text if c.isalpha()]
    if not letters:
        return 0.0
    return sum(1 for c in letters if "Ѐ" <= c <= "ӿ") / len(letters)


# ---------------------------------------------------------------------- input

PHRASE = re.compile(r"SELFTEST phrase t=([\d.]+) \[(\d+) ms\] (.*)")
UNIT = re.compile(r"SELFTEST unit t=([\d.]+) line=(\d+) (.*)")
# A row is translated in segments, so a translation is identified by its row
# and its place in it.
TRANSLATING = re.compile(r"SELFTEST translating t=([\d.]+) line=(\d+)\.(\d+) (.*)")
TRANSLATED = re.compile(
    r"SELFTEST translated t=([\d.]+) line=(\d+)\.(\d+) \[(\d+) ms to first word, (\d+) ms total\] (.*?)  ->  (.*)"
)
# Older logs did not carry the line number on a finished translation.
TRANSLATED_OLD = re.compile(
    r"SELFTEST translated t=([\d.]+) \[(\d+) ms to first word, (\d+) ms total\] (.*?)  ->  (.*)"
)


def read_log(path):
    phrases, units, translating, translated = [], [], {}, []
    for line in open(path, errors="replace"):
        if match := PHRASE.search(line):
            phrases.append((float(match.group(1)), int(match.group(2)), match.group(3)))
        elif match := UNIT.search(line):
            units.append((float(match.group(1)), int(match.group(2)), match.group(3)))
        elif match := TRANSLATING.search(line):
            key = (int(match.group(2)), int(match.group(3)))
            translating.setdefault(key, float(match.group(1)))
        elif match := TRANSLATED.search(line):
            translated.append({
                "t": float(match.group(1)),
                "line": (int(match.group(2)), int(match.group(3))),
                "first_word_ms": int(match.group(4)),
                "total_ms": int(match.group(5)),
                "source": match.group(6),
                "text": match.group(7),
            })
        elif match := TRANSLATED_OLD.search(line):
            translated.append({
                "t": float(match.group(1)),
                "line": (len(translated), 0),
                "first_word_ms": int(match.group(2)),
                "total_ms": int(match.group(3)),
                "source": match.group(4),
                "text": match.group(5),
            })
    return phrases, units, translating, translated


def median(values):
    if not values:
        return None
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


# --------------------------------------------------------------------- timing

def spoken_estimate(reference, hypothesis_lines, duration):
    """When each line finished being spoken.

    Position in the *reference* is what maps onto time, not position in the
    hypothesis. Measuring against the hypothesis instead looks reasonable and
    is a trap: a run that dropped a third of the words spreads the rest over
    the whole clip and scores as though it were fast.

    The fixtures are synthesised at a constant rate, so a reference word index
    converts to a time directly. A caption's words are matched forward through
    the reference, and words the recognizer lost simply advance the cursor.
    """
    estimates, cursor = [], 0
    for text in hypothesis_lines:
        for word in words_of(text):
            window = reference[cursor:cursor + LOOKAHEAD]
            if word in window:
                cursor += window.index(word) + 1
        estimates.append(min(cursor, len(reference)) / len(reference) * duration)
    return estimates


def main():
    name = sys.argv[1]
    log = sys.argv[2]

    reference = words_of(open(os.path.join(FIXTURES, f"{name}.txt")).read())
    russian_path = os.path.join(FIXTURES, f"{name}.ru.txt")
    russian = open(russian_path).read().strip() if os.path.exists(russian_path) else ""
    with wave.open(os.path.join(FIXTURES, f"{name}.wav")) as clip:
        duration = clip.getnframes() / clip.getframerate()

    phrases, units, translating, translated = read_log(log)

    hypothesis = words_of(" ".join(text for _, _, text in phrases))
    errors = edit_distance(reference, hypothesis)
    result = {
        "clip": name,
        "wer": errors / len(reference) if reference else 1.0,
        "lines": len(phrases),
        "words": len(hypothesis),
        "inference_ms_median": median([ms for _, ms, _ in phrases]),
    }

    # Every translated unit joined back together: the reader sees a stream, and
    # what matters is whether the whole stream says what the speaker said.
    joined = " ".join(item["text"] for item in translated).strip()
    if russian:
        result["chrf"] = chrf(russian, joined)
        result["cyrillic"] = cyrillic_ratio(joined)
    result["translated_units"] = len(translated)
    result["translation_ms_median"] = median([item["total_ms"] for item in translated])
    result["first_word_ms_median"] = median([item["first_word_ms"] for item in translated])

    # End-to-end: from the moment a line finished being spoken to the moment its
    # first translated word was on screen. This is the number the reader feels.
    spoken = spoken_estimate(reference, [text for _, _, text in phrases], duration)
    asr_lag = [t - estimate for (t, _, _), estimate in zip(phrases, spoken)]
    result["asr_lag_median"] = median(asr_lag)

    # Translation units may be finer than caption lines, so a unit is timed
    # against the caption whose text contains it.
    e2e_first, e2e_full = [], []
    for item in translated:
        estimate = spoken_for(item["source"], phrases, spoken)
        if estimate is None:
            continue
        start = translating.get(item["line"])
        if start is not None:
            e2e_first.append(start - estimate)
        e2e_full.append(item["t"] - estimate)
    result["e2e_first_word_median"] = median(e2e_first)
    result["e2e_complete_median"] = median(e2e_full)

    # Three numbers that need no alignment at all, and so carry none of its
    # guesswork: the clock starts when playback starts and the clip length is
    # known exactly.
    starts = sorted(translating.values())
    result["first_subtitle"] = starts[0] if starts else None
    result["refresh_median"] = median([b - a for a, b in zip(starts, starts[1:])])
    result["tail_lag"] = (translated[-1]["t"] - duration) if translated else None

    print(json.dumps(result, ensure_ascii=False))

    out = sys.stderr
    print(f"  {name}: WER {result['wer'] * 100:.1f}%  "
          f"({errors} errors in {len(reference)} words, {len(phrases)} lines)", file=out)
    if russian:
        print(f"  chrF {result['chrf'] * 100:.1f}  cyrillic {result['cyrillic'] * 100:.0f}%  "
              f"units {len(translated)}", file=out)
    for t, ms, text in phrases:
        print(f"    ASR  t={t:6.2f} [{ms:4d} ms] {text}", file=out)
    for item in translated:
        start = translating.get(item["line"])
        began = f"{start:6.2f}" if start is not None else "     -"
        print(f"    MT   t={item['t']:6.2f} (first word t={began}) {item['text']}", file=out)
    return 0


def spoken_for(source, phrases, spoken):
    """The estimated end time of the caption a translation unit came from."""
    needle = " ".join(words_of(source))
    if not needle:
        return None
    for (_, _, text), estimate in zip(phrases, spoken):
        if needle in " ".join(words_of(text)):
            return estimate
    return spoken[-1] if spoken else None


if __name__ == "__main__":
    sys.exit(main())
