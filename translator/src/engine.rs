//! Translation with a small instruction-tuned model through llama.cpp.
//!
//! An LLM is a heavier way to translate than a dedicated model, and it buys two
//! things worth the weight: any language pair without downloading a new model,
//! and context - the previous few captions go into the prompt, so pronouns and
//! terminology stay consistent across sentences instead of each one being
//! translated in isolation.

use std::path::Path;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

use crate::protocol::Pair;

/// Room for the instruction, a few captions of history and the answer.
const CONTEXT_TOKENS: u32 = 2048;
/// A translation is never much longer than its source; this stops a model that
/// has started rambling.
const MAX_OUTPUT_TOKENS_BASE: usize = 48;
const MAX_OUTPUT_TOKENS_RATIO: usize = 3;

self_cell::self_cell!(
    /// A context borrows its model, which makes the pair self-referential.
    /// `self_cell` owns both and hands out the context, with no unsafe code and
    /// no leaking a multi-gigabyte model to get a `'static` lifetime.
    struct LoadedModel {
        owner: LlamaModel,
        #[covariant]
        dependent: Context,
    }
);

type Context<'model> = LlamaContext<'model>;

pub struct LlmTranslator {
    loaded: LoadedModel,
    /// Kept for the prompt format, which differs between model families.
    template: PromptTemplate,
    /// The prompt tokens the model's KV cache currently holds.
    ///
    /// Every request repeats the instruction and the conversation so far and
    /// only adds a turn at the end, so re-reading the whole prompt each time is
    /// most of the work - and it is work done on the same GPU that recognition
    /// needs. Keeping what is already there and decoding only the new tail is
    /// what lets captions be short without the translator starving the
    /// recognizer.
    cached: Vec<LlamaToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptTemplate {
    /// ChatML, as used by Qwen.
    ChatMl,
    /// Gemma. Two things differ and both matter: the turn markers, and the
    /// absence of a system role - the instruction has to ride along with the
    /// first user turn or the model never sees it.
    Gemma,
}

impl PromptTemplate {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "chatml" => Some(Self::ChatMl),
            "gemma" => Some(Self::Gemma),
            _ => None,
        }
    }
}

impl LlmTranslator {
    pub fn load(
        backend: &LlamaBackend,
        path: &Path,
        threads: i32,
        template: PromptTemplate,
    ) -> Result<Self, String> {
        let params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(backend, path, &params)
            .map_err(|e| format!("could not load the model: {e}"))?;

        let context_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(CONTEXT_TOKENS))
            .with_n_threads(threads)
            .with_n_threads_batch(threads);

        let loaded = LoadedModel::try_new(model, |model| {
            model
                .new_context(backend, context_params)
                .map_err(|e| format!("could not load the model: {e}"))
        })?;

        Ok(Self {
            loaded,
            template,
            cached: Vec::new(),
        })
    }
}

