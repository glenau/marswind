//! Translating committed speech.
//!
//! Recognition hands over a segment as soon as enough words are frozen, rather
//! than waiting for the row they belong to to close; this turns each segment
//! into the target language and emits it. Engines sit behind one enum, so a
//! lighter one can be added without the rest of the pipeline noticing.

pub mod language;
pub mod sidecar;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::asr::Caption;
use language::Language;

/// How much of the conversation is carried into the next prompt, in characters
/// of source and translation together.
///
/// A sliding window of the last N captions looks like the obvious way to do
/// this and quietly costs everything: drop the oldest turn and the prompt no
/// longer starts the way the last one did, so the worker cannot reuse a single
/// token of what it already read. Letting the conversation grow instead means
/// each request adds a turn to the end and the whole prefix is reused - and the
/// translator has more context, not less.
const HISTORY_BUDGET_CHARS: usize = 2400;
/// Captions kept when the budget is reached. Trimming is what forces a full
/// re-read, so it happens rarely and takes a lot off at once.
const HISTORY_KEEP: usize = 2;

const TRANSLATION_EVENT: &str = "translate://line";
/// A translation still being generated. The reader gets the first words about a
/// second before the sentence is finished, which is most of the perceived lag.
const PARTIAL_EVENT: &str = "translate://partial";

#[derive(Debug, thiserror::Error)]
pub enum TranslateError {
    #[error("translation is already running")]
    AlreadyRunning,
    #[error("translation is not running")]
    NotRunning,
    #[error("unknown language '{0}'")]
    UnknownLanguage(String),
    #[error("could not load the translation model: {0}")]
    Load(String),
    #[error("translation failed: {0}")]
    Generation(String),
    #[error("{0}")]
    Other(String),
}

impl Serialize for TranslateError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// An instruction-tuned model through llama.cpp.
    Llm,
}

/// One translated sentence, kept as context for the next.
#[derive(Debug, Clone)]
pub struct Pair {
    pub source: String,
    pub target: String,
}

/// Not `Send`: llama.cpp contexts belong to the thread that made them, and the
/// worker thread creates its translator and keeps it for its whole life.
pub trait Translator {
    fn name(&self) -> &'static str;

    /// Translates one caption. `on_delta` is called with each new piece of the
    /// answer as it is generated; the returned string is the whole translation
    /// and is what the caller should keep.
    fn translate(
        &mut self,
        source: &str,
        history: &[Pair],
        target: Language,
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<String, TranslateError>;
}

#[derive(Debug, Clone)]
pub struct TranslateConfig {
    pub engine: Engine,
    pub model_path: PathBuf,
    pub model_id: String,
    /// How this model wants its conversation laid out. It travels with the
    /// model rather than being guessed from its name.
    pub target: Language,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateState {
    pub running: bool,
    pub engine: Option<Engine>,
    pub model_id: Option<String>,
    pub target_language: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationEvent {
    /// Matches the caption row, so the UI can attach the translation to it.
    line: u64,
    /// Where in that row this piece goes. A row is translated in segments as
    /// its words are committed, and they are appended in this order.
    seq: u32,
    source: String,
    text: String,
    translation_ms: u64,
    /// How long the reader actually waited: the time to the first word shown,
    /// which is the number the streaming exists to lower.
    first_word_ms: u64,
}

/// A translation as it is being written. Carries the whole text so far rather
/// than the new piece: a dropped or reordered event then costs nothing.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PartialEvent {
    line: u64,
    seq: u32,
    text: String,
}

struct Running {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    engine: Engine,
    model_id: String,
    target_language: String,
}

#[derive(Default)]
pub struct TranslationEngine {
    running: Mutex<Option<Running>>,
}

impl TranslationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> TranslateState {
        let running = self.running.lock();
        TranslateState {
            running: running.is_some(),
            engine: running.as_ref().map(|r| r.engine),
            model_id: running.as_ref().map(|r| r.model_id.clone()),
            target_language: running.as_ref().map(|r| r.target_language.clone()),
        }
    }

