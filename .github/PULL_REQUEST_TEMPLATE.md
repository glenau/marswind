<!-- Everything here is optional if it does not apply. Delete what you do not need. -->

## What this changes

<!-- One or two sentences. The diff says what; say why. -->

Closes #

## What I measured

<!--
Anything touching capture, recognition or translation: run tests/run-pipeline.sh
before and after and paste both. A single run on this corpus varies by twenty
points of word error rate, so one number on its own proves nothing - see
tests/README.md.

"Nothing, this does not touch the pipeline" is a perfectly good answer.
-->

| Clip | | WER | chrF | First subtitle | Refresh |
|---|---|---|---|---|---|
| | before | | | | |
| | after | | | | |

## Checks

- [ ] `npm run check`
- [ ] `npm run build:sidecar` (once - cargo will not build `src-tauri` without it)
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and the same for `translator`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] Ran it: `npm run dev:macos`

## Housekeeping

- [ ] New user-visible strings are declared in `src/lib/locales/en.ts`
- [ ] New Rust commands have a typed wrapper in `src/lib/api.ts`
- [ ] `npm run licenses`, if a dependency changed
- [ ] `docs/ARCHITECTURE.md` updated, if this changed a decision it describes
- [ ] README updated, if a user would notice
- [ ] Screenshots below, if anything is visible
