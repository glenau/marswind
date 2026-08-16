//! LocalAgreement-2: deciding which words are safe to show as final.
//!
//! Whisper is not a streaming model. Run it repeatedly on a growing window and
//! it will happily rewrite words it produced a moment ago, which on screen
//! looks like subtitles thrashing. The fix is to trust only what two
//! consecutive runs agree on: the longest common prefix of the previous
//! hypothesis and the current one is committed, the rest stays provisional.

#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    /// End of this word in seconds from the start of the current window.
    pub end: f64,
}

#[derive(Default)]
pub struct LocalAgreement {
    /// The tail of the previous hypothesis that has not been committed yet.
    previous: Vec<Word>,
}

impl LocalAgreement {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds the newest hypothesis and returns the words that just became
    /// stable. The uncommitted remainder is kept for the next comparison.
    pub fn advance(&mut self, hypothesis: &[Word]) -> Vec<Word> {
        let agreed = common_prefix_len(&self.previous, hypothesis);
        let committed = hypothesis[..agreed].to_vec();
        self.previous = hypothesis[agreed..].to_vec();
        committed
    }

    /// Words seen but not yet confirmed - rendered dimmed in the UI.
    pub fn tentative(&self) -> &[Word] {
        &self.previous
    }

    /// Commits everything outstanding. Used when a phrase ends, where waiting
    /// for a second opinion would only add latency.
    pub fn flush(&mut self) -> Vec<Word> {
        std::mem::take(&mut self.previous)
    }

    pub fn reset(&mut self) {
        self.previous.clear();
    }
}

fn common_prefix_len(a: &[Word], b: &[Word]) -> usize {
    a.iter()
        .zip(b.iter())
        .take_while(|(x, y)| normalize(&x.text) == normalize(&y.text))
        .count()
}

/// Words match on their spoken content, not their rendering: whisper moves
/// punctuation and capitalization around between runs even when it heard the
/// same thing.
fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn join(words: &[Word]) -> String {
    let mut out = String::new();
    for word in words {
        let text = word.text.trim();
        if text.is_empty() {
            continue;
        }
        let needs_space = !out.is_empty()
            && !text.starts_with(|c: char| ",.!?;:%)]}".contains(c))
            && !out.ends_with(['(', '[', '{', '"', '\'', '-']);
        if needs_space {
            out.push(' ');
        }
        out.push_str(text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(items: &[(&str, f64)]) -> Vec<Word> {
        items
            .iter()
            .map(|(text, end)| Word {
                text: (*text).to_string(),
                end: *end,
            })
            .collect()
    }

    #[test]
    fn nothing_is_committed_from_a_single_hypothesis() {
        let mut agreement = LocalAgreement::new();
        let committed = agreement.advance(&words(&[("the", 0.3), ("market", 0.7)]));

        assert!(committed.is_empty());
        assert_eq!(agreement.tentative().len(), 2);
    }

    #[test]
    fn commits_what_two_runs_agree_on() {
        let mut agreement = LocalAgreement::new();
        agreement.advance(&words(&[("the", 0.3), ("market", 0.7)]));

        let committed = agreement.advance(&words(&[("the", 0.3), ("market", 0.7), ("fell", 1.1)]));

        assert_eq!(committed.len(), 2);
        assert_eq!(committed[1].text, "market");
        // "fell" was seen once, so it stays provisional.
        assert_eq!(agreement.tentative().len(), 1);
    }

    #[test]
    fn a_revised_word_is_not_committed() {
        let mut agreement = LocalAgreement::new();
        agreement.advance(&words(&[("the", 0.3), ("mark", 0.6)]));

        // Whisper reconsidered the second word once it heard more audio.
        let committed = agreement.advance(&words(&[("the", 0.3), ("market", 0.7)]));

        assert_eq!(committed.len(), 1);
        assert_eq!(committed[0].text, "the");
    }

    #[test]
    fn punctuation_and_case_do_not_break_agreement() {
        let mut agreement = LocalAgreement::new();
        agreement.advance(&words(&[("the", 0.3), ("market", 0.7)]));

        let committed = agreement.advance(&words(&[("The", 0.3), ("market,", 0.7), ("in", 1.0)]));

        assert_eq!(committed.len(), 2);
    }

    #[test]
    fn flush_commits_the_provisional_tail() {
        let mut agreement = LocalAgreement::new();
        agreement.advance(&words(&[("hello", 0.4), ("there", 0.8)]));

        let flushed = agreement.flush();

        assert_eq!(flushed.len(), 2);
        assert!(agreement.tentative().is_empty());
    }

    #[test]
    fn joins_words_with_natural_spacing() {
        assert_eq!(
            join(&words(&[
                ("Good", 0.2),
                ("morning", 0.5),
                (",", 0.5),
                ("everyone", 0.9)
            ])),
            "Good morning, everyone"
        );
    }

    #[test]
    fn join_skips_empty_fragments() {
        assert_eq!(join(&words(&[("a", 0.1), ("   ", 0.2), ("b", 0.3)])), "a b");
    }
}