    pub fn start(
        &self,
        app: AppHandle,
        captions: Receiver<Caption>,
        config: TranslateConfig,
    ) -> Result<(), TranslateError> {
        let mut running = self.running.lock();
        if running.is_some() {
            return Err(TranslateError::AlreadyRunning);
        }

        let stop = Arc::new(AtomicBool::new(false));
        let handle = spawn_worker(app, captions, config.clone(), Arc::clone(&stop))?;

        *running = Some(Running {
            stop,
            handle: Some(handle),
            engine: config.engine,
            model_id: config.model_id,
            target_language: config.target.code.to_string(),
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), TranslateError> {
        let mut guard = self.running.lock();
        let Some(mut running) = guard.take() else {
            return Err(TranslateError::NotRunning);
        };

        running.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = running.handle.take() {
            let _ = handle.join();
        }
        log::info!("translation stopped");
        Ok(())
    }
}

impl Drop for TranslationEngine {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn spawn_worker(
    app: AppHandle,
    captions: Receiver<Caption>,
    config: TranslateConfig,
    stop: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, TranslateError> {
    std::thread::Builder::new()
        .name("marswind-translate".into())
        .spawn(move || {
            if let Err(e) = run(&app, captions, &config, &stop) {
                log::error!("translation stopped with an error: {e}");
            }
        })
        .map_err(|e| TranslateError::Other(format!("could not start the translation thread: {e}")))
}

fn run(
    app: &AppHandle,
    captions: Receiver<Caption>,
    config: &TranslateConfig,
    stop: &AtomicBool,
) -> Result<(), TranslateError> {
    let loading = Instant::now();
    let mut translator: Box<dyn Translator> = match config.engine {
        Engine::Llm => Box::new(sidecar::SidecarTranslator::spawn(
            &config.model_path,
            recommended_threads(),
        )?),
    };

    log::info!(
        "translation started: engine={} model={} target={} in {:?}",
        translator.name(),
        config.model_id,
        config.target.code,
        loading.elapsed()
    );

    let mut history: Vec<Pair> = Vec::new();

    while !stop.load(Ordering::Relaxed) {
        let caption = match captions.recv_timeout(std::time::Duration::from_millis(200)) {
            Ok(caption) => caption,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };

        if caption.text.trim().is_empty() {
            continue;
        }

        let started = Instant::now();

        // The pieces go straight to the UI as they arrive. Nothing is kept from
        // them: the finished translation below is what history and the
        // transcript are built from.
        let mut shown = String::new();
        let mut first_word: Option<Instant> = None;
        let mut on_delta = |delta: &str| {
            if first_word.is_none() {
                first_word = Some(Instant::now());
            }
            shown.push_str(delta);
            let _ = app.emit(
                PARTIAL_EVENT,
                PartialEvent {
                    line: caption.line,
                    seq: caption.seq,
                    text: shown.trim_end().to_string(),
                },
            );
        };

        let translated =
            match translator.translate(&caption.text, &history, config.target, &mut on_delta) {
                Ok(text) => text,
                Err(e) => {
                    log::error!("could not translate a caption: {e}");
                    continue;
                }
            };
        let first_word_ms = first_word
            .map(|at| at.duration_since(started).as_millis() as u64)
            .unwrap_or_else(|| started.elapsed().as_millis() as u64);

        if translated.is_empty() {
            continue;
        }

        history.push(Pair {
            source: caption.text.clone(),
            target: translated.clone(),
        });
        trim_history(&mut history);

        let _ = app.emit(
            TRANSLATION_EVENT,
            TranslationEvent {
                line: caption.line,
                seq: caption.seq,
                source: caption.text,
                text: translated,
                translation_ms: started.elapsed().as_millis() as u64,
                first_word_ms,
            },
        );
    }

    Ok(())
}

/// Drops the oldest captions once the conversation is too long to keep sending.
/// Nothing happens until the budget is reached, and then most of it goes at
/// once - see `HISTORY_BUDGET_CHARS`.
fn trim_history(history: &mut Vec<Pair>) {
    let length: usize = history
        .iter()
        .map(|pair| pair.source.len() + pair.target.len())
        .sum();

    if length > HISTORY_BUDGET_CHARS && history.len() > HISTORY_KEEP {
        history.drain(..history.len() - HISTORY_KEEP);
    }
}

/// Translation shares the machine with recognition, which is already using
/// most of it.
fn recommended_threads() -> i32 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    (cores / 2).clamp(2, 6) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_is_idle_before_starting() {
        let engine = TranslationEngine::new();
        let state = engine.state();

        assert!(!state.running);
        assert!(state.model_id.is_none());
    }

    #[test]
    fn stopping_when_idle_is_an_error_not_a_panic() {
        let engine = TranslationEngine::new();
        assert!(matches!(engine.stop(), Err(TranslateError::NotRunning)));
    }

    #[test]
    fn thread_count_leaves_room_for_recognition() {
        let threads = recommended_threads();
        assert!((2..=6).contains(&threads));
    }

    fn pair(length: usize) -> Pair {
        Pair {
            source: "a".repeat(length),
            target: "b".repeat(length),
        }
    }

    #[test]
    fn a_short_conversation_is_kept_whole() {
        let mut history = vec![pair(50), pair(50), pair(50), pair(50)];

        trim_history(&mut history);

        // Nothing is dropped, so the next prompt still starts the way this one
        // did and the worker reuses all of it.
        assert_eq!(history.len(), 4);
    }

    #[test]
    fn a_conversation_past_its_budget_is_cut_back_hard() {
        let mut history: Vec<Pair> = (0..8).map(|_| pair(400)).collect();

        trim_history(&mut history);

        assert_eq!(history.len(), HISTORY_KEEP);
    }

    #[test]
    fn trimming_never_empties_the_context() {
        let mut history = vec![pair(4000)];

        trim_history(&mut history);

        assert_eq!(history.len(), 1);
    }
}
