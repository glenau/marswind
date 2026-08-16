# Third-party notices

Marswind is MIT licensed (see [LICENSE](LICENSE)). It is built out of other
people's work, most of it under licenses that ask for their copyright notice to
travel with the binary. This file is that notice, and it is shipped inside
`Marswind.app` alongside the license itself.

**It is generated.** Run `npm run licenses` after changing a dependency; do not
edit it by hand. The Rust half comes from `cargo metadata`, so it lists what the
lockfiles actually resolve to, on every platform rather than only the one you
are building on.

Models are **not** covered here. They are downloaded on your request, from
Hugging Face, and carry their own licenses - see
[docs/MODELS.md](docs/MODELS.md).

## Compiled into the binaries

C and C++ libraries vendored inside their `-sys` crates and compiled straight
into `Marswind` and `marswind-translator`. Cargo does not list them separately,
so they are named here.

| Library | Version | License | Used by | What for |
|---|---|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | 1.8.3 | MIT | Marswind | Speech recognition, and the Silero VAD implementation with it. |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | vendored by `llama-cpp-sys-2` 0.1.153 | MIT | marswind-translator | Runs the translation model. Built with the server, tools and examples off, so the libraries vendored for those - cpp-httplib, miniaudio, nlohmann/json, stb - are not compiled or linked. |
| [ggml](https://github.com/ggml-org/ggml) | vendored by both of the above | MIT | both | The tensor library under both of the above, including the Metal backend. Each ships its own copy, which is why the two cannot share a process - see docs/ARCHITECTURE.md. |

Copyright holders, as stated in each project's own LICENSE file:

- whisper.cpp - Copyright (c) 2023-2024 The ggml authors
- llama.cpp - Copyright (c) 2023-2024 The ggml authors
- ggml - Copyright (c) 2023-2024 The ggml authors

## Rust crates

553 crates across both binaries, from the `src-tauri/Cargo.lock` and
`translator/Cargo.lock` graphs. Where a crate offers a choice of licenses
(`MIT OR Apache-2.0` and friends), Marswind takes it under whichever is
compatible with MIT; the choice is not narrowed here because the text is the
crate's to state.

| Package | Version | License | Source |
|---|---|---|---|
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | [link](https://github.com/oyvindln/adler2) |
| `aho-corasick` | 1.1.4 | Unlicense OR MIT | [link](https://github.com/BurntSushi/aho-corasick) |
| `alloc-no-stdlib` | 2.0.4 | BSD-3-Clause | [link](https://github.com/dropbox/rust-alloc-no-stdlib) |
| `alloc-stdlib` | 0.2.4 | BSD-3-Clause | [link](https://github.com/dropbox/rust-alloc-no-stdlib) |
| `android_system_properties` | 0.1.5 | MIT/Apache-2.0 | [link](https://github.com/nical/android_system_properties) |
| `anstream` | 1.0.0 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/anstyle.git) |
| `anstyle` | 1.0.14 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/anstyle.git) |
| `anstyle-parse` | 1.0.0 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/anstyle.git) |
| `anstyle-query` | 1.1.5 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/anstyle.git) |
| `anstyle-wincon` | 3.0.11 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/anstyle.git) |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/anyhow) |
| `async-broadcast` | 0.7.2 | MIT OR Apache-2.0 | [link](https://github.com/smol-rs/async-broadcast) |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-channel) |
| `async-executor` | 1.14.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-executor) |
| `async-io` | 2.6.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-io) |
| `async-lock` | 3.4.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-lock) |
| `async-process` | 2.5.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-process) |
| `async-recursion` | 1.1.1 | MIT OR Apache-2.0 | [link](https://github.com/dcchut/async-recursion) |
| `async-signal` | 0.2.14 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-signal) |
| `async-task` | 4.7.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-task) |
| `async-trait` | 0.1.91 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/async-trait) |
| `atk` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `atk-sys` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/atomic-waker) |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | [link](https://github.com/cuviper/autocfg) |
| `aws-lc-rs` | 1.17.3 | ISC AND (Apache-2.0 OR ISC) | [link](https://github.com/aws/aws-lc-rs) |
| `aws-lc-sys` | 0.43.0 | ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) | [link](https://github.com/aws/aws-lc-rs) |
| `base64` | 0.21.7 | MIT OR Apache-2.0 | [link](https://github.com/marshallpierce/rust-base64) |
| `base64` | 0.22.1 | MIT OR Apache-2.0 | [link](https://github.com/marshallpierce/rust-base64) |
| `bindgen` | 0.72.1 | BSD-3-Clause | [link](https://github.com/rust-lang/rust-bindgen) |
| `bit-set` | 0.8.0 | Apache-2.0 OR MIT | [link](https://github.com/contain-rs/bit-set) |
| `bit-vec` | 0.8.0 | Apache-2.0 OR MIT | [link](https://github.com/contain-rs/bit-vec) |
| `bitflags` | 2.13.1 | MIT OR Apache-2.0 | [link](https://github.com/bitflags/bitflags) |
| `bitflags` | 1.3.2 | MIT/Apache-2.0 | [link](https://github.com/bitflags/bitflags) |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/utils) |
| `block-buffer` | 0.12.1 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/utils) |
| `block2` | 0.6.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `blocking` | 1.6.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/blocking) |
| `brotli` | 8.0.4 | BSD-3-Clause AND MIT | [link](https://github.com/dropbox/rust-brotli) |
| `brotli-decompressor` | 5.0.3 | BSD-3-Clause/MIT | [link](https://github.com/dropbox/rust-brotli-decompressor) |
| `bs58` | 0.5.1 | MIT/Apache-2.0 | [link](https://github.com/Nullus157/bs58-rs) |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 | [link](https://github.com/fitzgen/bumpalo) |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/Lokathor/bytemuck) |
| `byteorder` | 1.5.0 | Unlicense OR MIT | [link](https://github.com/BurntSushi/byteorder) |
| `bytes` | 1.12.1 | MIT | [link](https://github.com/tokio-rs/bytes) |
| `cairo-rs` | 0.18.5 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `cairo-sys-rs` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `camino` | 1.2.5 | MIT OR Apache-2.0 | [link](https://github.com/camino-rs/camino) |
| `cargo-platform` | 0.1.9 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/cargo) |
| `cargo_metadata` | 0.19.2 | MIT | [link](https://github.com/oli-obk/cargo_metadata) |
| `cargo_toml` | 0.22.3 | Apache-2.0 OR MIT | [link](https://gitlab.com/lib.rs/cargo_toml) |
| `cc` | 1.4.0 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/cc-rs) |
| `cesu8` | 1.1.0 | Apache-2.0/MIT | [link](https://github.com/emk/cesu8-rs) |
| `cexpr` | 0.6.0 | Apache-2.0/MIT | [link](https://github.com/jethrogb/rust-cexpr) |
| `cfb` | 0.7.3 | MIT | [link](https://github.com/mdsteele/rust-cfb) |
| `cfg-expr` | 0.15.8 | MIT OR Apache-2.0 | [link](https://github.com/EmbarkStudios/cfg-expr) |
| `cfg-if` | 1.0.4 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/cfg-if) |
| `cfg_aliases` | 0.2.2 | MIT | [link](https://github.com/katharostech/cfg_aliases) |
| `chacha20` | 0.10.1 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/stream-ciphers) |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 | [link](https://github.com/chronotope/chrono) |
| `clang-sys` | 1.9.1 | Apache-2.0 | [link](https://github.com/KyleMayes/clang-sys) |
| `cmake` | 0.1.58 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/cmake-rs) |
| `colorchoice` | 1.0.5 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/anstyle.git) |
| `combine` | 4.6.7 | MIT | [link](https://github.com/Marwes/combine) |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/concurrent-queue) |
| `const-oid` | 0.10.2 | Apache-2.0 OR MIT | [link](https://github.com/RustCrypto/formats) |
| `cookie` | 0.18.1 | MIT OR Apache-2.0 | [link](https://github.com/SergioBenitez/cookie-rs) |
| `core-foundation` | 0.10.1 | MIT OR Apache-2.0 | [link](https://github.com/servo/core-foundation-rs) |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | [link](https://github.com/servo/core-foundation-rs) |
| `core-graphics` | 0.25.0 | MIT OR Apache-2.0 | [link](https://github.com/servo/core-foundation-rs) |
| `core-graphics-types` | 0.2.0 | MIT OR Apache-2.0 | [link](https://github.com/servo/core-foundation-rs) |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/utils) |
| `cpufeatures` | 0.3.0 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/utils) |
| `crc32fast` | 1.5.0 | MIT OR Apache-2.0 | [link](https://github.com/srijs/rust-crc32fast) |
| `crossbeam-channel` | 0.5.16 | MIT OR Apache-2.0 | [link](https://github.com/crossbeam-rs/crossbeam) |
| `crossbeam-utils` | 0.8.22 | MIT OR Apache-2.0 | [link](https://github.com/crossbeam-rs/crossbeam) |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/traits) |
| `crypto-common` | 0.2.2 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/traits) |
| `cssparser` | 0.36.0 | MPL-2.0 | [link](https://github.com/servo/rust-cssparser) |
| `cssparser-macros` | 0.6.1 | MPL-2.0 | [link](https://github.com/servo/rust-cssparser) |
| `ctor` | 0.8.0 | Apache-2.0 OR MIT | [link](https://github.com/mmastrac/rust-ctor) |
| `ctor-proc-macro` | 0.0.7 | Apache-2.0 OR MIT | [link](https://github.com/mmastrac/rust-ctor) |
| `darling` | 0.23.0 | MIT | [link](https://github.com/TedDriggs/darling) |
| `darling_core` | 0.23.0 | MIT | [link](https://github.com/TedDriggs/darling) |
| `darling_macro` | 0.23.0 | MIT | [link](https://github.com/TedDriggs/darling) |
| `dbus` | 0.9.12 | Apache-2.0/MIT | [link](https://github.com/diwic/dbus-rs) |
| `defmt` | 1.1.1 | MIT OR Apache-2.0 | [link](https://github.com/knurling-rs/defmt) |
| `defmt-macros` | 1.1.1 | MIT OR Apache-2.0 | [link](https://github.com/knurling-rs/defmt) |
| `defmt-parser` | 1.0.0 | MIT OR Apache-2.0 | [link](https://github.com/knurling-rs/defmt) |
| `deranged` | 0.5.8 | MIT OR Apache-2.0 | [link](https://github.com/jhpratt/deranged) |
| `derive_more` | 2.1.1 | MIT | [link](https://github.com/JelteF/derive_more) |
| `derive_more-impl` | 2.1.1 | MIT | [link](https://github.com/JelteF/derive_more) |
| `digest` | 0.10.7 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/traits) |
| `digest` | 0.11.3 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/traits) |
| `dirs` | 6.0.0 | MIT OR Apache-2.0 | [link](https://github.com/soc/dirs-rs) |
| `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 | [link](https://github.com/dirs-dev/dirs-sys-rs) |
| `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | [link](https://github.com/yaahc/displaydoc) |
| `dlopen2` | 0.8.2 | MIT | [link](https://github.com/OpenByteDev/dlopen2) |
| `dlopen2_derive` | 0.4.3 | MIT | [link](https://github.com/OpenByteDev/dlopen2) |
| `dom_query` | 0.27.0 | MIT | [link](https://github.com/niklak/dom_query) |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | [link](https://github.com/rust-windowing/winit) |
| `dtoa` | 1.0.11 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/dtoa) |
| `dtoa-short` | 0.3.5 | MPL-2.0 | [link](https://github.com/upsuper/dtoa-short) |
| `dtor` | 0.3.0 | Apache-2.0 OR MIT | [link](https://github.com/mmastrac/rust-ctor) |
| `dtor-proc-macro` | 0.0.6 | Apache-2.0 OR MIT | [link](https://github.com/mmastrac/rust-ctor) |
| `dunce` | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | [link](https://gitlab.com/kornelski/dunce) |
| `dyn-clone` | 1.0.20 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/dyn-clone) |
| `either` | 1.17.0 | MIT OR Apache-2.0 | [link](https://github.com/rayon-rs/either) |
| `embed-resource` | 3.0.11 | MIT | [link](https://github.com/nabijaczleweli/rust-embed-resource) |
| `embed_plist` | 1.2.2 | MIT OR Apache-2.0 | [link](https://github.com/nvzqz/embed-plist-rs) |
| `encoding_rs` | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause | [link](https://github.com/hsivonen/encoding_rs) |
| `endi` | 1.1.1 | MIT | [link](https://github.com/zeenix/endi) |
| `enumflags2` | 0.7.12 | MIT OR Apache-2.0 | [link](https://github.com/meithecatte/enumflags2) |
| `enumflags2_derive` | 0.7.12 | MIT OR Apache-2.0 | [link](https://github.com/meithecatte/enumflags2) |
| `env_filter` | 2.0.0 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/env_logger) |
| `env_logger` | 0.11.11 | MIT OR Apache-2.0 | [link](https://github.com/rust-cli/env_logger) |
| `equivalent` | 1.0.2 | Apache-2.0 OR MIT | [link](https://github.com/indexmap-rs/equivalent) |
| `erased-serde` | 0.4.10 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/erased-serde) |
| `errno` | 0.3.14 | MIT OR Apache-2.0 | [link](https://github.com/lambda-fairy/rust-errno) |
| `event-listener` | 5.4.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/event-listener) |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/event-listener-strategy) |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/fastrand) |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | [link](https://github.com/image-rs/fdeflate) |
| `field-offset` | 0.3.6 | MIT OR Apache-2.0 | [link](https://github.com/Diggsey/rust-field-offset) |
| `find-msvc-tools` | 0.1.9 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/cc-rs) |
| `find_cuda_helper` | 0.2.0 | MIT OR Apache-2.0 | [link](https://github.com/Rust-GPU/Rust-CUDA) |
| `flate2` | 1.1.9 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/flate2-rs) |
| `fnv` | 1.0.7 | Apache-2.0 / MIT | [link](https://github.com/servo/rust-fnv) |
| `foldhash` | 0.2.0 | Zlib | [link](https://github.com/orlp/foldhash) |
| `foreign-types` | 0.5.0 | MIT/Apache-2.0 | [link](https://github.com/sfackler/foreign-types) |
| `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 | [link](https://github.com/sfackler/foreign-types) |
| `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 | [link](https://github.com/sfackler/foreign-types) |
| `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 | [link](https://github.com/servo/rust-url) |
| `fs_extra` | 1.3.0 | MIT | [link](https://github.com/webdesus/fs_extra) |
| `futures-channel` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-core` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-executor` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-io` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-lite` | 2.6.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/futures-lite) |
| `futures-macro` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-sink` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-task` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `futures-util` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/futures-rs) |
| `gdk` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `gdk-pixbuf` | 0.18.5 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `gdk-pixbuf-sys` | 0.18.0 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `gdk-sys` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `gdkwayland-sys` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `gdkx11` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `gdkx11-sys` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `generic-array` | 0.14.7 | MIT | [link](https://github.com/fizyk20/generic-array.git) |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | [link](https://github.com/rust-random/getrandom) |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | [link](https://github.com/rust-random/getrandom) |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 | [link](https://github.com/rust-random/getrandom) |
| `gio` | 0.18.4 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `gio-sys` | 0.18.1 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `glib` | 0.18.5 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `glib-macros` | 0.18.5 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `glib-sys` | 0.18.1 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `glob` | 0.3.4 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/glob) |
| `gobject-sys` | 0.18.0 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `gtk` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `gtk-sys` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `gtk3-macros` | 0.18.2 | MIT | [link](https://github.com/gtk-rs/gtk3-rs) |
| `h2` | 0.4.15 | MIT | [link](https://github.com/hyperium/h2) |
| `hashbrown` | 0.12.3 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/hashbrown) |
| `heck` | 0.4.1 | MIT OR Apache-2.0 | [link](https://github.com/withoutboats/heck) |
| `heck` | 0.5.0 | MIT OR Apache-2.0 | [link](https://github.com/withoutboats/heck) |
| `hermit-abi` | 0.5.2 | MIT OR Apache-2.0 | [link](https://github.com/hermit-os/hermit-rs) |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | [link](https://github.com/KokaKiwi/rust-hex) |
| `hound` | 3.5.1 | Apache-2.0 | [link](https://github.com/ruuda/hound) |
| `html5ever` | 0.38.0 | MIT OR Apache-2.0 | [link](https://github.com/servo/html5ever) |
| `http` | 1.5.0 | MIT OR Apache-2.0 | [link](https://github.com/hyperium/http) |
| `http-body` | 1.1.0 | MIT | [link](https://github.com/hyperium/http-body) |
| `http-body-util` | 0.1.4 | MIT | [link](https://github.com/hyperium/http-body) |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 | [link](https://github.com/seanmonstar/httparse) |
| `hybrid-array` | 0.4.13 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/hybrid-array) |
| `hyper` | 1.11.0 | MIT | [link](https://github.com/hyperium/hyper) |
| `hyper-rustls` | 0.27.9 | Apache-2.0 OR ISC OR MIT | [link](https://github.com/rustls/hyper-rustls) |
| `hyper-util` | 0.1.20 | MIT | [link](https://github.com/hyperium/hyper-util) |
| `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 | [link](https://github.com/strawlab/iana-time-zone) |
| `iana-time-zone-haiku` | 0.1.2 | MIT OR Apache-2.0 | [link](https://github.com/strawlab/iana-time-zone) |
| `ico` | 0.5.0 | MIT | [link](https://github.com/mdsteele/rust-ico) |
| `icu_collections` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `icu_locale_core` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `icu_normalizer` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `icu_normalizer_data` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `icu_properties` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `icu_properties_data` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `icu_provider` | 2.2.0 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `ident_case` | 1.0.1 | MIT/Apache-2.0 | [link](https://github.com/TedDriggs/ident_case) |
| `idna` | 1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/servo/rust-url/) |
| `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT | [link](https://github.com/hsivonen/idna_adapter) |
| `indexmap` | 1.9.3 | Apache-2.0 OR MIT | [link](https://github.com/bluss/indexmap) |
| `indexmap` | 2.14.0 | Apache-2.0 OR MIT | [link](https://github.com/indexmap-rs/indexmap) |
| `infer` | 0.19.0 | MIT | [link](https://github.com/bojand/infer) |
| `ipnet` | 2.12.0 | MIT OR Apache-2.0 | [link](https://github.com/krisprice/ipnet) |
| `is-docker` | 0.2.0 | MIT | [link](https://github.com/TheLarkInn/is-docker) |
| `is-wsl` | 0.4.0 | MIT | [link](https://github.com/TheLarkInn/is-wsl) |
| `is_terminal_polyfill` | 1.70.2 | MIT OR Apache-2.0 | [link](https://github.com/polyfill-rs/is_terminal_polyfill) |
| `itertools` | 0.13.0 | MIT OR Apache-2.0 | [link](https://github.com/rust-itertools/itertools) |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/itoa) |
| `javascriptcore-rs` | 1.1.2 | MIT | [link](https://github.com/tauri-apps/javascriptcore-rs) |
| `javascriptcore-rs-sys` | 1.1.1 | MIT | [link](https://github.com/tauri-apps/javascriptcore-rs) |
| `jiff` | 0.2.35 | Unlicense OR MIT | [link](https://github.com/BurntSushi/jiff) |
| `jiff-core` | 0.1.0 | Unlicense OR MIT | [link](https://github.com/BurntSushi/jiff) |
| `jiff-static` | 0.2.35 | Unlicense OR MIT | [link](https://github.com/BurntSushi/jiff) |
| `jni` | 0.21.1 | MIT/Apache-2.0 | [link](https://github.com/jni-rs/jni-rs) |
| `jni` | 0.22.4 | MIT OR Apache-2.0 | [link](https://github.com/jni-rs/jni-rs) |
| `jni-macros` | 0.22.4 | MIT OR Apache-2.0 | [link](https://github.com/jni-rs/jni-rs) |
| `jni-sys` | 0.3.1 | MIT OR Apache-2.0 | [link](https://github.com/jni-rs/jni-sys) |
| `jni-sys` | 0.4.1 | MIT OR Apache-2.0 | [link](https://github.com/jni-rs/jni-sys) |
| `jni-sys-macros` | 0.4.1 | MIT OR Apache-2.0 | [link](https://github.com/jni-rs/jni-sys) |
| `jobserver` | 0.1.35 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/jobserver-rs) |
| `js-sys` | 0.3.103 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| `json-patch` | 3.0.1 | MIT/Apache-2.0 | [link](https://github.com/idubrov/json-patch) |
| `jsonptr` | 0.6.3 | MIT OR Apache-2.0 | [link](https://github.com/chanced/jsonptr) |
| `keyboard-types` | 0.7.0 | MIT OR Apache-2.0 | [link](https://github.com/pyfisch/keyboard-types) |
| `libappindicator` | 0.9.0 | Apache-2.0 OR MIT | |
| `libappindicator-sys` | 0.9.0 | Apache-2.0 OR MIT | |
| `libc` | 0.2.189 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/libc) |
| `libdbus-sys` | 0.2.7 | Apache-2.0/MIT | [link](https://github.com/diwic/dbus-rs) |
| `libloading` | 0.8.9 | ISC | [link](https://github.com/nagisa/rust_libloading/) |
| `libloading` | 0.7.4 | ISC | [link](https://github.com/nagisa/rust_libloading/) |
| `libredox` | 0.1.18 | MIT | [link](https://gitlab.redox-os.org/redox-os/libredox.git) |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [link](https://github.com/sunfishcode/linux-raw-sys) |
| `litemap` | 0.8.2 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `llama-cpp-2` | 0.1.153 | MIT OR Apache-2.0 | [link](https://github.com/utilityai/llama-cpp-rs) |
| `llama-cpp-sys-2` | 0.1.153 | MIT OR Apache-2.0 | [link](https://github.com/utilityai/llama-cpp-rs) |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 | [link](https://github.com/Amanieu/parking_lot) |
| `log` | 0.4.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/log) |
| `lru-slab` | 0.1.2 | MIT OR Apache-2.0 OR Zlib | [link](https://github.com/Ralith/lru-slab) |
| `markup5ever` | 0.38.0 | MIT OR Apache-2.0 | [link](https://github.com/servo/html5ever) |
| `memchr` | 2.8.3 | Unlicense OR MIT | [link](https://github.com/BurntSushi/memchr) |
| `memoffset` | 0.9.1 | MIT | [link](https://github.com/Gilnaa/memoffset) |
| `mime` | 0.3.17 | MIT OR Apache-2.0 | [link](https://github.com/hyperium/mime) |
| `minimal-lexical` | 0.2.1 | MIT/Apache-2.0 | [link](https://github.com/Alexhuszagh/minimal-lexical) |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | [link](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `mio` | 1.2.2 | MIT | [link](https://github.com/tokio-rs/mio) |
| `muda` | 0.19.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/muda) |
| `ndk` | 0.9.0 | MIT OR Apache-2.0 | [link](https://github.com/rust-mobile/ndk) |
| `ndk-sys` | 0.6.0+11769913 | MIT OR Apache-2.0 | [link](https://github.com/rust-mobile/ndk) |
| `new_debug_unreachable` | 1.0.6 | MIT | [link](https://github.com/mbrubeck/rust-debug-unreachable) |
| `nom` | 7.1.3 | MIT | [link](https://github.com/Geal/nom) |
| `num-complex` | 0.4.6 | MIT OR Apache-2.0 | [link](https://github.com/rust-num/num-complex) |
| `num-conv` | 0.2.2 | MIT OR Apache-2.0 | [link](https://github.com/jhpratt/num-conv) |
| `num-integer` | 0.1.46 | MIT OR Apache-2.0 | [link](https://github.com/rust-num/num-integer) |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | [link](https://github.com/rust-num/num-traits) |
| `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | [link](https://github.com/illicitonion/num_enum) |
| `num_enum_derive` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | [link](https://github.com/illicitonion/num_enum) |
| `objc2` | 0.6.4 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-cloud-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-audio` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-audio-types` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-data` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-image` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-location` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-text` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-encode` | 4.1.0 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-exception-helper` | 0.1.1 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-foundation` | 0.3.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-quartz-core` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-ui-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-user-notifications` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-web-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/madsmtm/objc2) |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | [link](https://github.com/matklad/once_cell) |
| `once_cell_polyfill` | 1.70.2 | MIT OR Apache-2.0 | [link](https://github.com/polyfill-rs/once_cell_polyfill) |
| `open` | 5.4.0 | MIT | [link](https://github.com/Byron/open-rs) |
| `openssl-probe` | 0.2.1 | MIT OR Apache-2.0 | [link](https://github.com/rustls/openssl-probe) |
| `option-ext` | 0.2.0 | MPL-2.0 | [link](https://github.com/soc/option-ext.git) |
| `ordered-stream` | 0.2.0 | MIT OR Apache-2.0 | [link](https://github.com/danieldg/ordered-stream) |
| `pango` | 0.18.3 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `pango-sys` | 0.18.0 | MIT | [link](https://github.com/gtk-rs/gtk-rs-core) |
| `parking` | 2.2.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/parking) |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | [link](https://github.com/Amanieu/parking_lot) |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | [link](https://github.com/Amanieu/parking_lot) |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | [link](https://github.com/servo/rust-url/) |
| `phf` | 0.13.1 | MIT | [link](https://github.com/rust-phf/rust-phf) |
| `phf_codegen` | 0.13.1 | MIT | [link](https://github.com/rust-phf/rust-phf) |
| `phf_generator` | 0.13.1 | MIT | [link](https://github.com/rust-phf/rust-phf) |
| `phf_macros` | 0.13.1 | MIT | [link](https://github.com/rust-phf/rust-phf) |
| `phf_shared` | 0.13.1 | MIT | [link](https://github.com/rust-phf/rust-phf) |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/pin-project-lite) |
| `piper` | 0.2.5 | MIT OR Apache-2.0 | [link](https://github.com/smol-rs/piper) |
| `pkg-config` | 0.3.33 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/pkg-config-rs) |
| `plist` | 1.10.0 | MIT | [link](https://github.com/ebarnard/rust-plist/) |
| `png` | 0.17.16 | MIT OR Apache-2.0 | [link](https://github.com/image-rs/image-png) |
| `png` | 0.18.1 | MIT OR Apache-2.0 | [link](https://github.com/image-rs/image-png) |
| `polling` | 3.11.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/polling) |
| `portable-atomic` | 1.14.0 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/portable-atomic) |
| `portable-atomic-util` | 0.2.7 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/portable-atomic-util) |
| `potential_utf` | 0.1.5 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `powerfmt` | 0.2.0 | MIT OR Apache-2.0 | [link](https://github.com/jhpratt/powerfmt) |
| `precomputed-hash` | 0.1.1 | MIT | [link](https://github.com/emilio/precomputed-hash) |
| `prettyplease` | 0.2.37 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/prettyplease) |
| `primal-check` | 0.3.4 | MIT OR Apache-2.0 | [link](https://github.com/huonw/primal) |
| `proc-macro-crate` | 1.3.1 | MIT OR Apache-2.0 | [link](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-crate` | 2.0.2 | MIT OR Apache-2.0 | [link](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 | [link](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-error` | 1.0.4 | MIT OR Apache-2.0 | [link](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| `proc-macro-error-attr` | 1.0.4 | MIT OR Apache-2.0 | [link](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/proc-macro2) |
| `quick-xml` | 0.41.0 | MIT | [link](https://github.com/tafia/quick-xml) |
| `quinn` | 0.11.11 | MIT OR Apache-2.0 | [link](https://github.com/quinn-rs/quinn) |
| `quinn-proto` | 0.11.16 | MIT OR Apache-2.0 | [link](https://github.com/quinn-rs/quinn) |
| `quinn-udp` | 0.5.15 | MIT OR Apache-2.0 | [link](https://github.com/quinn-rs/quinn) |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/quote) |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [link](https://github.com/r-efi/r-efi) |
| `r-efi` | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [link](https://github.com/r-efi/r-efi) |
| `rand` | 0.10.2 | MIT OR Apache-2.0 | [link](https://github.com/rust-random/rand) |
| `rand_core` | 0.10.1 | MIT OR Apache-2.0 | [link](https://github.com/rust-random/rand_core) |
| `rand_pcg` | 0.10.2 | MIT OR Apache-2.0 | [link](https://github.com/rust-random/rngs) |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | [link](https://github.com/rust-windowing/raw-window-handle) |
| `realfft` | 3.5.0 | MIT | [link](https://github.com/HEnquist/realfft) |
| `redox_syscall` | 0.5.18 | MIT | [link](https://gitlab.redox-os.org/redox-os/syscall) |
| `redox_users` | 0.5.2 | MIT | [link](https://gitlab.redox-os.org/redox-os/users) |
| `ref-cast` | 1.0.26 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/ref-cast) |
| `ref-cast-impl` | 1.0.26 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/ref-cast) |
| `regex` | 1.13.1 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/regex) |
| `regex-automata` | 0.4.16 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/regex) |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/regex) |
| `reqwest` | 0.13.4 | MIT OR Apache-2.0 | [link](https://github.com/seanmonstar/reqwest) |
| `ring` | 0.17.14 | Apache-2.0 AND ISC | [link](https://github.com/briansmith/ring) |
| `rtrb` | 0.3.4 | MIT OR Apache-2.0 | [link](https://github.com/mgeier/rtrb) |
| `rubato` | 0.16.2 | MIT | [link](https://github.com/HEnquist/rubato) |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/rustc-hash) |
| `rustc_version` | 0.4.1 | MIT OR Apache-2.0 | [link](https://github.com/djc/rustc-version-rs) |
| `rustfft` | 6.4.1 | MIT OR Apache-2.0 | [link](https://github.com/ejmahler/RustFFT) |
| `rustix` | 1.1.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [link](https://github.com/bytecodealliance/rustix) |
| `rustls` | 0.23.43 | Apache-2.0 OR ISC OR MIT | [link](https://github.com/rustls/rustls) |
| `rustls-native-certs` | 0.8.4 | Apache-2.0 OR ISC OR MIT | [link](https://github.com/rustls/rustls-native-certs) |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | [link](https://github.com/rustls/pki-types) |
| `rustls-platform-verifier` | 0.7.0 | MIT OR Apache-2.0 | [link](https://github.com/rustls/rustls-platform-verifier) |
| `rustls-platform-verifier-android` | 0.1.1 | MIT OR Apache-2.0 | [link](https://github.com/rustls/rustls-platform-verifier) |
| `rustls-webpki` | 0.103.13 | ISC | [link](https://github.com/rustls/webpki) |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/rustversion) |
| `same-file` | 1.0.6 | Unlicense/MIT | [link](https://github.com/BurntSushi/same-file) |
| `schannel` | 0.1.29 | MIT | [link](https://github.com/steffengy/schannel-rs) |
| `schemars` | 0.8.22 | MIT | [link](https://github.com/GREsau/schemars) |
| `schemars` | 0.9.0 | MIT | [link](https://github.com/GREsau/schemars) |
| `schemars` | 1.2.2 | MIT | [link](https://github.com/GREsau/schemars) |
| `schemars_derive` | 0.8.22 | MIT | [link](https://github.com/GREsau/schemars) |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | [link](https://github.com/bluss/scopeguard) |
| `security-framework` | 3.7.0 | MIT OR Apache-2.0 | [link](https://github.com/kornelski/rust-security-framework) |
| `security-framework-sys` | 2.17.0 | MIT OR Apache-2.0 | [link](https://github.com/kornelski/rust-security-framework) |
| `selectors` | 0.36.1 | MPL-2.0 | [link](https://github.com/servo/stylo) |
| `self_cell` | 1.3.0 | Apache-2.0 OR GPL-2.0-only | [link](https://github.com/Voultapher/self_cell) |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/semver) |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | [link](https://github.com/serde-rs/serde) |
| `serde-untagged` | 0.1.9 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/serde-untagged) |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | [link](https://github.com/serde-rs/serde) |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | [link](https://github.com/serde-rs/serde) |
| `serde_derive_internals` | 0.29.1 | MIT OR Apache-2.0 | [link](https://github.com/serde-rs/serde) |
| `serde_json` | 1.0.151 | MIT OR Apache-2.0 | [link](https://github.com/serde-rs/json) |
| `serde_repr` | 0.1.21 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/serde-repr) |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `serde_spanned` | 1.1.1 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `serde_with` | 3.21.0 | MIT OR Apache-2.0 | [link](https://github.com/jonasbb/serde_with/) |
| `serde_with_macros` | 3.21.0 | MIT OR Apache-2.0 | [link](https://github.com/jonasbb/serde_with/) |
| `serialize-to-javascript` | 0.1.2 | MIT OR Apache-2.0 | [link](https://github.com/chippers/serialize-to-javascript) |
| `serialize-to-javascript-impl` | 0.1.2 | MIT OR Apache-2.0 | [link](https://github.com/chippers/serialize-to-javascript) |
| `servo_arc` | 0.4.3 | MIT OR Apache-2.0 | [link](https://github.com/servo/stylo) |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/hashes) |
| `sha2` | 0.11.0 | MIT OR Apache-2.0 | [link](https://github.com/RustCrypto/hashes) |
| `shlex` | 1.3.0 | MIT OR Apache-2.0 | [link](https://github.com/comex/rust-shlex) |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | [link](https://github.com/comex/rust-shlex) |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 | [link](https://github.com/vorner/signal-hook) |
| `simd-adler32` | 0.3.10 | MIT | [link](https://github.com/mcountryman/simd-adler32) |
| `simd_cesu8` | 1.2.0 | Apache-2.0 OR MIT | [link](https://github.com/seancroach/simd_cesu8) |
| `simdutf8` | 0.1.5 | MIT OR Apache-2.0 | [link](https://github.com/rusticstuff/simdutf8) |
| `siphasher` | 1.0.3 | MIT/Apache-2.0 | [link](https://github.com/jedisct1/rust-siphash) |
| `slab` | 0.4.12 | MIT | [link](https://github.com/tokio-rs/slab) |
| `smallvec` | 1.15.2 | MIT OR Apache-2.0 | [link](https://github.com/servo/rust-smallvec) |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 | [link](https://github.com/rust-lang/socket2) |
| `softbuffer` | 0.4.8 | MIT OR Apache-2.0 | [link](https://github.com/rust-windowing/softbuffer) |
| `soup3` | 0.5.0 | MIT | [link](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| `soup3-sys` | 0.5.0 | MIT | [link](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | [link](https://github.com/storyyeller/stable_deref_trait) |
| `strength_reduce` | 0.2.4 | MIT OR Apache-2.0 | [link](http://github.com/ejmahler/strength_reduce) |
| `string_cache` | 0.9.0 | MIT OR Apache-2.0 | [link](https://github.com/servo/string-cache) |
| `string_cache_codegen` | 0.6.1 | MIT OR Apache-2.0 | [link](https://github.com/servo/string-cache) |
| `strsim` | 0.11.1 | MIT | [link](https://github.com/rapidfuzz/strsim-rs) |
| `subtle` | 2.6.1 | BSD-3-Clause | [link](https://github.com/dalek-cryptography/subtle) |
| `swift-rs` | 1.0.7 | MIT OR Apache-2.0 | [link](https://github.com/Brendonovich/swift-rs) |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/syn) |
| `syn` | 3.0.3 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/syn) |
| `syn` | 1.0.109 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/syn) |
| `sync_wrapper` | 1.0.2 | Apache-2.0 | [link](https://github.com/Actyx/sync_wrapper) |
| `synstructure` | 0.13.2 | MIT | [link](https://github.com/mystor/synstructure) |
| `system-deps` | 6.2.2 | MIT OR Apache-2.0 | [link](https://github.com/gdesmott/system-deps) |
| `tao` | 0.35.3 | Apache-2.0 | [link](https://github.com/tauri-apps/tao) |
| `tao-macros` | 0.1.4 | MIT OR Apache-2.0 | [link](https://github.com/tauri-apps/tao) |
| `target-lexicon` | 0.12.16 | Apache-2.0 WITH LLVM-exception | [link](https://github.com/bytecodealliance/target-lexicon) |
| `tauri` | 2.11.5 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-build` | 2.6.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-codegen` | 2.6.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-macros` | 2.6.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-plugin` | 2.6.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-plugin-opener` | 2.5.4 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/plugins-workspace) |
| `tauri-runtime` | 2.11.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-runtime-wry` | 2.11.4 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-utils` | 2.9.3 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `tauri-winres` | 0.3.6 | MIT | [link](https://github.com/tauri-apps/winres) |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | [link](https://github.com/Stebalien/tempfile) |
| `tendril` | 0.5.1 | MIT OR Apache-2.0 | [link](https://github.com/servo/html5ever) |
| `thiserror` | 2.0.19 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/thiserror) |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 2.0.19 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/thiserror) |
| `time` | 0.3.54 | MIT OR Apache-2.0 | [link](https://github.com/time-rs/time) |
| `time-core` | 0.1.9 | MIT OR Apache-2.0 | [link](https://github.com/time-rs/time) |
| `time-macros` | 0.2.32 | MIT OR Apache-2.0 | [link](https://github.com/time-rs/time) |
| `tinystr` | 0.8.3 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `tinyvec` | 1.12.0 | Zlib OR Apache-2.0 OR MIT | [link](https://github.com/Lokathor/tinyvec) |
| `tinyvec_macros` | 0.1.1 | MIT OR Apache-2.0 OR Zlib | [link](https://github.com/Soveu/tinyvec_macros) |
| `tokio` | 1.53.1 | MIT | [link](https://github.com/tokio-rs/tokio) |
| `tokio-rustls` | 0.26.4 | MIT OR Apache-2.0 | [link](https://github.com/rustls/tokio-rustls) |
| `tokio-util` | 0.7.19 | MIT | [link](https://github.com/tokio-rs/tokio) |
| `toml` | 0.8.2 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml` | 0.9.12+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml` | 1.1.4+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_datetime` | 0.6.3 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_datetime` | 0.7.5+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_datetime` | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.19.15 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.20.2 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.25.13+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_parser` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `toml_writer` | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/toml-rs/toml) |
| `tower` | 0.5.3 | MIT | [link](https://github.com/tower-rs/tower) |
| `tower-http` | 0.6.11 | MIT | [link](https://github.com/tower-rs/tower-http) |
| `tower-layer` | 0.3.3 | MIT | [link](https://github.com/tower-rs/tower) |
| `tower-service` | 0.3.3 | MIT | [link](https://github.com/tower-rs/tower) |
| `tracing` | 0.1.44 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-attributes` | 0.1.31 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-core` | 0.1.36 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `transpose` | 0.2.3 | MIT OR Apache-2.0 | [link](https://github.com/ejmahler/transpose) |
| `tray-icon` | 0.24.2 | MIT OR Apache-2.0 | [link](https://github.com/tauri-apps/tray-icon) |
| `try-lock` | 0.2.5 | MIT | [link](https://github.com/seanmonstar/try-lock) |
| `typeid` | 1.0.3 | MIT OR Apache-2.0 | [link](https://github.com/dtolnay/typeid) |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | [link](https://github.com/paholg/typenum) |
| `uds_windows` | 1.2.1 | MIT | [link](https://github.com/haraldh/rust_uds_windows) |
| `unic-char-property` | 0.9.0 | MIT/Apache-2.0 | [link](https://github.com/open-i18n/rust-unic/) |
| `unic-char-range` | 0.9.0 | MIT/Apache-2.0 | [link](https://github.com/open-i18n/rust-unic/) |
| `unic-common` | 0.9.0 | MIT/Apache-2.0 | [link](https://github.com/open-i18n/rust-unic/) |
| `unic-ucd-ident` | 0.9.0 | MIT/Apache-2.0 | [link](https://github.com/open-i18n/rust-unic/) |
| `unic-ucd-version` | 0.9.0 | MIT/Apache-2.0 | [link](https://github.com/open-i18n/rust-unic/) |
| `unicode-ident` | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | [link](https://github.com/dtolnay/unicode-ident) |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 | [link](https://github.com/unicode-rs/unicode-segmentation) |
| `untrusted` | 0.9.0 | ISC | [link](https://github.com/briansmith/untrusted) |
| `url` | 2.5.8 | MIT OR Apache-2.0 | [link](https://github.com/servo/rust-url) |
| `urlpattern` | 0.3.0 | MIT | [link](https://github.com/denoland/rust-urlpattern) |
| `utf8_iter` | 1.0.4 | Apache-2.0 OR MIT | [link](https://github.com/hsivonen/utf8_iter) |
| `utf8parse` | 0.2.2 | Apache-2.0 OR MIT | [link](https://github.com/alacritty/vte) |
| `uuid` | 1.24.0 | Apache-2.0 OR MIT | [link](https://github.com/uuid-rs/uuid) |
| `valuable` | 0.1.1 | MIT | [link](https://github.com/tokio-rs/valuable) |
| `version-compare` | 0.2.1 | MIT | [link](https://gitlab.com/timvisee/version-compare) |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | [link](https://github.com/SergioBenitez/version_check) |
| `vswhom` | 0.1.0 | MIT | [link](https://github.com/nabijaczleweli/vswhom.rs) |
| `vswhom-sys` | 0.1.3 | MIT | [link](https://github.com/nabijaczleweli/vswhom-sys.rs) |
| `walkdir` | 2.5.0 | Unlicense/MIT | [link](https://github.com/BurntSushi/walkdir) |
| `want` | 0.3.1 | MIT | [link](https://github.com/seanmonstar/want) |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [link](https://github.com/bytecodealliance/wasi) |
| `wasip2` | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [link](https://github.com/bytecodealliance/wasi-rs) |
| `wasm-bindgen` | 0.2.126 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen) |
| `wasm-bindgen-futures` | 0.4.76 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| `wasm-bindgen-macro` | 0.2.126 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| `wasm-bindgen-macro-support` | 0.2.126 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support) |
| `wasm-bindgen-shared` | 0.2.126 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| `wasm-streams` | 0.5.0 | MIT OR Apache-2.0 | [link](https://github.com/MattiasBuelens/wasm-streams/) |
| `web-sys` | 0.3.103 | MIT OR Apache-2.0 | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 | [link](https://github.com/daxpedda/web-time) |
| `web_atoms` | 0.2.5 | MIT OR Apache-2.0 | [link](https://github.com/servo/html5ever) |
| `webkit2gtk` | 2.0.2 | MIT | [link](https://github.com/tauri-apps/webkit2gtk-rs) |
| `webkit2gtk-sys` | 2.0.2 | MIT | [link](https://github.com/tauri-apps/webkit2gtk-rs) |
| `webpki-root-certs` | 1.0.9 | CDLA-Permissive-2.0 | [link](https://github.com/rustls/webpki-roots) |
| `webview2-com` | 0.38.2 | MIT | [link](https://github.com/wravery/webview2-rs) |
| `webview2-com-macros` | 0.8.1 | MIT | [link](https://github.com/wravery/webview2-rs) |
| `webview2-com-sys` | 0.38.2 | MIT | [link](https://github.com/wravery/webview2-rs) |
| `whisper-rs` | 0.16.0 | Unlicense | [link](https://codeberg.org/tazz4843/whisper-rs) |
| `whisper-rs-sys` | 0.15.0 | Unlicense | [link](https://codeberg.org/tazz4843/whisper-rs) |
| `winapi` | 0.3.9 | MIT/Apache-2.0 | [link](https://github.com/retep998/winapi-rs) |
| `winapi-i686-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | [link](https://github.com/retep998/winapi-rs) |
| `winapi-util` | 0.1.11 | Unlicense OR MIT | [link](https://github.com/BurntSushi/winapi-util) |
| `winapi-x86_64-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | [link](https://github.com/retep998/winapi-rs) |
| `window-vibrancy` | 0.6.0 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri-plugin-vibrancy) |
| `windows` | 0.61.3 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-collections` | 0.2.0 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.61.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.62.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-future` | 0.2.1 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.1.3 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-numerics` | 0.2.0 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.3.4 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.4.1 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.4.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.5.1 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.45.0 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.59.0 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-threading` | 0.1.0 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows-version` | 0.1.7 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.42.2 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 | [link](https://github.com/microsoft/windows-rs) |
| `winnow` | 0.5.40 | MIT | [link](https://github.com/winnow-rs/winnow) |
| `winnow` | 0.7.15 | MIT | [link](https://github.com/winnow-rs/winnow) |
| `winnow` | 1.0.4 | MIT | [link](https://github.com/winnow-rs/winnow) |
| `winreg` | 0.55.0 | MIT | [link](https://github.com/gentoo90/winreg-rs) |
| `wit-bindgen` | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [link](https://github.com/bytecodealliance/wit-bindgen) |
| `writeable` | 0.6.3 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `wry` | 0.55.1 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/wry) |
| `x11` | 2.21.0 | MIT | [link](https://github.com/AltF02/x11-rs.git) |
| `x11-dl` | 2.21.0 | MIT | [link](https://github.com/AltF02/x11-rs.git) |
| `yoke` | 0.8.3 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `yoke-derive` | 0.8.2 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `zbus` | 5.18.0 | MIT | [link](https://github.com/z-galaxy/zbus/) |
| `zbus_macros` | 5.18.0 | MIT | [link](https://github.com/z-galaxy/zbus/) |
| `zbus_names` | 4.3.4 | MIT | [link](https://github.com/z-galaxy/zbus/) |
| `zerofrom` | 0.1.8 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `zerofrom-derive` | 0.1.7 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `zeroize` | 1.9.0 | Apache-2.0 OR MIT | [link](https://github.com/RustCrypto/utils) |
| `zerotrie` | 0.2.4 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `zerovec` | 0.11.6 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `zerovec-derive` | 0.11.3 | Unicode-3.0 | [link](https://github.com/unicode-org/icu4x) |
| `zmij` | 1.0.23 | MIT | [link](https://github.com/dtolnay/zmij) |
| `zvariant` | 5.13.1 | MIT | [link](https://github.com/z-galaxy/zbus/) |
| `zvariant_derive` | 5.13.1 | MIT | [link](https://github.com/z-galaxy/zbus/) |
| `zvariant_utils` | 3.5.0 | MIT | [link](https://github.com/z-galaxy/zbus/) |

## npm packages

61 packages. Most of these are build tooling and no part of them reaches
the shipped app - the exceptions are the Svelte runtime and `@tauri-apps/api`,
which are compiled into the JavaScript bundle. They are listed together because
"which bundler output contains which module" is not a question a dependency tree
can answer honestly.

| Package | Version | License | Source |
|---|---|---|---|
| `@esbuild/darwin-arm64` | 0.25.12 | MIT | [link](https://github.com/evanw/esbuild) |
| `@jridgewell/gen-mapping` | 0.3.13 | MIT | [link](https://github.com/jridgewell/sourcemaps) |
| `@jridgewell/remapping` | 2.3.5 | MIT | [link](https://github.com/jridgewell/sourcemaps) |
| `@jridgewell/resolve-uri` | 3.1.2 | MIT | [link](https://github.com/jridgewell/resolve-uri) |
| `@jridgewell/sourcemap-codec` | 1.5.5 | MIT | [link](https://github.com/jridgewell/sourcemaps) |
| `@jridgewell/trace-mapping` | 0.3.31 | MIT | [link](https://github.com/jridgewell/sourcemaps) |
| `@polka/url` | 1.0.0-next.29 | MIT | |
| `@rollup/rollup-darwin-arm64` | 4.62.3 | MIT | [link](https://github.com/rollup/rollup) |
| `@standard-schema/spec` | 1.1.0 | MIT | [link](https://github.com/standard-schema/standard-schema) |
| `@sveltejs/acorn-typescript` | 1.0.11 | MIT | [link](https://github.com/sveltejs/acorn-typescript) |
| `@sveltejs/adapter-static` | 3.0.10 | MIT | [link](https://github.com/sveltejs/kit) |
| `@sveltejs/kit` | 2.70.2 | MIT | [link](https://github.com/sveltejs/kit) |
| `@sveltejs/load-config` | 0.2.1 | MIT | [link](https://github.com/sveltejs/language-tools) |
| `@sveltejs/vite-plugin-svelte` | 5.1.1 | MIT | [link](https://github.com/sveltejs/vite-plugin-svelte) |
| `@sveltejs/vite-plugin-svelte-inspector` | 4.0.1 | MIT | [link](https://github.com/sveltejs/vite-plugin-svelte) |
| `@tauri-apps/api` | 2.11.1 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `@tauri-apps/cli` | 2.11.4 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `@tauri-apps/cli-darwin-arm64` | 2.11.4 | Apache-2.0 OR MIT | [link](https://github.com/tauri-apps/tauri) |
| `@tauri-apps/plugin-opener` | 2.5.4 | MIT OR Apache-2.0 | [link](https://github.com/tauri-apps/plugins-workspace) |
| `@types/cookie` | 0.6.0 | MIT | [link](https://github.com/DefinitelyTyped/DefinitelyTyped) |
| `@types/estree` | 1.0.9 | MIT | [link](https://github.com/DefinitelyTyped/DefinitelyTyped) |
| `@types/trusted-types` | 2.0.7 | MIT | [link](https://github.com/DefinitelyTyped/DefinitelyTyped) |
| `acorn` | 8.18.0 | MIT | [link](https://github.com/acornjs/acorn) |
| `aria-query` | 5.3.1 | Apache-2.0 | [link](https://github.com/A11yance/aria-query) |
| `axobject-query` | 4.1.0 | Apache-2.0 | [link](https://github.com/A11yance/axobject-query) |
| `chokidar` | 4.0.3 | MIT | [link](https://github.com/paulmillr/chokidar) |
| `clsx` | 2.1.1 | MIT | |
| `cookie` | 0.6.0 | MIT | |
| `debug` | 4.4.3 | MIT | [link](https://github.com/debug-js/debug) |
| `deepmerge` | 4.3.1 | MIT | [link](https://github.com/TehShrike/deepmerge) |
| `devalue` | 5.8.2 | MIT | |
| `esbuild` | 0.25.12 | MIT | [link](https://github.com/evanw/esbuild) |
| `esm-env` | 1.2.2 | MIT | [link](https://github.com/benmccann/esm-env) |
| `esrap` | 2.3.0 | MIT | [link](https://github.com/sveltejs/esrap) |
| `fdir` | 6.5.0 | MIT | [link](https://github.com/thecodrr/fdir) |
| `fsevents` | 2.3.3 | MIT | [link](https://github.com/fsevents/fsevents) |
| `is-reference` | 3.0.3 | MIT | [link](https://github.com/Rich-Harris/is-reference) |
| `kleur` | 4.1.5 | MIT | |
| `locate-character` | 3.0.0 | MIT | [link](https://gitlab.com/Rich-Harris/locate-character) |
| `magic-string` | 0.30.21 | MIT | [link](https://github.com/Rich-Harris/magic-string) |
| `mri` | 1.2.0 | MIT | |
| `mrmime` | 2.0.1 | MIT | |
| `ms` | 2.1.3 | MIT | |
| `nanoid` | 3.3.18 | MIT | |
| `picocolors` | 1.1.1 | ISC | |
| `picomatch` | 4.0.5 | MIT | |
| `postcss` | 8.5.25 | MIT | |
| `readdirp` | 4.1.2 | MIT | [link](https://github.com/paulmillr/readdirp) |
| `rollup` | 4.62.3 | MIT | [link](https://github.com/rollup/rollup) |
| `sade` | 1.8.1 | MIT | |
| `set-cookie-parser` | 3.1.2 | MIT | |
| `sirv` | 3.0.2 | MIT | |
| `source-map-js` | 1.2.1 | BSD-3-Clause | |
| `svelte` | 5.56.8 | MIT | [link](https://github.com/sveltejs/svelte) |
| `svelte-check` | 4.7.4 | MIT | [link](https://github.com/sveltejs/language-tools) |
| `tinyglobby` | 0.2.17 | MIT | [link](https://github.com/SuperchupuDev/tinyglobby) |
| `totalist` | 3.0.1 | MIT | |
| `typescript` | 5.6.3 | Apache-2.0 | [link](https://github.com/microsoft/TypeScript) |
| `vite` | 6.4.3 | MIT | [link](https://github.com/vitejs/vite) |
| `vitefu` | 1.1.3 | MIT | [link](https://github.com/svitejs/vitefu) |
| `zimmerframe` | 1.1.4 | MIT | [link](https://github.com/sveltejs/zimmerframe) |
