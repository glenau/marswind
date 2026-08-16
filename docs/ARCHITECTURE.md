# Architecture

How Marswind is put together, and why it is put together that way. Where a
decision cost something to learn, the measurement that settled it is written
down beside it.

- [What this is](#what-this-is)
- [The shape of the thing](#the-shape-of-the-thing)
- [Threads and queues](#threads-and-queues)
- [Audio capture](#audio-capture)
- [Speech recognition](#speech-recognition)
- [Translation](#translation)
- [User interface](#user-interface)
- [Data on disk](#data-on-disk)
- [Headless self-test](#headless-self-test)
- [Build and distribution](#build-and-distribution)
- [Tried and reverted](#tried-and-reverted)

## What this is

A desktop app that captures the audio your computer is playing - a video in a
browser, a call, a local file - recognizes the speech in it, and shows a
translation beside the original as the speaker talks.

It makes **no network request you did not ask for**. There are exactly two that
it can make, and both start with a press: downloading a model, and the update
check in Settings. Nothing runs on a timer, nothing goes out at launch, and
between those presses the process is silent. Audio is held in memory,
recognized in memory, and never written to disk.

The frontend is SvelteKit + TypeScript inside [Tauri 2](https://tauri.app);
everything below it is Rust.

## The shape of the thing

Two binaries ship together:

| Binary | What it holds |
|---|---|
| `Marswind` | capture, VAD, whisper.cpp, the window, history, model downloads |
| `marswind-translator` | llama.cpp and nothing else, ~3.8 MB, bundled as a Tauri sidecar |

They are separate because they have to be - see
[why translation runs in its own process](#why-translation-runs-in-its-own-process).

```
┌─────────────────────────────────────────────────────────────────────────┐
│  System audio                                                           │
│  macOS: Core Audio process taps   │   Windows: WASAPI   Linux: PipeWire │
└────────────────────────────┬────────────────────────────────────────────┘
                             │ f32 PCM, device rate, N channels
                             ▼
                  ┌──────────────────────┐
                  │  Downmix → mono      │
                  │  Resample → 16 kHz   │
                  │  Ring buffer (30 s)  │
                  └──────────┬───────────┘
                             │
                             ▼
                  ┌──────────────────────┐
                  │  Silero VAD          │  phrase boundaries, drops silence
                  └──────────┬───────────┘
                             │ speech windows
                             ▼
                  ┌──────────────────────┐
                  │  whisper.cpp         │  Metal / CoreML on Apple Silicon
                  │  LocalAgreement-2    │  freezes words that stop moving
                  └──────────┬───────────┘
                             │ rows (to read) and segments (to translate)
                             ▼
                  ┌──────────────────────┐
                  │  marswind-translator │  llama.cpp, separate process
                  │  JSON lines, a pipe  │  streams its answer back
                  └──────────┬───────────┘
                             │
                             ▼
              ┌────────────────────────────────┐
              │  Window                        │
              │  original │ translation        │
              │  settings, models, history     │
              └────────────────────────────────┘
```

The frontend receives events over Tauri IPC (`audio://level`,
`asr://transcript`, `translate://line`, `translate://partial`,
`translate://skipped`, `models://progress`) and never touches audio
processing. Commands go the other
way, and each one has a typed wrapper in `src/lib/api.ts` so the UI cannot spell
one wrong.

## Threads and queues

| Thread | Job | Why it is its own |
|---|---|---|
| `capture` | OS audio callback | Real-time: any blocking is an audible dropout. Writes to a lock-free queue, allocates nothing |
| `pump` | downmix, resample, level, ring buffer | Cheap but continuous; must never wait on inference |
| `asr` | VAD + whisper.cpp | Holds the GPU or the Neural Engine for hundreds of milliseconds at a time |
| `mt` | talking to the translator process | Runs while ASR is already working on the next window |
| UI (main) | Tauri, the window, events | Never blocked by inference |

Threads talk over bounded channels. **On overflow, work is dropped and counted**
rather than queued: for live subtitles, losing a phrase beats drifting a minute
behind the audio. Dropped audio samples surface in the status bar; a dropped
translation segment is reported as `translate://skipped` so the interface can
tell "still coming" from "never arriving".

That last part was a bug before it was a feature. The translator dropped
segments silently, so a row whose translation had been discarded showed
"translating…" for the rest of the session. The queue now holds 64 segments
instead of 16, a drop is announced, and a finished row knows how many segments it
was cut into.

## Audio capture

### macOS - supported

**Core Audio process taps** (`AudioHardwareCreateProcessTap`,
`CATapDescription`), available since macOS 14.4. Three things it buys:

- it needs the **Audio Recording** permission rather than Screen Recording - no
  screen-recording indicator in the menu bar, and a far less alarming prompt;
- it can tap **one process** (the browser only, the call only) or the whole
  system output;
- **no virtual audio driver.** The user does not install BlackHole and rewire
  their output device.

`ScreenCaptureKit` remains possible behind the same trait as a fallback for
macOS 13, and is not implemented.

### Windows and Linux - in development

Windows will use **WASAPI loopback**, which captures whatever the default output
device is playing and needs no OS permission; per-process loopback
(`ActivateAudioInterfaceAsync` with `AUDIOCLIENT_ACTIVATION_PARAMS`, Windows 10
2004+) comes after that. Linux will use **PipeWire**.

Until then `audio::unsupported` stands in for both: the app builds and runs, and
capture reports itself unavailable. Everything above capture - resampling, VAD,
recognition, translation, the interface - is already platform-independent.

### The shared interface

```rust
trait AudioSource: Send {
    fn list_sources() -> Vec<SourceInfo>;   // devices and processes
    fn start(&mut self, sink: SampleSink) -> Result<()>;
    fn stop(&mut self);
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
}
```

Implementations are selected with `#[cfg(target_os)]`.

### Resampling

Devices hand over 44.1, 48 or 96 kHz; whisper wants 16 kHz mono. The **FFT**
resampler is used rather than a polynomial one: downsampling without an
anti-aliasing filter folds high frequencies back into the speech band, and that
noise costs recognition accuracy.

The ring buffer holds the most recent 30 seconds at the pipeline rate.
Overwriting the oldest samples is deliberate - stale audio is worthless to a
live captioner.

## Speech recognition

**whisper.cpp** through `whisper-rs`, chosen over faster-whisper because:

- no Python or CTranslate2 dependency - the app stays a bundle of two binaries;
- on Apple Silicon the encoder can run on the Neural Engine through CoreML and
  the decoder on Metal, where faster-whisper is CPU-bound on macOS;
- the same code runs on CPU, Vulkan or CUDA elsewhere without a build per
  backend;
- Silero VAD is built in, so voice activity detection needs no second runtime.

### Streaming a model that does not stream

Whisper works on 30-second windows. The naive approach - re-transcribe a growing
window and show the result - makes the text jump: words the reader has already
read get rewritten under them.

**LocalAgreement-2** is the fix. Transcribe a sliding window, and commit only the
prefix that two consecutive runs agree on. The unstable tail is rendered dimmed.
Subtitles grow instead of rewriting themselves.

Four rules around it look optional and are not. Each was found by watching it
fail:

- **Committed audio must leave the window.** LocalAgreement compares the tail of
  the last hypothesis against the head of the new one. Leave committed audio in
  place and the two stop overlapping, so the same words get committed twice.
- **Silence must never reach whisper.** Handed silence it invents text - "Thank
  you." is its favourite. The VAD runs first and a window without speech is
  discarded before the model sees it.
- **The last words of a hypothesis are never committed.** Whisper completes what
  it thinks it heard, so a window ending mid-phrase gets an invented ending - and
  the next window often invents the same one. Two runs agreeing is not evidence
  when both are guessing at the same cut-off.
- **Trim only on timestamps from the current window.** Words committed in earlier
  rounds carry timestamps of windows that no longer exist; trimming by those
  throws away audio nobody has read.

### Rows and segments

A caption used to be two things at once - the line the reader reads, and the unit
handed to the translator - and those want opposite sizes.

A good row is a sentence. A good translation unit is whatever is ready *now*,
because the translator cannot start until its unit is closed, and while it waits
the committed words sit on screen at full brightness with nothing beside them.
Words that LocalAgreement had already frozen, waiting on the end of a sentence
they were never going to change with.

So they are separate. Committed words go to the translator in **segments** as
soon as there are enough of them; the **row** they belong to keeps growing on
screen and closes later, at a boundary that makes it readable. A row's
translation is its segments joined in order, so the reader still sees one row of
source beside one row of target. Segments carry `line.seq` so the interface
appends them in place.

Both cut on the same kinds of boundary, at different sizes:

| | Full stop after | Comma after | Length valve |
|---|---|---|---|
| Segment - a translation unit | 4 words | 5 words | 8 words |
| Row - a line of the transcript | 4 words | 8 words | 16 words |

Segment size is a straight trade, measured in both directions on the same clip:

| Segment | WER | chrF | First subtitle | New subtitle every |
|---|---|---|---|---|
| 6 words | 22.1% | 57.3 | 4.9 s | 1.5 s |
| 8 words | 7.8% | 69.4 | 6.3 s | 2.3 s |
| a whole row (16) | 3.9% | 69.8 | 6.3 s | 3.1 s |

Six words puts text on screen fastest and translates visibly worse: there is not
enough of a clause in a piece that size to place the words, and a row ends with a
one-word fragment hanging off it. Sixteen - a segment per row, which is where
this started - costs four seconds before anything appears. **Eight is where
quality stops falling.**

The minimums keep "Mr." and a one-word answer from becoming units of their own.
On a forced close - a pause has ended the phrase and the rest is already in hand -
a row break that would leave fewer than four words behind is not taken, because
those words would go out as a row of their own, and a translator handed "month"
alone has nothing to work with.

Trailing punctuation is whisper's, so a clause boundary is only as good as its
comma; on live speech there are often none at all and the length valve is what
actually fires. That is the trade, and the prompt tells the translator so.

### Re-read words

Trimming leaves a fraction of a second of already-captioned audio at the head of
the window so a word on the boundary is not cut in half. Whisper reads it again,
and those words have to come off before anything else looks at the hypothesis -
a re-read word at the front pushes LocalAgreement out of step and commits the
same word twice. Captions read "Good Good evening" and "storm A storm system".

The test is text **and** time: the words must match what was recently committed
*and* end inside the audio that was already captioned. Neither works alone. Text
on its own edits a speaker who genuinely repeats themselves. Time on its own was
tried first and measurably cost real words - whisper's word timestamps move by
roughly the width of a short word between windows, so cutting on them turned "by
a failure" into "by failure".

A related one: when whisper loops a phrase, the hypothesis is discarded - and the
whole window used to go with it, including seconds after the loop that had never
been captioned. A sentence would simply vanish. Only the captioned part is
dropped now.

### Recognition models

Six, downloaded from Hugging Face inside the app with progress reporting and
SHA-256 verification against `src-tauri/src/models/catalog.rs`.

| Model | Size | Where it fits |
|---|---|---|
| `tiny` | ~78 MB | Old hardware, or a smoke test |
| `base` | ~148 MB | Compromise on machines with little memory |
| `small` | ~488 MB | Practical quality floor |
| `large-v3-turbo-q5_0` | ~574 MB | Best quality per gigabyte; the default above 16 GB |
| `medium` | ~1.5 GB | Strong, and heavier than Turbo for similar results |
| `large-v3-turbo` | ~1.6 GB | Highest accuracy, wants 8 GB free |

All six are multilingual. whisper also publishes `.en` builds, smaller and more
accurate at their size, and 0.1.1 dropped the two that were offered: this app
exists to caption speech in a language the reader does not have, and a
recognizer that only hears English cannot take part in that.

Sizes and checksums come from the Hugging Face LFS metadata of the source
repositories, so an interrupted download cannot be mistaken for a good one. A
default is suggested from installed memory, erring small on purpose: a model
that runs behind real time makes the app useless, where a smaller one merely
makes it less accurate.

Each entry also carries the license its weights are published under, shown on
the row beside the Install button. The app downloads models rather than shipping
them, so its own MIT license says nothing about them, and they do not all say
the same thing - [MODELS.md](MODELS.md) has the table and the one entry that
needs reading before it is used.

## Translation

Whisper can only translate **into English** (`task=translate`), so every other
target language needs a separate engine. Engines sit behind one enum and one
config, so a lighter one can be added without the rest of the pipeline noticing.

### The engine: a small instruct model

Qwen3 1.7B/4B/8B in GGUF through llama.cpp, all Q4_K_M. This is heavier than a
dedicated MT model and buys two things worth the weight:

- **any language pair** without downloading another model;
- **context** - previous captions go into the prompt as conversation turns, so a
  sentence that continues the one before it is translated as a continuation
  rather than in isolation.

Because captions are clauses rather than sentences, the prompt says so: each
message is the next piece of one continuous speech, a fragment is to stay a
fragment, and a sentence the speaker has not finished is not to be finished for
them. Without that the model completes the thought it thinks it heard, and the
next caption then translates the same words again.

**The prompt is ChatML**, which is what Qwen expects, and the worker writes no
other layout. A model from a family with different turn markers does not fail on
the wrong one - it answers with the markers written out as text and no idea
where its turn ends - so a new family means teaching the worker its format
rather than adding a catalog entry and hoping.

### Why translation runs in its own process

whisper.cpp and llama.cpp each bundle their own copy of ggml. Linking both into
one executable **appears to work**: it compiles, it links without a warning, and
the binary ends up with a single set of ggml symbols. It is broken anyway - one
library gets the other's ggml, and the mismatch is silent.

Measured on the same commit, changing nothing but the dependency:

| | Recognition output | Time per pass |
|---|---|---|
| llama.cpp linked in | `!!!!!!!!!!!!!!!!!!` | 1045-1410 ms |
| llama.cpp removed | "Good evening, Federal investigators say…" | 671-686 ms |

So the translation model runs in `marswind-translator`, a separate binary bundled
beside the app, speaking one JSON object per line over a pipe. Being forced apart
turns out to be an improvement: a translator that runs out of memory or hangs
mid-sentence takes nothing else down with it.

**Anything else built on ggml has to be treated the same way.**

### Translations arrive a word at a time

A sentence takes one to four seconds to translate, and waiting for the last word
before showing the first is most of the delay the reader feels. The worker writes
its answer out as it generates it - `{"id":N,"delta":"…"}` lines between the
request and its `Response`. The `Response` is unchanged and stays authoritative:
the pieces put words on screen early, they are not what the transcript or the
history is built from.

Streaming a language model's output is not a matter of forwarding every token:

- **A marker arrives one token at a time.** `<|im_end|>` and `<think>` are
  stripped from the finished text, but mid-generation `<|im` is indistinguishable
  from text that has not finished arriving. Any tail that could still grow into a
  marker is held back rather than shown and then taken away.
- **Text is released a whole word at a time.** A half-generated word reads as a
  typo, so a piece is only handed over once whitespace follows it.
- **A character can be split across two tokens.** Bytes are buffered and only the
  part that is complete UTF-8 is decoded; decoding each token on its own turns
  Cyrillic into replacement characters.
- **What is shown must be a prefix of what is returned.** Everything above
  preserves that, which is what lets the UI append instead of redraw - words a
  reader has already read never change under them.

The app forwards the pieces as `translate://partial` carrying the whole text so
far rather than the increment, so a dropped or reordered event costs nothing.

Measured from the moment recognition closes a caption: the first translated word
lands after 0.55-0.9 s where the finished sentence takes 1.2-3.6 s.

### The prompt is not read twice

Every request carries the same instruction and the same conversation plus one new
turn. Re-reading all of it is most of the cost of a translation, and it is paid on
the GPU that recognition is waiting for - with captions this short it is the
difference between the two models coexisting and the translator starving the
recognizer.

The worker keeps the prompt tokens it last decoded, compares them against the new
prompt, and decodes only the tail that differs; everything from the first
difference onward is dropped from the KV cache. Two details make it work:

- **At least one token must always be decoded.** The logits that start the answer
  come out of decoding, so a prompt identical to the last one still has to run its
  final token through the model.
- **The conversation grows; it does not slide.** Keeping "the last three captions"
  is the natural way to bound context and it silently destroys the cache - drop
  the oldest turn and every token after the instruction shifts, so nothing can be
  reused. Captions accumulate until they pass a character budget and are then cut
  back hard, which makes a full re-read rare instead of constant and gives the
  model more context in the meantime.

Across the fixture corpus this took the first subtitle from 8.5 s to 6.3 s, the
refresh interval from 4.0 s to 3.1 s, and chrF from 63.5 to 69.8 - while
recognition held at 660-700 ms a pass, which is the number that says the
translator is no longer taking the GPU away from it.

### Translation models

| Model | Size | Prompt family | Where it fits |
|---|---|---|---|
| `qwen3-1.7b-q4` | ~1.3 GB | ChatML | Below 16 GB. Measurably clumsier |
| `qwen3-4b-instruct-q4` | ~2.5 GB | ChatML | The default above 16 GB |
| `qwen3-8b-q4` | ~5.0 GB | ChatML | Steadier on long sentences, slower per line |

Thirteen target languages, listed in `src-tauri/src/translate/language.rs`. The
**English** name of the language is what goes into the prompt - instruction-tuned
models respond to "Russian" far more reliably than to "ru".

Translation quality is now limited by recognition rather than by the translator:
the remaining bad output traces back to a misheard source word nearly every time.

### A lighter engine, considered

A dedicated MT model - Opus-MT or NLLB-200-distilled in ONNX, 100-300 MB, ~100 ms
a phrase - was designed for and not built. Splitting translation into its own
process changed the arithmetic: the "light engine" it was meant to provide
already exists as a smaller GGUF model in the same worker, and a second runtime
means hand-writing a tokenizer and a decoding loop for a translator that is worse
and loses context between captions. It comes back if machines under 16 GB turn
out to need it.

## User interface

The main window is a fixed shell: the native title bar, a toolbar, and the
transcript taking everything left. Resizing gives the reader more subtitles
rather than more empty space.

```
┌─────────────────────────────────────────┐
│ ● ● ●   Marswind          (native bar)  │  draggable
├─────────────────────────────────────────┤
│ ◍  Waiting…    [Start] [History] [⚙]    │  level, state, one button, two views
├─────────────────────────────────────────┤
│  ORIGINAL              │  TRANSLATED    │  header inside the scroller
│  Officials confirmed…  │  Los func…     │
│                        │                │  fills the window
├─────────────────────────────────────────┤
│ ○ Audio ○ Recognition ○ Translation     │  status, timings, Clear
└─────────────────────────────────────────┘
```

Two rules that came out of building it:

- **The title bar stays.** Hiding it (`titleBarStyle: Overlay`) buys about thirty
  pixels and costs the ability to move the window, because nothing in the HTML is
  a drag region.
- **One control height for everything.** A single `--control` variable drives
  buttons, dropdowns and status pills, and paired dropdowns sit on a `1fr 1fr`
  grid. Letting each control size itself to its content is what makes a settings
  panel look accidental.

While capture is running, settings are locked with the reason shown rather than
silently ignoring edits.

### The transcript

Two columns: the original on the left, its translation level with it on the
right. Each row is its own grid, so the two cells start at the same baseline
however long either runs - a reader checking where a translated phrase came from
finds it beside the phrase rather than by counting lines. Below about 46 rem the
pair stacks with the translation first.

The rule between the columns is painted on the sheet rather than on the rows, so
it runs the full height of the window whether there is anything in it or not.
Drawn per row it appeared and vanished as the transcript filled, which made the
table look like it was assembling itself.

Nothing is width-capped. A fixed column looks tidier on a wide screen and is the
wrong call here: widening the window is a request for longer lines, not for more
margin.

**The column header lives inside the scroller**, stuck to its top. Outside it,
the header kept the full width while the rows lost the scrollbar's worth, so the
two halves stopped lining up the moment the transcript filled - and the rule,
drawn at the middle of the whole area, no longer sat at the middle of the rows.

### Scrollbars take no width

Two rules, and both are needed.

**The scrollbar is not styled.** Any `::-webkit-scrollbar` rule opts the page into
the classic scrollbar, which takes its width out of the content. Unstyled, macOS
uses its overlay scrollbar, which floats above the content and takes nothing.

Which means the page has to say which way round it is. `color-scheme` on the root
is what the engine draws its own widgets from - scrollbars, dropdown menus, the
checkbox, focus rings. Without it the page is assumed light, and in the dark
theme the native scrollbar arrives as the dark thumb meant for a white
background, which on that surface is a black bar.

**Scrolling areas ask for `scrollbar-gutter: stable both-edges`.** It does nothing
while scrollbars are overlay, and covers the user who has set "Show scroll bars:
Always". `stable` on its own is a trap - it stops the layout jumping and leaves
the content permanently further from the right edge than the left.

Measured both ways with the settings panel overflowing:

| Scrollbar | Width taken | Left inset | Right inset |
|---|---|---|---|
| Overlay (default) | 0 | 16 px | 16 px |
| Classic ("Always") | 22 px | 27 px | 27 px |

### The level indicator

The one piece of the interface that moves on its own: a circle with a pool of
liquid in it whose surface rises with the input level, crossed by two sine waves
at wavelengths that do not divide into each other. A bar only ever said how wide
the window was, and a dot that changed size said nothing at all while it was
quiet.

It is a CSS gradient masked by a repeating sine (`src/lib/LevelOrb.svelte`) and
not a canvas, deliberately: recognition and translation are both competing for
the GPU and for a main thread that has to stay free to paint captions, so the
indicator gets a compositor animation and no per-frame JavaScript. The level
arrives on `audio://level` and is eased before use, because a linear level spends
most of its time in the bottom third and the surface has to move for quiet speech
as much as for loud. Under `prefers-reduced-motion` the waves hold still - the
level is the height of the fill, not the motion, so nothing is lost.

The application icon is the same idea drawn at rest, **generated rather than
drawn**: `scripts/make-icon.py` computes the macOS corner as a real superellipse
(an SVG `rx` gives a circular corner that meets the straight edge at a visible
crease) and the two waves as real sines. A shape that comes out of forty lines of
arithmetic can be argued with; a PNG cannot.

### One surface, one scale

Every size in the app is a `rem`, and the root font size is a single number
multiplied by the user's text-size setting. Changing that setting moves text,
control heights, padding and radii together - a font-size switch that leaves the
buttons the size they were is not a bigger interface, it is a broken one.

There is also one background colour. Toolbars, panels and dialogs each used to
have their own, which is what made the window read as a stack of unrelated boxes.
Separation is hairlines; the only raised surface is a control.

The tokens are spacing (six steps), type (four sizes), one control height and one
radius. Anything not on those scales is a mistake, and having them in one place
is what makes that visible.

### Themes

Every colour is a custom property defined twice: once on `:root` and once on
`:root[data-theme="light"]`. The theme is one attribute on one element, and no
component knows which theme it is being drawn in - a rule that reaches for a
literal colour is a rule that works in only one of them. The light surfaces run
the other way round from the dark ones: the page is the lightest thing on screen
and a raised panel is a shade darker, which keeps a hairline visible without
turning every card into a box.

The setting is applied by an inline script in `src/app.html` **before the first
paint**, not by the app once it is running: a window that opens dark and turns
light on every launch is the one thing a light theme must not do. That is the
only place a preference is read from two files, and it is worth it for the flash
it costs otherwise.

### Interface language

A dictionary of thirteen locales and a `t()`, not a library - the app ships
nothing it does not need. English is the source of truth and declares the keys in
`src/lib/locales/en.ts`; every other locale is a partial of it in a file of its
own, and a missing key falls back to English, so an untranslated string is
visibly untranslated rather than blank.

The list is deliberately the one the translator offers: someone reading Polish
subtitles should be able to have the window in Polish.

### Views, not dialogs

Settings and history fill the window rather than floating over it. There is
nothing to see behind them while they are open, so a modal bought a second set of
edges and a smaller area to put things in. Their toolbar buttons behave as tabs:
pressing the one already open goes back.

The transcript stays mounted underneath. It is what listens to the pipeline, and
unmounting it while the user reads the settings would throw away everything
captured in the meantime.

### What is remembered

Language, theme and text size live in `localStorage` rather than the settings
file, because the window has to be laid out before any backend call returns. The
pipeline settings - the source, both models, both languages, and whether
translation and the original column are on - are stored the same way, under one
key as one object: they are read before the window is drawn, and nothing on the
Rust side needs them until Start is pressed. Anything missing or of the wrong
type falls back to the default rather than being trusted, so a store written by
an older build costs a default and not a launch.

### Closing while listening

The pipeline is brought down in order before the process goes: translation, then
recognition, then capture, because each is fed by the one after it. The session
being recorded is written out on the way, which is what anyone closing the window
mid-session expects.

Left to the runtime this crashed, and the stack says why:

```
ggml_abort  ←  ggml_metal_device_free  ←  ~unique_ptr  ←  __cxa_finalize_ranges  ←  exit
```

`exit()` runs ggml's static destructors, which tear the Metal device down. If a
whisper context is still alive at that point its residency set is not empty, and
ggml aborts rather than leak. Managed state has no ordering the runtime could
infer, so the order is written down in `shut_down()`. `MARSWIND_SELFTEST=quit:<sec>`
is that scenario: without the handler it exits 134, with it 0.

Both `WindowEvent::CloseRequested` and `RunEvent::ExitRequested` route there,
because quitting from the menu, a logout or `app.exit()` never reach the window.

## Data on disk

```
macOS:   ~/Library/Application Support/com.marswind.app/
Windows: %APPDATA%\com.marswind.app\
    models/…                     downloaded whisper and translation models
    transcripts/<epoch>.json     one file per listening session
    transcripts/exports/         what the export buttons write
    recordings/                  only ever written by the self-test
```

Nothing is written outside this directory, and nothing leaves the machine.
**Audio is never written to disk** except by the explicit capture self-test.

A session file is named after the epoch second it started, so files sort by time
on their name alone and the list needs no index. It holds every row with its
translation, when it was said, how long recognition took on it, how long its
translation took, and how many of its segments were dropped - the numbers are the
point as much as the text is, because a transcript on its own cannot say whether a
change to the pipeline helped.

The recorder listens to the same `asr://` and `translate://` events the transcript
is drawn from, rather than being called from inside the pipeline. That keeps
recognition and translation ignorant of it, and it means what is written down is
exactly what was shown instead of a second version of the truth assembled from a
private path.

Sessions export as text, SRT or JSON.

## Headless self-test

`MARSWIND_SELFTEST` makes the app exercise itself on launch and quit, so the
pipeline can be checked against real audio without a human clicking through the
UI:

| Value | What it does |
|---|---|
| `list` | Print the available audio sources |
| `capture:<sec>` | Record system audio to a WAV and report its level |
| `download:<id>` | Install a model from the catalog |
| `asr:<sec>` | Capture and transcribe, printing every caption with timings |
| `pipeline:<sec>` | The whole chain including translation |
| `quit:<sec>` | Start everything and exit with it still running |

`MARSWIND_SELFTEST_PLAY=<file>` plays a reference recording from inside the run,
which is what makes the reported lag meaningful - the clock starts at a known
instant. The rest override what the run would otherwise pick for itself:

| Variable | Overrides | Default |
|---|---|---|
| `MARSWIND_SELFTEST_SOURCE` | the audio source | all system audio |
| `MARSWIND_SELFTEST_MODEL` | the recognition model | what the machine's memory recommends |
| `MARSWIND_SELFTEST_LANGUAGE` | the spoken language | `en`, rather than detecting it |
| `MARSWIND_SELFTEST_NO_PROMPT` | set to stop feeding captions back to whisper | on |
| `MARSWIND_SELFTEST_MT_MODEL` | the translation model | what the machine's memory recommends |
| `MARSWIND_SELFTEST_TARGET` | the language to translate into | `ru` |
| `MARSWIND_SELFTEST_OUT` | where `capture` writes its WAV | the app data directory |

A fixture corpus with a known English reference is the reason the language is
pinned rather than detected: letting the run decide adds a second source of
variance to a number that already moves twenty points on its own.

`pipeline` is what `tests/run-pipeline.sh` drives: one run carries enough to score
recognition, translation and latency together, which is the only way to see that a
change bought seconds off the clock at the cost of the words. The fixture corpus
and the scoring are documented in [tests/README.md](../tests/README.md).

**Nothing else may be on the GPU during a run.** Both models use it, and a second
copy of the app halves the machine. One run measured recognition at 2000 ms a pass
where it is really 670, and every conclusion drawn from it was wrong. Check what
else is running before believing a latency regression.

### Developing against the tap API

`npm run tauri dev` launches a bare executable with no `Info.plist` and no code
signature. Process taps do not work in that shape - the permission record is keyed
to a signing identity and its usage description, so capture fails before it
starts. `npm run dev:macos` builds a debug `.app`, ad-hoc signs it, and launches
that.

An ad-hoc signature changes on every build, so macOS treats each build as a new
application and re-asks for the Audio Recording permission. A Developer ID
certificate is what makes that stop.

## Build and distribution

- **Two binaries.** `npm run build:sidecar` builds the worker into
  `src-tauri/binaries/` with the target triple Tauri expects. Every other build
  script calls it first, because Tauri bundles the worker as a sidecar and the
  build fails without one. This catches people running `cargo` directly rather
  than through a script: on a fresh clone `cargo test` stops at
  `resource path 'binaries/marswind-translator-…' doesn't exist`, because the
  sidecar is declared in `tauri.conf.json` and `tauri-build` checks for it
  before anything in `src-tauri` compiles.
- **The icon is generated.** `npm run build:icons` redraws it from
  `scripts/make-icon.py` and regenerates the whole `src-tauri/icons/` set. Its
  results are committed, so a normal build never runs it.
- **Two install paths.** `npm run dev:macos` builds, signs and launches a *debug*
  bundle; `npm run install:macos` builds a *release* bundle and installs it into
  `/Applications`. Both ad-hoc sign, which is not cosmetic: process taps need a
  signing identity to hang the Audio Recording permission on.
- **`npm run build:dmg`** builds the release bundle, signs it and packs it with
  `hdiutil`. The order is the point: what is copied out of an image is the app, so
  the app is signed before the image exists. Tauri's own dmg bundler is not used -
  it drives Finder through AppleScript to lay the window out, which needs a
  desktop session and hangs without one.
- **There is no CI.** Every check in this repository is a local command. The
  reason is the build: whisper.cpp and llama.cpp are compiled from source, and the
  pipeline harness plays audio through the system output and needs models on disk.
  A hosted runner can do the cheap half and would have to be trusted for the rest.
- **Models are not shipped in the image** - they are downloaded on first run
  according to the user's choice, which keeps the image around 13 MB.
- **The notices are generated.** `npm run licenses` walks both Cargo lockfiles
  through `cargo metadata` and the npm tree through `npm ls`, and writes
  `THIRD-PARTY-NOTICES.md`. The file is bundled into the `.app` as a resource
  next to `LICENSE`, because almost everything under this app is MIT or
  Apache-2.0 and both ask for their notice to travel with the binary. An
  inventory typed out by hand stops being true at the first `cargo update`.
- **Not notarized.** The image opens on the machine that built it; on anyone
  else's it has to be let through Gatekeeper by hand.
  [RELEASING.md](RELEASING.md) is the checklist for cutting one.

### Updating

A button in Settings → About, and nothing else. It reads GitHub's
`releases/latest`, compares the tag against the running version, and if there is
a newer one downloads that architecture's `.dmg` into Downloads and shows it in
Finder. `src-tauri/src/update.rs`.

Three decisions inside that are worth stating.

**It is a press, not a timer.** This app promises that nothing goes out unasked,
and a check on launch would quietly make that untrue for a claim repeated in
three files. There is no interval, no setting, and nothing to switch off,
because nothing runs.

**It downloads but does not install.** `tauri-plugin-updater` would swap the
bundle and restart, and it wants a signing key and a manifest to do it. The
reward would be small: the app is ad-hoc signed, so every new build is a new
identity to macOS and the Audio Recording permission gets asked for again
whichever way the file arrives. What is left worth automating is the download
and the checksum, and those are what this does.

**A release without a checksum is refused.** GitHub publishes no digest for an
attachment, so `scripts/build-dmg.sh` writes a `.dmg.sha256` beside the image
and the release carries both. The download is hashed as it streams and thrown
away on a mismatch - the same path a model takes, for the same reason. Forget
to attach the digest and the check reports no update rather than installing
something it cannot vouch for.

## Tried and reverted

Each of these looked obviously right, was built, and was measured. They are here
so nobody spends an afternoon rediscovering them.

**A context window in front of the audio.** Trimming right up to the last
committed word leaves whisper decoding two-second snippets, and short windows are
where it is weakest. Keeping five seconds of already-captioned audio in front of
the window as context should have helped and cost nothing, since whisper pads
every window to thirty seconds anyway. It was clearly worse: **32.5% word error
rate against 10.4%** on the same clip, with every clip regressing past its
threshold.

**Linking llama.cpp into the app.** It compiles and links cleanly and silently
corrupts recognition. See
[why translation runs in its own process](#why-translation-runs-in-its-own-process).

**Sliding the conversation window.** Keeping the last three captions bounds
context the obvious way and destroys the prompt cache completely, because every
token after the instruction shifts. Growing and then cutting back hard is both
faster and gives the model more context.

**Six-word translation segments.** Faster to first subtitle and visibly worse
Russian - chrF 57.3 against 69.4, WER 22.1% against 7.8%. Eight is the floor.

**Stripping re-read words by timestamp alone.** Whisper's word times move by about
the width of a short word between windows, so it turned "by a failure" into "by
failure". The check needs text and time together.

**Hiding the title bar.** `titleBarStyle: Overlay` bought about thirty pixels of
height and cost the ability to move the window - no native bar, and no drag region
in the HTML, leaves nothing to grab.

**Styling the scrollbars.** A `::-webkit-scrollbar` rule opts the page into the
classic scrollbar and takes 22 px out of every panel's right-hand side.

**A separate subtitle overlay window.** A borderless strip pinned above everything,
showing one caption at a time. It worked, and it was the wrong shape: the
transcript now shows original and translation side by side, which is what a reader
checking a translation actually wants, and a strip that shows one line and forgets
it cannot do that. Two windows also meant two copies of the caption state and two
places for every UI decision to be made. The AppKit window-level work it took to
float above a full-screen app is in the git history if it comes back.

**GitHub Actions.** Workflows for the checks and for a tagged release existed for
a day. What the project actually needed was a `.dmg` it could hand to someone, and
that is `npm run build:dmg`. Nothing automated runs against this repository now -
no workflows and no dependency bot - because a build that compiles whisper.cpp and
llama.cpp from source cannot be checked by a bot that never runs it.
