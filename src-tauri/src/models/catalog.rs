//! The models a user can install, with the checksums that make an interrupted
//! or corrupted download impossible to mistake for a good one.
//!
//! Sizes and SHA-256 values come from the Hugging Face LFS metadata of the
//! source repositories.
//!
//! Every entry also carries the license its weights come under, shown on the
//! row before the download starts. Marswind does not redistribute any of them -
//! they are fetched on request, from the repository named in `url` - but it is
//! the thing doing the fetching, and somebody choosing between two rows
//! deserves to know what they are agreeing to.
//!
//! Two rules decide what gets a row, and both are held by tests below.
//!
//! **Everything offered is open source**: MIT or Apache-2.0, nothing else. A
//! model whose terms a user would have to read before pressing Install does not
//! belong behind an Install button.
//!
//! **Nothing is English-only.** whisper's `.en` builds are smaller and more
//! accurate at their size, and this is an app for watching things in languages
//! you do not speak; a recognizer that only hears English cannot do that. See
//! docs/MODELS.md.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelKind {
    /// Speech recognition.
    Asr,
    /// Voice activity detection, used to cut audio into phrases.
    Vad,
    /// Translation.
    Mt,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelSpec {
    pub id: &'static str,
    pub kind: ModelKind,
    pub name: &'static str,
    /// One line explaining who this model is for.
    pub note: &'static str,
    pub url: &'static str,
    pub file_name: &'static str,
    pub size_bytes: u64,
    pub sha256: &'static str,
    /// What the weights are licensed under, as the user should read it.
    pub license: &'static str,
    /// Where those terms are stated, opened from the row in Settings.
    pub license_url: &'static str,
}

// Source repositories, spelled out in each entry's `url` because `concat!`
// only takes literals:
//   whisper: https://huggingface.co/ggerganov/whisper.cpp/resolve/main
//   vad:     https://huggingface.co/ggml-org/whisper-vad/resolve/main

/// Both sets of terms in the catalog, as (name, where they are stated). Named
/// rather than repeated so a typo cannot put one model under a license it is
/// not published under.
const MIT: (&str, &str) = ("MIT", "https://opensource.org/license/mit");
const APACHE_2: (&str, &str) = ("Apache-2.0", "https://www.apache.org/licenses/LICENSE-2.0");

pub const VAD_MODEL_ID: &str = "silero-v5.1.2";

