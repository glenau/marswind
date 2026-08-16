<div align="center">

<img src="src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Live subtitles and translation for your computer's audio. Fully offline.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#platform-support)
[![Version](https://img.shields.io/badge/version-0.1.1-brightgreen.svg)](#)

**English** ·
[Русский](docs/readme/README.ru.md) ·
[Deutsch](docs/readme/README.de.md) ·
[Español](docs/readme/README.es.md) ·
[Français](docs/readme/README.fr.md) ·
[Italiano](docs/readme/README.it.md) ·
[Português](docs/readme/README.pt.md) ·
[Polski](docs/readme/README.pl.md) ·
[Türkçe](docs/readme/README.tr.md) ·
[Українська](docs/readme/README.uk.md) ·
[中文](docs/readme/README.zh.md) ·
[日本語](docs/readme/README.ja.md) ·
[한국어](docs/readme/README.ko.md)

<img src="docs/screenshot.png" alt="The Marswind window: the original transcript on the left, its Spanish translation on the right" width="900">

</div>

Marswind listens to whatever is playing on your machine - a YouTube video, a
Google Meet, Teams or Zoom call, a local video file - recognizes the speech, and
translates it into the language of your choice as the speaker talks.

No API keys, no accounts, no internet. Models are downloaded once and then run
locally; your audio stays in memory and is never written to disk or sent
anywhere.

## What it does

- **Captures system audio** with no virtual audio driver - everything the
  machine plays, or a single application such as your browser
- **Recognizes speech** with whisper.cpp on the GPU, captions growing as they
  are spoken instead of being rewritten under the reader
- **Translates as the speaker talks** - words go to the translator as soon as
  they are committed, not once the sentence is finished, and the translation
  streams in a word at a time
- **Manages models** from inside the app: six recognition models and three
  translation models, every one of them MIT or Apache-2.0, downloaded with
  progress and SHA-256 verification
- **Records every session** - browse past ones and export them as text,
  subtitles (`.srt`) or JSON with the timings behind them
- **Ships sample clips** so it can be tried without going to find a video
- **Speaks thirteen languages** - the same ones it translates into - in a light
  or dark theme, with a text size that scales the whole interface rather than
  only its text

### Languages

English, Russian, German, Spanish, French, Italian, Portuguese, Polish, Turkish,
Ukrainian, Chinese, Japanese and Korean, both as translation targets and as the
language of the window itself. Recognition works out the spoken language from
the audio by default, and covers everything whisper does.

## How it works

```
System audio  →  resample to 16 kHz mono  →  voice activity detection (Silero)
              →  speech recognition (whisper.cpp)
              →  translation (llama.cpp, in a separate process)
              →  transcript: original on the left, translation beside it
```

Everything below the interface runs in Rust on dedicated threads, and
translation runs in a separate binary because whisper.cpp and llama.cpp cannot
share one process. The design and the reasoning behind it are in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

Measured on Apple Silicon with the default models, on the synthetic fixture
corpus in [tests/](tests/README.md) - medians of three runs per clip: the first
subtitle about 6 seconds into a clip, a new one every 2-3 seconds after that,
and a word error rate between 4% on a clear read and 23% on a clip of proper
nouns and figures. Recognition is not deterministic and a single run varies by
around twenty points, so those are medians and not results; how the numbers are
produced is documented alongside the harness.

## Platform support

| Platform | State |
|---|---|
| **macOS 14.4+** | Supported - Core Audio process taps, Metal |
| **Windows** | In development - WASAPI loopback |
| **Linux** | In development - PipeWire |

The app builds and runs on Windows and Linux today, but audio capture reports
itself unavailable there, which makes it a window with nothing to listen to.
Everything above capture is platform-independent and already works.

A virtual audio driver such as BlackHole is **not** required on any platform:
capture goes through the native OS APIs.

## Requirements

| | |
|---|---|
| macOS | 14.4 or newer, Apple Silicon or Intel |
| Memory | 8 GB for recognition alone, 16 GB with translation |
| Disk | 0.1-6.5 GB for the models you choose |
| To build | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Install

### Download it

The [latest release](https://github.com/glenau/marswind/releases/latest) has a
`.dmg`. Open it, drag Marswind to Applications, done - about 13 MB, since the
models are downloaded later and only the ones you pick.

**macOS will refuse to open it on the first try.** The image is signed but not
notarized: there is no paid Developer ID certificate behind this project, and
Gatekeeper treats anything without one as unidentified. The way through:

1. Open it once and let it be blocked. Press **Done** - not "Move to Bin".
2. **System Settings → Privacy & Security**, scroll down to **Security**. There
   is a line saying Marswind was blocked, and an **Open Anyway** button beside
   it.
3. Press it, authenticate, and confirm once more.

macOS asks once and remembers. The button only appears after a blocked launch
and lasts about an hour; if it is not there, try opening the app again.

Right-clicking the app and choosing Open is the older shortcut for this and
still works on macOS 14. macOS 15 removed it, so the route through Settings is
the one that works everywhere.

### Or build it

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

That builds the translation worker, builds the release bundle, ad-hoc signs it
and copies it to `/Applications/Marswind.app`. The first build takes several
minutes - whisper.cpp and llama.cpp are compiled from source. Nothing else is
needed: no submodules to check out, no libraries to install by hand, and no
models to fetch before starting.

### First run

1. `open /Applications/Marswind.app`
2. macOS asks for the **Audio Recording** permission. Say yes - without it the
   app hears nothing. If it was refused, grant it again in System Settings →
   Privacy & Security → Audio Recording.
3. Open **Settings** and download one recognition model and one translation
   model. `Large v3 Turbo (compressed)` and `Qwen3 4B Instruct` are the defaults
   on a machine with 16 GB or more; `Small` and `Qwen3 1.7B` fit in 8 GB. Around
   3 GB of downloads, verified against a published checksum as they arrive.
   Each row names the license the weights come under - see
   [docs/MODELS.md](docs/MODELS.md).
4. Press **Start listening**, then play something with speech. There are four
   sample clips in Settings if you would rather not go and find a video.

Two things worth knowing about a copy you built yourself:

- **It is ad-hoc signed.** The signature is stable for a given build, so the
  audio permission persists - but rebuilding produces a new identity and macOS
  asks for the permission again. A Developer ID certificate is what removes
  this, and there is none yet.
- **Do not move the app while it is running.** To update it, rerun
  `npm run install:macos`; it replaces `/Applications/Marswind.app` in place.

### Updating

**Settings → About → Check for updates.** It asks GitHub whether there is a
newer release; if there is, it downloads that image into Downloads, checks it
against the checksum published beside it, and shows it in Finder. Installing it
is the same drag as the first time.

Nothing checks on its own. There is no timer and no check at launch, because the
app makes no network request you did not press a button for.

A copy you built yourself updates the way it was installed: `npm run
install:macos` again.

### Building a disk image

```bash
npm run build:dmg
```

Builds the release bundle, signs it and packs it into
`src-tauri/target/Marswind-<version>-<arch>.dmg` - the same image that is
attached to a release, and it carries the same Gatekeeper caveat as the one
above. [docs/RELEASING.md](docs/RELEASING.md) is the checklist around it.

## Development

`tauri dev` produces a bare executable with no `Info.plist` and no signature,
and Core Audio process taps refuse to work in that shape. Use this instead - it
builds a debug bundle, signs it and launches it:

```bash
npm run dev:macos
```

| Command | What it does |
|---|---|
| `npm run dev:macos` | build, sign and launch a debug bundle |
| `npm run install:macos` | build a release bundle and install it |
| `npm run check` | Svelte and TypeScript types |
| `npm run build:dmg` | a signed `.dmg` to hand to somebody else |
| `npm run build:sidecar` | the translation worker on its own |
| `npm run build:icons` | redraw the app icon from `scripts/make-icon.py` |
| `npm run build:social` | redraw the social preview card GitHub shows on a link |
| `npm run licenses` | regenerate `THIRD-PARTY-NOTICES.md` from the lockfiles |

There is no CI: whisper.cpp and llama.cpp are compiled from source and the
pipeline harness plays audio through the system output, so every check is a
local command. [CONTRIBUTING.md](CONTRIBUTING.md) lists them.

## Tests

Unit tests cover the pure logic; the scripts in [tests/](tests/README.md) play
audio through the system output and score what comes back out of the real
pipeline - recognition, translation and latency together.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

The first line is needed once, and then only after a `cargo clean`. Tauri
bundles the translation worker as a sidecar, so its build script refuses to
build `src-tauri` at all until the binary is there - on a fresh clone, `cargo
test` on its own stops at `resource path 'binaries/marswind-translator-…'
doesn't exist`. Every `npm run` build command does this step for you; `cargo`
run directly does not.

The pipeline scripts need a built and signed bundle, and models installed:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

A single run on the fixture corpus varies by around twenty points of word error
rate, so one number on its own means nothing. Compare medians across runs, and
read the transcripts and not only the scores.

## Privacy

- Audio is captured, resampled and recognized **in memory**. It is never written
  to disk and never sent anywhere.
- The only network traffic is what you press a button for: downloading a model,
  or checking for a new version. Nothing runs on a timer or at launch.
- No telemetry, no analytics, no crash reporting, no account.
- Transcripts are written to your app data directory and nowhere else, so the
  History view has something to show. Delete them from inside the app.

## Contributing

Bug reports, ideas and pull requests are welcome.
[CONTRIBUTING.md](CONTRIBUTING.md) covers the setup, the checks, the commit
convention and what review looks for. Please open an issue before starting
anything large - several obvious improvements have already been tried and
reverted with the measurements recorded.

- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md) - report vulnerabilities privately, not in an
  issue

## Built on

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | recognition, and the Silero VAD implementation with it |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | translation, in the sidecar |
| [ggml](https://github.com/ggml-org/ggml) | MIT | the tensor library and Metal backend under both |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | the Rust binding to whisper.cpp |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | the Rust binding to llama.cpp |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | the model that finds phrase boundaries |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | the window and the process boundary |
| [Svelte](https://svelte.dev) | MIT | the interface |
| [rubato](https://github.com/HEnquist/rubato) | MIT | the FFT resampler in front of whisper |

Everything in the dependency graph, with the license each package is published
under, is in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) - generated from
the lockfiles, and shipped inside the app alongside the license itself.

**Models are not covered by any of that.** They are downloaded from
[Hugging Face](https://huggingface.co) on your request and keep their
publishers' terms - and the catalog only offers models under one people can take
at face value: the whisper and Silero models are MIT, Qwen3 is Apache-2.0. Each
row in Settings names its license before the download starts.
[docs/MODELS.md](docs/MODELS.md) has the detail.

## License

MIT - see [LICENSE](LICENSE). That covers this repository; it does not cover the
models, and the notices above are not a substitute for the licenses they point
at.
