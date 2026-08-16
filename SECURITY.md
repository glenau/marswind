# Security Policy

## Supported versions

Marswind is pre-1.0. Only the latest release, and `main`, get fixes.

| Version | Supported |
|---|---|
| 0.1.x | ✅ |
| older | ❌ |

## Reporting a vulnerability

**Do not open a public issue.**

Use GitHub's private reporting -
[Security → Report a vulnerability](https://github.com/glenau/marswind/security/advisories/new)
- or email **glenaudev@gmail.com**.

Please include what you can of: what the flaw is, how to reproduce it, what an
attacker gets out of it, and which version and OS you saw it on.

You will get an acknowledgement within **72 hours** and a decision on whether it
is a vulnerability within **7 days**. This is a one-person project, so a fix may
take longer than that; you will be told where it stands rather than left
waiting. Credit in the release notes unless you would rather not have it.

## What is in scope

The parts of the attack surface this project actually owns:

- **Model downloads.** Models are fetched over HTTPS from Hugging Face and
  verified against a SHA-256 in `src-tauri/src/models/catalog.rs` before use.
  A path that lets a file through without that check, or lets a download escape
  the models directory, is in scope.
- **The sidecar protocol.** The app and `marswind-translator` exchange JSON
  lines over a pipe. Anything that makes the app act on data it should not
  trust, or that lets another process talk to the worker, is in scope.
- **Transcripts on disk.** Sessions are written to the app data directory as
  JSON. Anything that widens their permissions, or writes them somewhere else,
  is in scope.
- **The Tauri boundary.** The frontend is trusted, but the capability set in
  `src-tauri/capabilities/default.json` is meant to be the smallest one that
  works. A command reachable from the frontend that does more than its name
  says is in scope.
- **Audio.** Captured audio is held in memory and never written to disk except
  by the explicit self-test. Anything that persists or transmits it is in scope,
  and is the most serious thing you could find here.

## What is out of scope

- Vulnerabilities in whisper.cpp, llama.cpp, Tauri or the models themselves -
  report those upstream. Tell us anyway if Marswind's use of them makes an
  upstream flaw materially worse.
- Anything requiring an attacker who already has code execution as your user.
- The absence of Apple notarization. It is known and documented in the README.
- Missing hardening flags with no demonstrated impact.
- `npm audit` advisories against SvelteKit's server-side request handling - the
  `cookie` one, at the time of writing. The frontend is `adapter-static` with
  `ssr = false`: there is no server, no request, and no code path that reaches
  them. Say so if you find one that is actually reachable; that is a different
  report and a welcome one.

## What the app does with your data

Stated here because it is the answer to most security questions about it:

- Audio is captured, resampled and recognized **in memory**. It is not written
  to disk and not sent anywhere.
- The only network traffic the app makes is downloading models from Hugging
  Face, on your explicit request, and it makes none at all once they are
  installed.
- There is no telemetry, no analytics, no crash reporting and no account.
- Transcripts are written to your app data directory, and only there, so that
  the History view has something to show. Delete them from inside the app or
  from the folder.