pub static CATALOG: &[ModelSpec] = &[
    ModelSpec {
        id: "tiny",
        kind: ModelKind::Asr,
        name: "Tiny",
        note: "Fastest and roughest. For old hardware or a quick smoke test.",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main",
            "/ggml-tiny.bin"
        ),
        file_name: "ggml-tiny.bin",
        size_bytes: 77_691_713,
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        license: MIT.0,
        license_url: MIT.1,
    },
    ModelSpec {
        id: "base",
        kind: ModelKind::Asr,
        name: "Base",
        note: "Compromise for machines with little memory.",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main",
            "/ggml-base.bin"
        ),
        file_name: "ggml-base.bin",
        size_bytes: 147_951_465,
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        license: MIT.0,
        license_url: MIT.1,
    },
    ModelSpec {
        id: "small",
        kind: ModelKind::Asr,
        name: "Small",
        note: "The practical quality floor. Understands 99 languages.",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main",
            "/ggml-small.bin"
        ),
        file_name: "ggml-small.bin",
        size_bytes: 487_601_967,
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        license: MIT.0,
        license_url: MIT.1,
    },
    ModelSpec {
        id: "large-v3-turbo-q5_0",
        kind: ModelKind::Asr,
        name: "Large v3 Turbo (compressed)",
        note: "Best quality per gigabyte. Recommended on Apple Silicon.",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main",
            "/ggml-large-v3-turbo-q5_0.bin"
        ),
        file_name: "ggml-large-v3-turbo-q5_0.bin",
        size_bytes: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        license: MIT.0,
        license_url: MIT.1,
    },
    ModelSpec {
        id: "medium",
        kind: ModelKind::Asr,
        name: "Medium",
        note: "Strong quality, noticeably heavier than Turbo for similar results.",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main",
            "/ggml-medium.bin"
        ),
        file_name: "ggml-medium.bin",
        size_bytes: 1_533_763_059,
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        license: MIT.0,
        license_url: MIT.1,
    },
    ModelSpec {
        id: "large-v3-turbo",
        kind: ModelKind::Asr,
        name: "Large v3 Turbo",
        note: "Highest accuracy. Wants 8 GB of free memory.",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main",
            "/ggml-large-v3-turbo.bin"
        ),
        file_name: "ggml-large-v3-turbo.bin",
        size_bytes: 1_624_555_275,
        sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
        license: MIT.0,
        license_url: MIT.1,
    },
    ModelSpec {
        id: "qwen3-4b-instruct-q4",
        kind: ModelKind::Mt,
        name: "Qwen3 4B Instruct (compressed)",
        note: "Translates into any language and follows the conversation. Wants 4 GB free.",
        url: concat!(
            "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main",
            "/Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
        ),
        file_name: "Qwen3-4B-Instruct-2507-Q4_K_M.gguf",
        size_bytes: 2_497_281_120,
        sha256: "3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597",
        license: APACHE_2.0,
        license_url: APACHE_2.1,
    },
    ModelSpec {
        id: "qwen3-1.7b-q4",
        kind: ModelKind::Mt,
        name: "Qwen3 1.7B (compressed)",
        note: "Half the memory, and measurably clumsier. Only worth it below 16 GB.",
        url: concat!(
            "https://huggingface.co/ggml-org/Qwen3-1.7B-GGUF/resolve/main",
            "/Qwen3-1.7B-Q4_K_M.gguf"
        ),
        file_name: "Qwen3-1.7B-Q4_K_M.gguf",
        size_bytes: 1_282_439_264,
        sha256: "d2387ca2dbfee2ffabce7120d3770dadca0b293052bc2f0e138fdc940d9bc7b5",
        license: APACHE_2.0,
        license_url: APACHE_2.1,
    },
    ModelSpec {
        id: "qwen3-8b-q4",
        kind: ModelKind::Mt,
        name: "Qwen3 8B (compressed)",
        note: "Steadier than the 4B on long sentences. Wants 8 GB free and is slower per line.",
        url: concat!(
            "https://huggingface.co/unsloth/Qwen3-8B-GGUF/resolve/main",
            "/Qwen3-8B-Q4_K_M.gguf"
        ),
        file_name: "Qwen3-8B-Q4_K_M.gguf",
        size_bytes: 5_027_784_512,
        sha256: "120307ba529eb2439d6c430d94104dabd578497bc7bfe7e322b5d9933b449bd4",
        license: APACHE_2.0,
        license_url: APACHE_2.1,
    },
    ModelSpec {
        id: VAD_MODEL_ID,
        kind: ModelKind::Vad,
        name: "Silero VAD",
        note: "Finds phrase boundaries. Required, and tiny.",
        url: concat!(
            "https://huggingface.co/ggml-org/whisper-vad/resolve/main",
            "/ggml-silero-v5.1.2.bin"
        ),
        file_name: "ggml-silero-v5.1.2.bin",
        size_bytes: 885_098,
        sha256: "29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf",
        license: MIT.0,
        license_url: MIT.1,
    },
];

pub fn find(id: &str) -> Option<&'static ModelSpec> {
    CATALOG.iter().find(|spec| spec.id == id)
}

/// Picks a default recognition model for this machine. Erring small is the
/// right call: a model that runs behind real time makes the app useless, while
/// a smaller one merely makes it less accurate.
pub fn recommended_asr(total_memory_bytes: u64) -> &'static str {
    const GB: u64 = 1024 * 1024 * 1024;

    if total_memory_bytes >= 16 * GB {
        "large-v3-turbo-q5_0"
    } else if total_memory_bytes >= 8 * GB {
        "small"
    } else {
        "base"
    }
}

