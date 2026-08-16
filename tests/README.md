# Tests

Rust unit tests cover the pure logic - ring buffer, resampler, word agreement,
sentence boundaries, model catalog:

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

The first line is a one-off. `src-tauri` bundles the translation worker as a
Tauri sidecar, so nothing in it compiles - tests included - until that binary
exists.

Everything below is shell and `python3` with nothing but the standard library
behind it - no `pip install`, no virtualenv. The scoring is a hundred lines of
arithmetic and worth reading rather than trusting.

The scripts here cover what unit tests cannot: whether the app, running as a
real signed bundle, hears what the machine plays and turns it into the right
words. Both play audio through the system output and capture it back through
the tap, the resampler and the model - the same path a user gets.

They need a built and signed bundle:

```bash
npm run dev:macos
```

## Capture fidelity

```bash
tests/run-capture.sh
```

Plays the 440 Hz reference tone and checks the recording that comes back: same
amplitude, same frequency, no harmonics. A resampler that aliases, a downmix
that scales wrongly, or a tap that drops buffers all show up here immediately.

## Recognition accuracy

```bash
tests/run-asr.sh                      # every clip, three runs each
tests/run-asr.sh --runs 5             # more runs, tighter median
tests/run-asr.sh news-bulletin        # one clip
tests/run-asr.sh --model small        # a different model
```

Each clip is played, transcribed, and scored against its reference with word
error rate. Thresholds live in `fixtures/manifest.json`.

**Recognition is not deterministic.** The same clip scores anywhere from 5% to
35% on repeated runs - the pipeline decides when to run the model based on
arrival timing, so no two runs see the same windows. Each clip is therefore
measured several times and judged on the median, and thresholds are set well
above the median rather than at it. A single bad run means nothing; a moved
median means something.

Because of that spread, these tests catch regressions, not small improvements.
Anything under a few points of difference is noise, and needs more runs before
it means anything.

## The whole pipeline

```bash
tests/run-pipeline.sh                          # every clip, one run each
tests/run-pipeline.sh --runs 2 news-bulletin   # one clip, twice
tests/run-pipeline.sh --label after --out /tmp/after
```

Same clips, same path, but translation runs too and one run scores everything at
once: recognition against the English reference, translation against the Russian
one, and the latency of both. Comparing a change means two directories and

```bash
python3 tests/report.py /tmp/before /tmp/after
```

Translation is scored with **chrF** - character n-gram F-score. Word-level
metrics are unusable here: Russian marks case with endings, so a correct
translation that picks a different construction scores as wrong. chrF against a
single reference still punishes an honest paraphrase, so treat it the way WER is
treated - a moved median, not a verdict.

Four of the timing columns need no reference at all, and are the ones to trust:

| Column | What it is |
|---|---|
| `1st sub` | When the first translated word reached the screen, from the start of the clip |
| `refresh` | How long the reader looks at one subtitle before the next arrives |
| `wait` | From a line being finished to the first word of its translation |
| `tail` | How far past the end of the audio the last translation landed |

The two `e2e` columns estimate when each line was *spoken*, by matching its words
forward through the reference and converting the position to a time - the
fixtures are synthesised at a constant rate, so that holds. It breaks where
whisper writes "11:40" for "eleven forty", so read those columns as approximate.

An earlier version of that estimate measured position in the hypothesis instead
of the reference. It flattered exactly the runs it should have punished: a run
that dropped a third of the words spreads the rest across the whole clip and
scores as though it were fast.

### Nothing else may be on the GPU

Both models run on it, and a second copy of the app halves the machine. A run
made against one measured recognition at 2000 ms a pass where it is really 670,
and every conclusion drawn from it was wrong.

```bash
pkill -f "Marswind.app/Contents/MacOS/marswind"
```

The tell is `slowest_inference_ms` in the run's `SELFTEST result` line. On this
hardware with `large-v3-turbo-q5_0` it is 650-850 ms and barely moves. If it is
above a second, something is sharing the GPU and the run should be thrown away -
no change to caption length or prompt size moves that number.

## The fixtures

| Clip | What it is for |
|---|---|
| `news-bulletin` | Clear read at broadcast pace - the baseline |
| `named-entities` | Proper nouns, numbers and dates |
| `fast-conversational` | Fast speech, long sentences, few pauses |
| `two-speakers` | Voice changes every sentence |
| `tone-440` | Capture fidelity, not recognition |

The speech is synthesised with the macOS speech synthesiser, so the clips are
ours to redistribute and the reference transcripts are exact - there is no
question about what was said. Each clip also has a `<name>.ru.txt`, a human
Russian translation, which is what the translation score is measured against.

The cost of that choice: **synthetic speech is easier than the real thing.**
These numbers are better than what a user gets on a live broadcast with music
beds, crosstalk and compression. Treat them as a regression signal, not as a
quality claim.

To rebuild or extend the corpus, add an entry to `fixtures/manifest.json` with a
matching `<name>.txt` and run:

```bash
tests/generate-fixtures.sh
```

The generated `.wav` files are committed so the tests run without needing the
macOS voices installed.
