# Models

Marswind ships no models. It ships a catalog - ten entries in
[`src-tauri/src/models/catalog.rs`](../src-tauri/src/models/catalog.rs), each
with a URL, a size and a SHA-256 - and downloads the ones you ask for on first
run. That keeps the disk image at about 13 MB instead of several gigabytes, and
it means the weights on your disk came from the people who published them rather
than from a copy someone repackaged.

Two rules decide what gets a row.

**Everything offered is open source.** MIT or Apache-2.0, nothing else.
Marswind is MIT, and a catalog is not the place to hand somebody terms they did
not go looking for.

**Nothing is English-only.** whisper's `.en` builds are smaller and more
accurate at their size, and they are still not offered: this is an app for
watching things in languages you do not speak, and a recognizer that only hears
English cannot do that.

Neither rule is a limit of the app. Both are enforced by tests in the catalog,
so a row that breaks one fails the build rather than reaching a user.

## What you need

Two models to start, three files on disk:

| | | |
|---|---|---|
| **Recognition** | one of six whisper models | 78 MB - 1.6 GB |
| **Translation** | one of three Qwen3 models | 1.3 - 5.0 GB |
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

## The catalog

### Recognition

| Model | Size | Where it fits |
|---|---|---|
| `tiny` | 78 MB | Old hardware, or a quick smoke test |
| `base` | 148 MB | Compromise on machines with little memory |
| `small` | 488 MB | The practical quality floor |
| `large-v3-turbo-q5_0` | 574 MB | Best quality per gigabyte; the default above 16 GB |
| `medium` | 1.5 GB | Strong, and heavier than Turbo for similar results |
| `large-v3-turbo` | 1.6 GB | Highest accuracy, wants 8 GB free |

All six understand the 99 languages whisper was trained on, and work out which
one is being spoken unless you tell them.

### Translation

| Model | Size | Where it fits |
|---|---|---|
| `qwen3-1.7b-q4` | 1.3 GB | Below 16 GB. Measurably clumsier |
| `qwen3-4b-instruct-q4` | 2.5 GB | The default above 16 GB |
| `qwen3-8b-q4` | 5.0 GB | Steadier on long sentences, slower per line |

All three are Qwen3 in GGUF at Q4_K_M, and all three translate into any of the
thirteen target languages without a second download.

## Licenses

Every row in Settings names its license next to the Install button, and the name
is a link to the terms. This table is the same information in one place.

| Model | Published by | License |
|---|---|---|
| whisper `tiny` … `large-v3-turbo` | [ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) | [MIT](https://opensource.org/license/mit) |
| Silero VAD `v5.1.2` | [ggml-org/whisper-vad](https://huggingface.co/ggml-org/whisper-vad) | [MIT](https://opensource.org/license/mit) |
| Qwen3 1.7B | [ggml-org/Qwen3-1.7B-GGUF](https://huggingface.co/ggml-org/Qwen3-1.7B-GGUF) | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) |
| Qwen3 4B Instruct | [unsloth/Qwen3-4B-Instruct-2507-GGUF](https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF) | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) |
| Qwen3 8B | [unsloth/Qwen3-8B-GGUF](https://huggingface.co/unsloth/Qwen3-8B-GGUF) | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) |

Marswind never redistributes these weights, so its own MIT license would be
unaffected whatever the catalog held. Keeping it to MIT and Apache-2.0 is a
choice about what to put behind an Install button, not a legal necessity.

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

**A model the catalog no longer lists is not deleted from your disk.** Nothing
reads it any more, and the app will not offer to remove something it does not
list, so a file left over from an older version sits there until you delete it
by hand.

## Adding one

A new entry needs the fields in `ModelSpec`, and the three that cannot be
guessed are the size, the digest and the license. Hugging Face states the first
two on the file's page under **LFS**; if you would rather take them from the file
you already downloaded:

```bash
shasum -a 256 path/to/model.gguf
stat -f %z path/to/model.gguf
```

The worker lays every prompt out as ChatML, which is what Qwen and
Qwen-derived models expect. A model from a family that wants different turn
markers does not fail; it answers with the markers written out as text, which is
a confusing bug to chase from a transcript. Adding one means teaching
`translator/src/engine.rs` its format first.

The catalog's tests check that every entry is uniquely addressable, that its
digest is well formed, that its URL ends in its file name, that the license on
the row matches the repository it points at, that the license is one of the two
this project offers, and that no `.en` build has slipped in.
