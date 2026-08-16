//! Target languages offered for translation.
//!
//! The English name is what goes into the prompt - instruction-tuned models
//! respond to "Russian" far more reliably than to "ru".

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    /// ISO 639-1 code, used as the stable identifier.
    pub code: &'static str,
    /// Name in English, for the model.
    pub name: &'static str,
    /// Name in the language itself, for the person choosing it.
    pub endonym: &'static str,
}

/// Same order as the interface languages in `src/lib/i18n.ts`: the dropdown a
/// reader picks a target language from and the one they pick the window's
/// language from are the same list, and a list that reorders itself between two
/// panels is one they have to read twice.
pub static LANGUAGES: &[Language] = &[
    Language {
        code: "en",
        name: "English",
        endonym: "English",
    },
    Language {
        code: "ru",
        name: "Russian",
        endonym: "Русский",
    },
    Language {
        code: "de",
        name: "German",
        endonym: "Deutsch",
    },
    Language {
        code: "es",
        name: "Spanish",
        endonym: "Español",
    },
    Language {
        code: "fr",
        name: "French",
        endonym: "Français",
    },
    Language {
        code: "it",
        name: "Italian",
        endonym: "Italiano",
    },
    Language {
        code: "pt",
        name: "Portuguese",
        endonym: "Português",
    },
    Language {
        code: "pl",
        name: "Polish",
        endonym: "Polski",
    },
    Language {
        code: "tr",
        name: "Turkish",
        endonym: "Türkçe",
    },
    Language {
        code: "uk",
        name: "Ukrainian",
        endonym: "Українська",
    },
    Language {
        code: "zh",
        name: "Chinese",
        endonym: "中文",
    },
    Language {
        code: "ja",
        name: "Japanese",
        endonym: "日本語",
    },
    Language {
        code: "ko",
        name: "Korean",
        endonym: "한국어",
    },
];

pub fn find(code: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|language| language.code == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_are_addressable_by_code() {
        assert_eq!(find("ru").map(|l| l.name), Some("Russian"));
        assert_eq!(find("ja").map(|l| l.endonym), Some("日本語"));
        assert!(find("xx").is_none());
    }

    #[test]
    fn codes_are_unique_and_well_formed() {
        for language in LANGUAGES {
            assert_eq!(language.code.len(), 2, "{} is not ISO 639-1", language.code);
            assert_eq!(
                LANGUAGES.iter().filter(|l| l.code == language.code).count(),
                1
            );
            assert!(!language.name.is_empty());
            assert!(!language.endonym.is_empty());
        }
    }
}