/// How many leading tokens two prompts share.
fn common_prefix_len(a: &[LlamaToken], b: &[LlamaToken]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

impl LlmTranslator {
    /// Translates one caption, calling `on_delta` with each new piece of the
    /// answer as it is generated. The returned string is the whole translation
    /// and is authoritative: the pieces are there to put words on screen early,
    /// not to save the caller from reading the result.
    pub fn translate(
        &mut self,
        source: &str,
        history: &[Pair],
        target_language: &str,
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        let prompt = build_prompt(self.template, source, history, target_language);

        let template = self.template;
        let cached = std::mem::take(&mut self.cached);
        let mut decoded: Vec<LlamaToken> = Vec::new();

        let result = self.loaded.with_dependent_mut(|model, context| {
            let tokens = model
                .str_to_token(&prompt, AddBos::Never)
                .map_err(|e| e.to_string())?;

            if tokens.len() >= CONTEXT_TOKENS as usize {
                return Err("prompt does not fit in the context window".to_string());
            }

            // At least one token has to go through the model: the logits that
            // start the answer come from decoding it, so a prompt identical to
            // the last one still needs its final token.
            let reused = common_prefix_len(&cached, &tokens).min(tokens.len() - 1);

            // Everything past the shared prefix is stale - the previous answer
            // and whatever turn it belonged to - and has to go before the new
            // tail is decoded into those positions.
            context
                .clear_kv_cache_seq(Some(0), Some(reused as u32), None)
                .map_err(|e| e.to_string())?;

            let fresh = &tokens[reused..];
            let mut batch = LlamaBatch::new(fresh.len().max(1), 1);
            let last = fresh.len() - 1;
            for (offset, token) in fresh.iter().enumerate() {
                batch
                    .add(*token, (reused + offset) as i32, &[0], offset == last)
                    .map_err(|e| e.to_string())?;
            }
            context.decode(&mut batch).map_err(|e| e.to_string())?;
            decoded = tokens.clone();

            // Translation is not a creative task: sampling only adds ways to be
            // wrong.
            let mut sampler = LlamaSampler::greedy();
            let budget =
                MAX_OUTPUT_TOKENS_BASE + tokens.len().saturating_mul(MAX_OUTPUT_TOKENS_RATIO);

            // A character can be split across two tokens, so bytes are buffered
            // and only the part that is complete UTF-8 is decoded - decoding
            // every token on its own would turn Cyrillic into replacement
            // characters.
            let mut stream = Stream::new(template);
            let mut pending: Vec<u8> = Vec::new();

            // Generation picks up where the prompt ended, so the counter is the
            // position in the context rather than the number of tokens so far.
            for position in (tokens.len() as i32..).take(budget) {
                let token = sampler.sample(context, -1);
                if model.is_eog_token(token) {
                    break;
                }
                sampler.accept(token);

                let bytes = model
                    .token_to_piece_bytes(token, 64, false, None)
                    .map_err(|e| e.to_string())?;
                pending.extend_from_slice(&bytes);

                let valid = match std::str::from_utf8(&pending) {
                    Ok(text) => text.len(),
                    Err(error) => error.valid_up_to(),
                };
                if valid > 0 {
                    let text = String::from_utf8_lossy(&pending[..valid]).into_owned();
                    pending.drain(..valid);
                    if let Some(delta) = stream.push(&text) {
                        on_delta(&delta);
                    }
                }

                batch.clear();
                batch
                    .add(token, position, &[0], true)
                    .map_err(|e| e.to_string())?;
                context.decode(&mut batch).map_err(|e| e.to_string())?;
            }

            Ok(stream.finish())
        });

        // A failed request leaves the cache in a state nobody has a record of,
        // so the next one starts from nothing rather than from a guess.
        self.cached = if result.is_ok() { decoded } else { Vec::new() };
        result
    }
}

fn stop_sequence(template: PromptTemplate) -> Option<&'static str> {
    match template {
        PromptTemplate::ChatMl => Some("<|im_end|>"),
        PromptTemplate::Gemma => Some("<end_of_turn>"),
    }
}

/// Everything the model may emit that must never reach the screen.
fn markers(template: PromptTemplate) -> &'static [&'static str] {
    match template {
        PromptTemplate::ChatMl => &["<|im_end|>", "<think>", "</think>"],
        PromptTemplate::Gemma => &["<end_of_turn>", "<start_of_turn>", "<think>", "</think>"],
    }
}

/// Assembles the answer as it is generated and decides how much of it is safe
/// to show.
///
/// Two things make this more than a running substring. A marker arrives one
/// token at a time, so anything that could still become one is held back rather
/// than shown and then taken away. And a half-generated word reads as a typo,
/// so text is released a whole word at a time.
struct Stream {
    template: PromptTemplate,
    raw: String,
    /// Bytes of the visible text already handed to the caller.
    emitted: usize,
}

impl Stream {
    fn new(template: PromptTemplate) -> Self {
        Self {
            template,
            raw: String::new(),
            emitted: 0,
        }
    }

    /// Adds newly generated text and returns whatever became safe to show.
    fn push(&mut self, piece: &str) -> Option<String> {
        self.raw.push_str(piece);

        let visible = self.visible();
        if visible.len() <= self.emitted {
            return None;
        }

        let fresh = &visible[self.emitted..];
        let cut = fresh
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map(|(index, c)| index + c.len_utf8())?;

        let delta = fresh[..cut].to_string();
        self.emitted += cut;
        Some(delta)
    }

    /// The whole translation, cleaned the same way it always was.
    fn finish(&self) -> String {
        clean_output(&self.raw, self.template)
    }

    /// The text so far, minus anything that might still turn into a marker.
    /// Always a prefix of what `finish` will return, which is what lets the
    /// caller keep appending instead of redrawing.
    fn visible(&self) -> String {
        let held = hold_back_partial_marker(&self.raw, markers(self.template));
        let text = strip_markers(held, self.template);
        let text = text.trim_start();
        // A quoted answer has its quotes stripped at the end; showing one and
        // removing it later is worse than never showing it.
        text.strip_prefix('"').unwrap_or(text).to_string()
    }
}

