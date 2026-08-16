# Models

Marswind ships no models. It ships a catalog - thirteen entries in
[`src-tauri/src/models/catalog.rs`](../src-tauri/src/models/catalog.rs), each
with a URL, a size and a SHA-256 - and downloads the ones you ask for on first
run. That keeps the disk image at about 13 MB instead of several gigabytes, and
it means the weights on your disk came from the people who published them rather
than from a copy someone repackaged.

It also means **the licenses are theirs, not this project's.** Marswind is MIT;
what you download through it is not necessarily. One family in particular is
not open source at all.

## What you need

Two models to start, three files on disk:

| | | |
|---|---|---|
| **Recognition** | one of seven whisper models | 78 MB - 1.6 GB |
| **Translation** | one of five instruct models | 1.3 - 7.3 GB |
| **Voice activity** | Silero VAD, installed with either | 865 KB |

The VAD model is required and small enough not to be a decision. Recognition
works on its own if you only want subtitles in the spoken language; translation
needs both.

Defaults are chosen from installed memory, erring small on purpose - a model
that runs behind real time makes the app useless, where a smaller one merely
makes it less accurate.

| Memory | Recognition | Translation |
|---|---|---|
| 16 GB or more | `Large v3 Turbo (compressed)` | `Qwen3 4B Instruct` |
| 8 GB | `Small` | `Qwen3 1.7B` |
| less | `Base` | `Qwen3 1.7B` |

## Licenses

Every row in Settings names its license next to the Install button, and the name
is a link to the terms. This table is the same information in one place.

| Model | Published by | License |
|---|---|---|
| whisper `tiny.en` … `large-v3-turbo` | [ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) | [MIT](https://opensource.org/license/mit) |
| Silero VAD `v5.1.2` | [ggml-org/whisper-vad](https://huggingface.co/ggml-org/whisper-vad) | [MIT](https://opensource.org/license/mit) |
| Qwen3 1.7B | [ggml-org/Qwen3-1.7B-GGUF](https://huggingface.co/ggml-org/Qwen3-1.7B-GGUF) | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) |
| Qwen3 4B Instruct | [unsloth/Qwen3-4B-Instruct-2507-GGUF](https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF) | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) |
| Qwen3 8B | [unsloth/Qwen3-8B-GGUF](https://huggingface.co/unsloth/Qwen3-8B-GGUF) | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) |
| Gemma 3 4B | [unsloth/gemma-3-4b-it-GGUF](https://huggingface.co/unsloth/gemma-3-4b-it-GGUF) | [Gemma Terms of Use](https://ai.google.dev/gemma/terms) |
| Gemma 3 12B | [unsloth/gemma-3-12b-it-GGUF](https://huggingface.co/unsloth/gemma-3-12b-it-GGUF) | [Gemma Terms of Use](https://ai.google.dev/gemma/terms) |

### The Gemma models are not open source

They are free to download and good at European languages, which is why they are
offered. They are also the one thing in this app that comes with strings
attached, so it is worth being plain about what they are:

- The [Gemma Terms of Use](https://ai.google.dev/gemma/terms) are Google's own
  license, not an OSI-approved one. Redistributing the weights, or anything
  derived from them, means passing the same terms and the
  [Prohibited Use Policy](https://ai.google.dev/gemma/prohibited_use_policy)
  along with them.
- Those restrictions reach the **output**, which for Marswind means the
  translations on your screen. Nothing in the terms interferes with reading
  subtitles; if you intend to publish or build on what comes out, read them.
- Google may update the Prohibited Use Policy, and the terms bind you to the
  current version.

Pick a Qwen3 model instead if you would rather not think about any of this.
Apache-2.0 asks for attribution and nothing else, and on most language pairs the
difference is small.

Marswind itself never redistributes these weights, so its own MIT license is
unaffected either way. The obligations are between you and the publisher, which
is exactly why the app names them before the download starts rather than after.

## What a download does

1. Fetches the file over HTTPS from the URL in the catalog. Nothing is
   mirrored, proxied or rewritten.
2. Writes it to `<app data>/models/<name>.part`, hashing as it goes.
3. Compares the digest against the SHA-256 in the catalog. **A mismatch
   installs nothing** - the partial file is deleted and the error says so.
4. Renames the file into place.

Sizes and digests come from the Hugging Face LFS metadata of the source
repositories, which is what makes an interrupted download impossible to mistake
for a good one. A file that is present at exactly the published size counts as
installed; hashing gigabytes every time the list is drawn would be the slowest
thing the app does.

## Where they live

```
macOS:   ~/Library/Application Support/com.marswind.app/models/
Windows: %APPDATA%\com.marswind.app\models\
```

Remove them from Settings, or delete the files - the app notices either way.
Nothing else in that directory is a model: transcripts and exports are its
neighbours, and are described in
[ARCHITECTURE.md](ARCHITECTURE.md#data-on-disk).

## Adding one

A new entry needs the fields in `ModelSpec`, and the three that cannot be
guessed are the size, the digest and the license. Hugging Face states the first
two on the file's page under **LFS**; if you would rather take them from the file
you already downloaded:

```bash
shasum -a 256 path/to/model.gguf
stat -f %z path/to/model.gguf
```

A translation model also needs its `PromptFamily` - ChatML for Qwen and
Qwen-derived models, Gemma for Gemma. A model handed the wrong one does not
fail; it answers with the turn markers written out as text, which is a
confusing bug to chase from the transcript. There is no auto-detection because
guessing from a file name is how that bug gets shipped.

The catalog's tests check that every entry is uniquely addressable, that its
digest is well formed, that its URL ends in its file name, and that the license
on the row matches the repository it points at.
