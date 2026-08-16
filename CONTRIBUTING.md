# Contributing to Marswind

Thanks for looking. This is a small project, so the process is short.

## Before you write code

**Open an issue first** for anything larger than a fix. Not for bureaucracy -
this app has measured numbers behind most of its design decisions, and quite a
few obvious improvements have already been tried and reverted.
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) records which ones and why; reading
the section about the thing you want to change will often save you an afternoon.

Small fixes - a typo, a crash, a wrong string - need no issue. Send the pull
request.

## Setting up

Requires [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20 or newer
and cmake (`brew install cmake`). The first build compiles whisper.cpp and
llama.cpp from source and takes several minutes.

```bash
git clone https://github.com/glenau/marswind.git && cd marswind && npm install
```

```bash
npm run dev:macos
```

`tauri dev` on its own produces a bare executable with no `Info.plist` and no
signature, and Core Audio process taps refuse to work in that shape.
`npm run dev:macos` builds a debug bundle, ad-hoc signs it and launches it,
which is why it exists.

## Checks

There is no CI. Everything below runs on your own machine, and running it is
your job before you open a pull request - nothing on GitHub will do it for you.

```bash
npm run check
```

Anything that asks cargo to *build* `src-tauri` needs the translation worker to
exist first. Tauri bundles it as a sidecar, and its build script stops with
`resource path 'binaries/marswind-translator-…' doesn't exist` when it is
missing - which on a fresh clone is every clippy and test run. Once is enough,
until you `cargo clean`:

```bash
npm run build:sidecar
```

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo fmt --manifest-path translator/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

The unit tests cover the pure logic. Anything that touches audio, recognition
or translation is covered by the scripts in [tests/](tests/README.md), which
play sound through the system output and check what comes back out of the real
pipeline. They need models installed and a quiet machine:

```bash
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

**If you change anything in the recognition or translation path, run
`tests/run-pipeline.sh` before and after and put both numbers in the pull
request.** A single run on this corpus varies by twenty points of word error
rate, so one number on its own says nothing - compare medians across runs, and
read the transcripts and not only the scores.

If you add, remove or upgrade a dependency, regenerate the notices:

```bash
npm run licenses
```

It rewrites `THIRD-PARTY-NOTICES.md` from the lockfiles and prints a warning for
any package that cannot be taken under a permissive license. That file ships
inside the `.app`, so a stale one is a distribution missing an attribution
somebody is owed.

## Commits

[Conventional Commits](https://www.conventionalcommits.org): `type: summary`,
in the imperative, no full stop.

```
feat: translate a row before recognition closes it
fix: stop the translating indicator on a dropped segment
docs: record the segment-size measurements
```

Types in use: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`,
`chore`.

The body is for **why**, not what - the diff already says what. If a change
came out of a measurement, put the measurement in the body.

## Pull requests

- One change per pull request. A formatting sweep mixed into a bug fix is two
  pull requests.
- Say what you measured, or say that you measured nothing. Both are fine; a
  claim with nothing behind it is not.
- Update [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) if you changed a decision
  it describes, and the README if a user would notice.
- Screenshots for anything visible.

## What review looks for

In roughly this order:

1. **Is it true?** Claims in comments and docs are checked against the code. A
   comment that describes behaviour the code no longer has is a defect.
2. **Does it hold up when things go wrong?** Audio arrives on a real-time
   thread, models take seconds to load, the translator is a separate process
   that can die. Every new path gets asked what happens when its neighbour
   fails.
3. **Does it fit the shapes already here?** One control height, one spacing
   scale, one type scale; strings go through `src/lib/i18n.ts` and are declared
   in `src/lib/locales/en.ts`; Rust commands get a typed wrapper in
   `src/lib/api.ts`.
4. **Comments explain the decision, not the syntax.** The convention in this
   codebase is that a comment says why the obvious approach was not taken. If
   nothing surprising happened, no comment is needed.
5. **No dead ends left behind.** Code that is commented out, a flag with one
   value, a branch that cannot be reached - delete it. Git remembers.

Review is not a gate you have to be perfect for. Push what you have and say
what you are unsure about.

## Architecture

[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) is the map: what each process is
for, why translation is a separate binary, and where the boundaries are. Worth
reading before a first change to the pipeline.

## License

By contributing you agree that your work is licensed under the
[MIT License](LICENSE), the same as the rest of the project.