/// Trims the longest tail that could still grow into a marker: `"<|im"` is not
/// text, it is the first four characters of `<|im_end|>`.
fn hold_back_partial_marker<'a>(text: &'a str, markers: &[&str]) -> &'a str {
    let longest = markers.iter().map(|m| m.len()).max().unwrap_or(0);

    for length in (1..=longest.min(text.len())).rev() {
        let start = text.len() - length;
        if !text.is_char_boundary(start) {
            continue;
        }
        let tail = &text[start..];
        if markers
            .iter()
            .any(|marker| marker.len() > length && marker.starts_with(tail))
        {
            return &text[..start];
        }
    }

    text
}

/// Removes the stop marker and the reasoning blocks that hybrid models emit
/// even when told not to. Everything before the first unclosed `<think>` is
/// kept, so the result only ever grows as more text arrives.
fn strip_markers(raw: &str, template: PromptTemplate) -> String {
    let mut text = raw.to_string();

    if let Some(end) = stop_sequence(template) {
        if let Some(index) = text.find(end) {
            text.truncate(index);
        }
    }

    while let Some(start) = text.find("<think>") {
        match text[start..].find("</think>") {
            Some(offset) => {
                let end = start + offset + "</think>".len();
                text.replace_range(start..end, "");
            }
            None => {
                text.truncate(start);
                break;
            }
        }
    }

    text
}

/// Builds the prompt. Previous captions go in as real conversation turns rather
/// than as a block of text: the model already knows how to continue a
/// translation dialogue, and it keeps the current sentence clearly separated
/// from its context.
fn build_prompt(
    template: PromptTemplate,
    source: &str,
    history: &[Pair],
    target_language: &str,
) -> String {
    let instruction = instruction(target_language);

    match template {
        PromptTemplate::ChatMl => {
            let mut prompt = format!("<|im_start|>system\n{instruction} /no_think<|im_end|>\n");
            for pair in history {
                prompt.push_str(&format!(
                    "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n{}<|im_end|>\n",
                    pair.source.trim(),
                    pair.target.trim()
                ));
            }
            prompt.push_str(&format!(
                "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                source.trim()
            ));
            prompt
        }
        PromptTemplate::Gemma => {
            // No system role: the instruction goes on the front of the first
            // user turn, which is where Gemma's own chat template puts it.
            let mut prompt = String::new();
            let mut instructed = false;
            let mut turn = |prompt: &mut String, user: &str, model: Option<&str>| {
                prompt.push_str("<start_of_turn>user\n");
                if !instructed {
                    prompt.push_str(&instruction);
                    prompt.push_str("\n\n");
                    instructed = true;
                }
                prompt.push_str(user);
                prompt.push_str("<end_of_turn>\n");
                match model {
                    Some(answer) => {
                        prompt.push_str("<start_of_turn>model\n");
                        prompt.push_str(answer);
                        prompt.push_str("<end_of_turn>\n");
                    }
                    None => prompt.push_str("<start_of_turn>model\n"),
                }
            };

            for pair in history {
                turn(&mut prompt, pair.source.trim(), Some(pair.target.trim()));
            }
            turn(&mut prompt, source.trim(), None);
            prompt
        }
    }
}

/// The one instruction, shared by every template. Captions are clauses rather
/// than sentences, so the prompt says so: a fragment stays a fragment, and a
/// sentence the speaker has not finished is not finished for them.
fn instruction(target_language: &str) -> String {
    format!(
        "You are translating live subtitles into {target_language}. Each message is \
the next piece of one continuous speech, and a piece is often a clause rather than a \
whole sentence. Translate only the current message, continuing naturally from what came \
before: keep a fragment a fragment, do not finish a sentence the speaker has not \
finished, and do not repeat anything already translated. Reply with the translation \
only - no explanations, no notes, no quotation marks. Keep the tone and register of the \
original."
    )
}

