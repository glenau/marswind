# Releasing

Cutting a release is a `.dmg` and a GitHub release page. There is no CI to do
any of it - the build compiles whisper.cpp and llama.cpp from source and the
pipeline harness plays audio through the system output, so every step below runs
on a Mac somebody is sitting at.

## Version numbers

The version is written in four files, and they have to agree:

| File | What reads it |
|---|---|
| `src-tauri/tauri.conf.json` | the bundle, and everything downstream of it |
| `package.json` | npm, and `scripts/build-dmg.sh` for the image name |
| `src-tauri/Cargo.toml` | the crate |
| `translator/Cargo.toml` | the worker crate |

Two more places mention it in prose and should be looked at: the version badge
at the top of `README.md`, and the supported-versions table in `SECURITY.md`.
The About panel is not one of them - it reads the version out of the running
bundle, which is why there is one number on screen and not a fifth copy of it.

The build number beside it is the commit count, stamped into `Info.plist` by
`scripts/stamp-build.sh` on every bundled build. Nothing to bump.

```bash
grep -rn '"version"\|^version' package.json src-tauri/tauri.conf.json \
  src-tauri/Cargo.toml translator/Cargo.toml
```

## Before building

```bash
npm run licenses          # THIRD-PARTY-NOTICES.md, if a dependency moved
npm run check
npm run build:sidecar     # cargo will not build src-tauri without it
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo fmt --manifest-path translator/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

`npm run licenses` should leave the file unchanged unless a dependency actually
moved. It is generated from the lockfiles and reproducible across installs, so a
diff here means something in the graph changed and the notices shipped inside the
app were about to be wrong.

Then the pipeline, because unit tests cannot tell you that the app still hears
anything:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-pipeline.sh --runs 3
```

Read the transcripts, not only the scores. A single run varies by around twenty
points of word error rate, and nothing else may be on the GPU while it runs -
[tests/README.md](../tests/README.md) explains both.

## Building the image

```bash
npm run build:dmg
```

Produces `src-tauri/target/Marswind-<version>-<arch>.dmg`, around 13 MB. The
script builds the worker, builds the release bundle, stamps the build number,
ad-hoc signs the app **before** packing it, and lays the image out with
`hdiutil`. The order is the point: what a user copies out of the image is the
app, so the app is what has to be signed.

`uname -m` decides the architecture in the filename. An Apple Silicon Mac builds
`arm64` and an Intel one builds `x86_64`, and neither can build the other here -
whisper.cpp and llama.cpp are compiled for the host. Two machines means two
images; one machine means saying which one it is on the release page.

## Checking the image

The build machine is the one computer where a signing problem cannot show
itself, so check it somewhere else - a second Mac, or at least a second account.

1. Mount the `.dmg`, drag Marswind to Applications.
2. Open it. **macOS should refuse**, because the image is not notarized. That is
   expected and documented; confirm the way through works - System Settings →
   Privacy & Security → **Open Anyway**, on the macOS version you are testing.
   Right-click → Open was the way through until macOS 15 removed it, so the
   instructions have to be checked against the newest macOS rather than the one
   you remember.
3. Say yes to the Audio Recording prompt.
4. Install a recognition model and a translation model from Settings.
5. Play a sample clip from Settings and watch subtitles and a translation
   appear.
6. Settings → About → **Third-party licenses** opens.
7. Quit while it is still listening. It should exit cleanly and the session
   should be in History afterwards.

Step 2 is the one people file issues about. Step 7 is the one that used to
crash.

## Publishing

```bash
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
gh release create v0.1.0 \
  src-tauri/target/Marswind-0.1.0-arm64.dmg \
  --title "Marswind 0.1.0" \
  --notes-file notes.md
```

The release notes should carry, every time:

- **what changed**, in the language a user would use;
- **the requirement**: macOS 14.4 or newer, Apple Silicon or Intel;
- **the Gatekeeper step**, spelled out rather than linked. Somebody who has just
  downloaded a `.dmg` and been told it is damaged does not go looking for a
  README;
- **which architecture** the image was built for;
- **that models are downloaded on first run**, around 3 GB, and that the app is
  a window with nothing to say until they are.

## Not notarized

Every release carries the Gatekeeper caveat because there is no paid Apple
Developer ID certificate behind this project. If one is ever added, the change
is `codesign` with the identity instead of `-`, then `xcrun notarytool submit`
and `xcrun stapler staple` in `scripts/build-dmg.sh` - and the reward is that
the Audio Recording permission survives a rebuild, since the permission record
is keyed to the signing identity. Ad-hoc signing gives every build a new one.