/// Same reasoning for translation, and stricter: recognition and translation
/// share the machine with whatever is being transcribed.
pub fn recommended_mt(total_memory_bytes: u64) -> &'static str {
    const GB: u64 = 1024 * 1024 * 1024;

    if total_memory_bytes >= 16 * GB {
        "qwen3-4b-instruct-q4"
    } else {
        "qwen3-1.7b-q4"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_is_addressable_and_unique() {
        for spec in CATALOG {
            assert_eq!(find(spec.id).map(|s| s.id), Some(spec.id));
            assert_eq!(
                CATALOG.iter().filter(|s| s.id == spec.id).count(),
                1,
                "duplicate id {}",
                spec.id
            );
        }
    }

    #[test]
    fn checksums_are_well_formed() {
        for spec in CATALOG {
            assert_eq!(spec.sha256.len(), 64, "{} has a malformed digest", spec.id);
            assert!(
                spec.sha256.chars().all(|c| c.is_ascii_hexdigit()),
                "{} has a non-hex digest",
                spec.id
            );
            assert!(spec.size_bytes > 0);
            assert!(
                spec.url.ends_with(spec.file_name),
                "{} url/file mismatch",
                spec.id
            );
        }
    }

    /// The catalog is where the app tells a user what they are agreeing to, so
    /// a row carrying the wrong terms is a false statement about somebody
    /// else's license, made on their behalf.
    #[test]
    fn every_entry_states_its_terms() {
        for spec in CATALOG {
            let expected = if spec.url.contains("/Qwen3-") {
                APACHE_2
            } else {
                MIT
            };
            assert_eq!(
                (spec.license, spec.license_url),
                expected,
                "{} is published under different terms from the ones listed",
                spec.id
            );
        }
    }

    /// Anything a user can install through this app is under a license they can
    /// take at face value: MIT or Apache-2.0 and nothing else. A model whose
    /// terms reach as far as what the output may be used for fails here rather
    /// than in somebody's issue.
    #[test]
    fn nothing_offered_is_outside_an_open_source_license() {
        for spec in CATALOG {
            assert!(
                [MIT, APACHE_2].contains(&(spec.license, spec.license_url)),
                "{} is offered under '{}', which is not on the open-source list",
                spec.id,
                spec.license
            );
            assert!(
                spec.license_url.starts_with("https://"),
                "{} has no link to its terms",
                spec.id
            );
        }
    }

    /// Recognition that only hears English is no use to somebody watching a
    /// film in a language they do not speak, which is the whole app. The
    /// `.en` builds are smaller and better at their size, and they are still
    /// not offered.
    #[test]
    fn no_english_only_models_are_offered() {
        for spec in CATALOG {
            assert!(
                !spec.file_name.contains(".en."),
                "{} is an English-only build",
                spec.id
            );
        }
    }

    #[test]
    fn a_vad_model_exists() {
        let vad = find(VAD_MODEL_ID).expect("VAD model must be in the catalog");
        assert_eq!(vad.kind, ModelKind::Vad);
    }

    #[test]
    fn recommendation_scales_with_memory() {
        const GB: u64 = 1024 * 1024 * 1024;
        assert_eq!(recommended_asr(4 * GB), "base");
        assert_eq!(recommended_asr(8 * GB), "small");
        assert_eq!(recommended_asr(32 * GB), "large-v3-turbo-q5_0");
    }

    #[test]
    fn recommendations_point_at_real_entries() {
        const GB: u64 = 1024 * 1024 * 1024;
        for memory in [2, 8, 16, 64] {
            assert!(find(recommended_asr(memory * GB)).is_some());
            assert!(find(recommended_mt(memory * GB)).is_some());
        }
    }

    #[test]
    fn recommendations_have_the_right_kind() {
        const GB: u64 = 1024 * 1024 * 1024;
        assert_eq!(find(recommended_asr(24 * GB)).unwrap().kind, ModelKind::Asr);
        assert_eq!(find(recommended_mt(24 * GB)).unwrap().kind, ModelKind::Mt);
    }

    #[test]
    fn a_translation_model_exists_for_small_machines() {
        const GB: u64 = 1024 * 1024 * 1024;
        let small = find(recommended_mt(8 * GB)).unwrap();
        let large = find(recommended_mt(32 * GB)).unwrap();
        assert!(small.size_bytes < large.size_bytes);
    }
}