/// Trims what the model adds around the translation: the stop marker, thinking
/// blocks from hybrid-reasoning models, and the quotation marks it sometimes
/// wraps the answer in despite being asked not to.
fn clean_output(raw: &str, template: PromptTemplate) -> String {
    let text = strip_markers(raw, template);

    let text = text.trim();
    let text = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .unwrap_or(text);

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_gemma_prompt_carries_the_instruction_in_its_first_user_turn() {
        let history = vec![Pair {
            source: "The committee met.".into(),
            target: "Комитет собрался.".into(),
        }];

        let prompt = build_prompt(PromptTemplate::Gemma, "They voted.", &history, "Russian");

        // Gemma has no system role, so the instruction rides with the first
        // turn - and only the first.
        assert_eq!(prompt.matches("into Russian").count(), 1);
        assert!(prompt.starts_with("<start_of_turn>user\nYou are translating"));
        assert!(prompt.contains("<start_of_turn>model\nКомитет собрался.<end_of_turn>"));
        assert!(prompt.ends_with("They voted.<end_of_turn>\n<start_of_turn>model\n"));
    }

    #[test]
    fn a_template_is_recognised_by_name() {
        assert_eq!(PromptTemplate::parse("gemma"), Some(PromptTemplate::Gemma));
        assert_eq!(
            PromptTemplate::parse("chatml"),
            Some(PromptTemplate::ChatMl)
        );
        assert_eq!(PromptTemplate::parse("llama"), None);
    }

    #[test]
    fn a_gemma_answer_stops_at_its_own_marker() {
        let cleaned = clean_output(
            "Добрый вечер.<end_of_turn>\n<start_of_turn>",
            PromptTemplate::Gemma,
        );
        assert_eq!(cleaned, "Добрый вечер.");
    }

    #[test]
    fn prompt_names_the_target_language() {
        let prompt = build_prompt(PromptTemplate::ChatMl, "Good evening.", &[], "Russian");

        assert!(prompt.contains("into Russian"));
        assert!(prompt.contains("Good evening."));
        assert!(prompt.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn history_becomes_conversation_turns() {
        let history = vec![Pair {
            source: "The committee met on Monday.".into(),
            target: "Комитет собрался в понедельник.".into(),
        }];

        let prompt = build_prompt(PromptTemplate::ChatMl, "They voted.", &history, "Russian");

        assert!(prompt.contains("Комитет собрался в понедельник."));
        // The new sentence comes last, after the history.
        assert!(prompt.rfind("They voted.").unwrap() > prompt.rfind("Комитет").unwrap());
    }

    #[test]
    fn output_stops_at_the_end_marker() {
        let cleaned = clean_output(
            "Добрый вечер.<|im_end|>\n<|im_start|>",
            PromptTemplate::ChatMl,
        );
        assert_eq!(cleaned, "Добрый вечер.");
    }

    #[test]
    fn reasoning_blocks_are_removed() {
        let cleaned = clean_output(
            "<think>The speaker is greeting the audience.</think>\n\nДобрый вечер.",
            PromptTemplate::ChatMl,
        );
        assert_eq!(cleaned, "Добрый вечер.");
    }

    #[test]
    fn an_unclosed_reasoning_block_does_not_leak() {
        let cleaned = clean_output("Добрый вечер. <think>wait, maybe", PromptTemplate::ChatMl);
        assert_eq!(cleaned, "Добрый вечер.");
    }

    #[test]
    fn wrapping_quotes_are_dropped() {
        let cleaned = clean_output("\"Добрый вечер.\"", PromptTemplate::ChatMl);
        assert_eq!(cleaned, "Добрый вечер.");
    }

    #[test]
    fn quotes_inside_the_sentence_are_kept() {
        let cleaned = clean_output("Он сказал \"да\" и ушёл.", PromptTemplate::ChatMl);
        assert_eq!(cleaned, "Он сказал \"да\" и ушёл.");
    }

    /// Feeds `pieces` through a stream the way the token loop does and returns
    /// what the caller would have shown.
    fn stream_of(pieces: &[&str]) -> (String, String) {
        let mut stream = Stream::new(PromptTemplate::ChatMl);
        let mut shown = String::new();
        for piece in pieces {
            if let Some(delta) = stream.push(piece) {
                shown.push_str(&delta);
            }
        }
        (shown, stream.finish())
    }

    #[test]
    fn a_stream_releases_whole_words() {
        let (shown, _) = stream_of(&["Доб", "рый", " веч", "ер"]);

        // "Добрый " is complete and safe; "вечер" has no space after it yet and
        // could still be "вечера".
        assert_eq!(shown, "Добрый ");
    }

    #[test]
    fn what_a_stream_shows_is_a_prefix_of_what_it_returns() {
        let pieces = [
            "Офици",
            "альные",
            " лица",
            " ожидают",
            ",",
            " что",
            " движение",
            " восстановят",
            " к",
            " пятнице",
            ".",
            "<|im_end|>",
        ];
        let (shown, finished) = stream_of(&pieces);

        assert!(!shown.is_empty());
        assert!(
            finished.starts_with(shown.trim_end()),
            "shown {shown:?} is not a prefix of {finished:?}"
        );
        assert_eq!(
            finished,
            "Официальные лица ожидают, что движение восстановят к пятнице."
        );
    }

    #[test]
    fn a_marker_is_never_shown_half_written() {
        // The stop marker arrives in pieces like any other text.
        let (shown, finished) = stream_of(&["Готово. ", "<|", "im_", "end", "|>"]);

        assert_eq!(shown, "Готово. ");
        assert_eq!(finished, "Готово.");
    }

    #[test]
    fn a_reasoning_block_is_never_shown() {
        let (shown, finished) = stream_of(&[
            "<think>",
            "The speaker ",
            "is greeting ",
            "the audience.",
            "</think>",
            "Добрый ",
            "вечер. ",
        ]);

        assert!(!shown.contains("speaker"));
        assert!(!shown.contains("think"));
        assert_eq!(shown.trim(), "Добрый вечер.");
        assert_eq!(finished, "Добрый вечер.");
    }

    #[test]
    fn a_leading_quote_is_not_shown_and_then_taken_away() {
        let (shown, finished) = stream_of(&["\"Добрый ", "вечер.\""]);

        assert!(!shown.starts_with('"'));
        assert_eq!(finished, "Добрый вечер.");
    }

    #[test]
    fn multibyte_characters_survive_being_split_across_tokens() {
        // What the token loop hands the stream is already valid UTF-8; this is
        // the guard that nothing downstream re-splits it.
        let (shown, finished) = stream_of(&["Здра", "вствуйте", ", ", "коллеги", "."]);

        assert!(!shown.contains('\u{fffd}'));
        assert_eq!(finished, "Здравствуйте, коллеги.");
    }

    #[test]
    fn nothing_is_released_before_the_first_word_ends() {
        let mut stream = Stream::new(PromptTemplate::ChatMl);
        assert_eq!(stream.push("Доб"), None);
        assert_eq!(stream.push("рый"), None);
        assert_eq!(stream.push(" "), Some("Добрый ".to_string()));
    }

    #[test]
    fn leading_whitespace_does_not_shift_the_text() {
        let (shown, finished) = stream_of(&["\n\n", "Добрый ", "вечер. "]);

        assert!(!shown.starts_with(char::is_whitespace));
        assert_eq!(finished, "Добрый вечер.");
    }

    #[test]
    fn a_prompt_that_extends_the_last_one_reuses_all_of_it() {
        let previous: Vec<LlamaToken> = [1, 2, 3, 4].into_iter().map(LlamaToken).collect();
        let next: Vec<LlamaToken> = [1, 2, 3, 4, 5, 6].into_iter().map(LlamaToken).collect();

        assert_eq!(common_prefix_len(&previous, &next), 4);
    }

    #[test]
    fn a_prompt_that_dropped_its_oldest_turn_reuses_only_the_instruction() {
        // What a sliding history window does to the cache: everything after the
        // system prompt moves.
        let previous: Vec<LlamaToken> = [1, 2, 30, 31, 40].into_iter().map(LlamaToken).collect();
        let next: Vec<LlamaToken> = [1, 2, 40, 41, 50].into_iter().map(LlamaToken).collect();

        assert_eq!(common_prefix_len(&previous, &next), 2);
    }

    #[test]
    fn nothing_is_reused_on_the_first_request() {
        let next: Vec<LlamaToken> = [1, 2, 3].into_iter().map(LlamaToken).collect();

        assert_eq!(common_prefix_len(&[], &next), 0);
    }

    #[test]
    fn a_tail_that_cannot_become_a_marker_is_not_held_back() {
        assert_eq!(
            hold_back_partial_marker("Добрый вечер.", markers(PromptTemplate::ChatMl)),
            "Добрый вечер."
        );
        assert_eq!(
            hold_back_partial_marker("Добрый <", markers(PromptTemplate::ChatMl)),
            "Добрый "
        );
    }
}
