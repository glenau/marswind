//! Streaming speech recognition on top of whisper.cpp.
//!
//! Whisper works on fixed windows, so turning it into a live captioner means
//! deciding two things continuously: when to run it, and which of its words to
//! believe. Silero VAD answers the first (run on speech, cut on silence) and
//! LocalAgreement-2 answers the second.

pub mod agreement;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, RecvTimeoutError, Sender, TrySendError};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperVadContext,
    WhisperVadContextParams, WhisperVadParams,
};

use crate::audio::TARGET_SAMPLE_RATE;
use agreement::{join, LocalAgreement, Word};

const SAMPLE_RATE_F: f64 = TARGET_SAMPLE_RATE as f64;

/// Don't bother whisper until there is at least this much audio. Whisper is
/// unreliable on very short windows - the first words of a phrase come out
/// mangled - and two consecutive runs can agree on the same mangled guess,
/// which would commit it.
const MIN_INFER_SECONDS: f64 = 1.6;
/// How much new audio to accumulate between runs. A word needs two runs to
/// agree before it is committed, so this lands directly in the caption lag -
/// but dropping below the time one run takes would just queue work.
const STEP_SECONDS: f64 = 0.5;
/// A phrase this long without a pause gets committed anyway. Whisper degrades
/// on very long windows and the user should not wait forever for a full stop.
const MAX_WINDOW_SECONDS: f64 = 20.0;
/// Trailing silence that ends a phrase.
const FLUSH_SILENCE_SECONDS: f32 = 0.7;
/// A window with less speech than this is not worth transcribing. Whisper
/// invents text when handed silence - "Thank you." and "Be a man." are its
/// favourites - so silence must never reach it.
const MIN_SPEECH_SECONDS: f32 = 0.25;
/// Audio kept when discarding a silent window, so a word starting right at the
/// boundary is not cut in half.
const SILENCE_KEEP_SECONDS: f64 = 0.5;
/// Audio kept before the last committed word when trimming. Whisper's word
/// timestamps are approximate, so this is deliberately generous:
/// re-transcribing a little audio is free, losing a word is not.
///
/// Keeping several seconds of already-captioned audio here as context for the
/// model was tried and measured clearly worse - 32.5% word error rate against
/// 10.4% on the same clip - so the margin stays small on purpose.
const TRIM_MARGIN_SECONDS: f64 = 0.3;
/// Words of already-committed text fed back to whisper as context.
const PROMPT_WORDS: usize = 32;
/// Committed words kept for spotting a re-read. The trim leaves a fraction of a
/// second of captioned audio in the window, which is a word or two.
const RECENT_WORDS: usize = 6;
/// How far past the captioned audio a word may end and still count as a re-read
/// of it. Whisper's word times move by around a tenth of a second between
/// windows, and the word straddling the boundary is the one that matters most.
const RE_READ_SLACK_SECONDS: f64 = 0.5;
/// Shortest line that may be closed by a full stop. Without this, "Mr." or a
/// one-word answer would each become their own caption.
const MIN_WORDS_PER_LINE: usize = 4;
/// Shortest row that may be closed at a comma.
const MIN_CLAUSE_WORDS: usize = 8;
/// Longest row allowed without any boundary at all. A row is what the reader
/// reads, so this is a readability limit, not a latency one - the translator
/// stopped waiting for rows when segments were introduced, below.
const MAX_WORDS_PER_LINE: usize = 16;

// A row of the transcript and a unit of translation used to be the same thing,
// and that is what made the translation lag a whole line behind the speaker: a
// row only closes at a boundary, so committed words - words LocalAgreement has
// already frozen and shown in full brightness - sat untranslated waiting for the
// rest of a sentence they were never going to change with.
//
// They are separate now. Committed words go to the translator in **segments** as
// soon as there are enough of them to translate, while the row they belong to
// keeps growing on screen. Segments are smaller than rows and cut on the same
// kinds of boundary, so what the reader gets is still one row of source beside
// one row of target, assembled from two or three pieces that each started as
// early as they could.
//
/// Shortest segment that may be closed by a full stop.
const MIN_SEGMENT_SENTENCE_WORDS: usize = 4;
/// Shortest segment that may be closed at a comma.
const MIN_SEGMENT_CLAUSE_WORDS: usize = 5;
/// Longest segment with no boundary in it at all.
///
/// The floor here is not quality, it is arithmetic: every segment is a request
/// to a language model sharing a GPU with recognition, and segments this size
/// arrive about every two seconds of speech. Smaller ones translate worse -
/// there is not enough of a clause to place the words - and stop being free.
const MAX_SEGMENT_WORDS: usize = 8;
/// Words at the end of a hypothesis that are never committed, however often
/// whisper repeats them.
///
/// Whisper completes what it thinks it heard, so a window that ends mid-phrase
/// gets an invented ending - and because the next window starts with the same
/// audio, it often invents the same ending again. Two runs agreeing is not
/// evidence when both are guessing at the same cut-off point, so the tail stays
/// provisional until real audio pushes it inward.
const UNSTABLE_TAIL_WORDS: usize = 2;

const TRANSCRIPT_EVENT: &str = "asr://transcript";
/// A segment that never reached the translator because it was too far behind.
const SKIPPED_EVENT: &str = "translate://skipped";

/// Segments a downstream stage may fall behind by.
///
/// Translation is slower than recognition, so this is where a backlog builds,
/// and segments arrive at roughly twice the rate rows used to. Overflowing this
/// is not silent: the segment is reported skipped so the row it belongs to
/// stops waiting for a translation that is never coming.
const SUBSCRIBER_CAPACITY: usize = 64;

#[derive(Debug, thiserror::Error)]
pub enum AsrError {
    #[error("recognition is already running")]
    AlreadyRunning,
    #[error("recognition is not running")]
    NotRunning,
    #[error("audio capture must be started before recognition")]
    CaptureNotRunning,
    #[error("whisper failed: {0}")]
    Whisper(String),
    #[error("{0}")]
    Other(String),
}

impl Serialize for AsrError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<whisper_rs::WhisperError> for AsrError {
    fn from(value: whisper_rs::WhisperError) -> Self {
        Self::Whisper(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct AsrConfig {
    pub model_path: PathBuf,
    pub vad_model_path: PathBuf,
    pub model_id: String,
    /// ISO code, or `None` to let whisper detect the language.
    pub language: Option<String>,
    /// Feed recent captions back to whisper as context. Helps with names and
    /// terminology, but a garbled caption biases the next window towards more
    /// of the same.
    pub use_prompt: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrState {
    pub running: bool,
    pub model_id: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptEvent {
    /// Identifies the phrase being built, so the UI updates a line in place
    /// instead of appending duplicates.
    line: u64,
    /// How many translation segments this row was cut into. Only meaningful on
    /// a finished row, and it is what lets the interface tell "still coming"
    /// from "never arriving".
    segments: u32,
    /// Words two consecutive runs agreed on. These never change again.
    text: String,
    /// Words seen once. Shown dimmed, and may still be revised.
    tentative: String,
    #[serde(rename = "final")]
    is_final: bool,
    inference_ms: u64,
    window_seconds: f32,
}

struct Running {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    model_id: String,
    language: Option<String>,
}

/// A piece of committed text, ready for translation.
///
/// Several of these make up one row of the transcript: `line` says which row it
/// belongs to and `seq` where in that row it goes, so the translations can be
/// appended in order as they come back.
#[derive(Debug, Clone)]
pub struct Caption {
    pub line: u64,
    pub seq: u32,
    pub text: String,
}

#[derive(Default)]
pub struct AsrEngine {
    running: Mutex<Option<Running>>,
    subscribers: Arc<Mutex<Vec<Sender<Caption>>>>,
}

impl AsrEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> AsrState {
        let running = self.running.lock();
        AsrState {
            running: running.is_some(),
            model_id: running.as_ref().map(|r| r.model_id.clone()),
            language: running.as_ref().and_then(|r| r.language.clone()),
        }
    }

    /// Receives every finished caption from now on. Dropping the receiver
    /// unsubscribes.
    pub fn subscribe(&self) -> Receiver<Caption> {
        let (sender, receiver) = bounded(SUBSCRIBER_CAPACITY);
        self.subscribers.lock().push(sender);
        receiver
    }

    pub fn start(
        &self,
        app: AppHandle,
        audio: Receiver<Arc<[f32]>>,
        config: AsrConfig,
    ) -> Result<(), AsrError> {
        let mut running = self.running.lock();
        if running.is_some() {
            return Err(AsrError::AlreadyRunning);
        }

        let stop = Arc::new(AtomicBool::new(false));
        let handle = spawn_worker(
            app,
            audio,
            config.clone(),
            Arc::clone(&self.subscribers),
            Arc::clone(&stop),
        )?;

        *running = Some(Running {
            stop,
            handle: Some(handle),
            model_id: config.model_id,
            language: config.language,
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), AsrError> {
        let mut guard = self.running.lock();
        let Some(mut running) = guard.take() else {
            return Err(AsrError::NotRunning);
        };

        running.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = running.handle.take() {
            let _ = handle.join();
        }
        log::info!("recognition stopped");
        Ok(())
    }
}

impl Drop for AsrEngine {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn spawn_worker(
    app: AppHandle,
    audio: Receiver<Arc<[f32]>>,
    config: AsrConfig,
    subscribers: Arc<Mutex<Vec<Sender<Caption>>>>,
    stop: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, AsrError> {
    std::thread::Builder::new()
        .name("marswind-asr".into())
        .spawn(move || {
            if let Err(e) = run(&app, audio, &config, &subscribers, &stop) {
                log::error!("recognition stopped with an error: {e}");
            }
        })
        .map_err(|e| AsrError::Other(format!("could not start the recognition thread: {e}")))
}

fn run(
    app: &AppHandle,
    audio: Receiver<Arc<[f32]>>,
    config: &AsrConfig,
    subscribers: &Arc<Mutex<Vec<Sender<Caption>>>>,
    stop: &AtomicBool,
) -> Result<(), AsrError> {
    let loading = Instant::now();
    let context =
        WhisperContext::new_with_params(&config.model_path, WhisperContextParameters::default())?;
    let mut state = context.create_state()?;

    let mut vad_context_params = WhisperVadContextParams::new();
    vad_context_params.set_use_gpu(false);
    let mut vad =
        WhisperVadContext::new(&config.vad_model_path.to_string_lossy(), vad_context_params)?;

    log::info!(
        "recognition started with model {} in {:?}",
        config.model_id,
        loading.elapsed()
    );

    let threads = recommended_threads();
    let min_samples = (MIN_INFER_SECONDS * SAMPLE_RATE_F) as usize;
    let step_samples = (STEP_SECONDS * SAMPLE_RATE_F) as usize;
    let max_samples = (MAX_WINDOW_SECONDS * SAMPLE_RATE_F) as usize;

    let mut window: Vec<f32> = Vec::with_capacity(max_samples + step_samples);
    let mut agreement = LocalAgreement::new();
    let mut row = Row::default();
    let mut history: Vec<String> = Vec::new();
    let mut fresh_samples = 0usize;
    // How much of the window, from its start, has already become committed
    // words. Trimming deliberately leaves a margin of captioned audio in place,
    // so this says both that the next hypothesis will re-read something and how
    // much of the window may be discarded without losing anything.
    let mut captioned_until = 0.0f64;
    // The last few committed words, kept past the caption line they went into,
    // so a re-read can still be recognized after the line has closed.
    let mut recent: Vec<Word> = Vec::new();

    while !stop.load(Ordering::Relaxed) {
        match audio.recv_timeout(Duration::from_millis(100)) {
            Ok(block) => {
                window.extend_from_slice(&block);
                fresh_samples += block.len();
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        while let Ok(block) = audio.try_recv() {
            window.extend_from_slice(&block);
            fresh_samples += block.len();
        }

        let overlong = window.len() >= max_samples;
        if window.len() < min_samples || (fresh_samples < step_samples && !overlong) {
            continue;
        }
        fresh_samples = 0;

        // Ask the VAD before whisper: it is a thousand times cheaper and it
        // keeps silence - where whisper hallucinates - out of the model.
        let speech = detect_speech(&mut vad, &window)?;

        // Silence with nothing part-said behind it: drop all but the tail of the
        // window and wait. Silence with a row still open has to go to whisper -
        // that is how the row gets finished.
        if speech.total_seconds < MIN_SPEECH_SECONDS
            && row.words.is_empty()
            && agreement.tentative().is_empty()
        {
            let keep = (SILENCE_KEEP_SECONDS * SAMPLE_RATE_F) as usize;
            if window.len() > keep {
                window.drain(..window.len() - keep);
            }
            continue;
        }

        let prompt = if config.use_prompt {
            prompt_from(&history, &row.words)
        } else {
            String::new()
        };

        let started = Instant::now();
        let hypothesis = transcribe(
            &mut state,
            &window,
            config.language.as_deref(),
            &prompt,
            threads,
        )?;
        let inference_ms = started.elapsed().as_millis() as u64;

        // The head of the window is audio that has already been captioned, kept
        // only so a word straddling the boundary is not cut in half. Whisper
        // reads it again, and those words have to come off before the
        // hypothesis is compared with anything: LocalAgreement lines the tail of
        // the last hypothesis up against the head of this one, and a re-read
        // word at the front pushes the two out of step - which is how a word
        // ends up committed twice and the caption reads "Good Good evening".
        let hypothesis = drop_repeat_of(&recent, hypothesis, captioned_until);

        // A window that loops the same phrase back at us is whisper failing,
        // not the speaker repeating themselves. Feeding it into the committed
        // text would poison the prompt and make the next window worse.
        if looks_degenerate(&hypothesis) {
            log::warn!(
                "discarding a degenerate hypothesis of {} words over {:.1}s",
                hypothesis.len(),
                seconds(window.len())
            );
            // Only the audio that has already been captioned may go. Clearing
            // the whole window here used to take the words nobody had read yet
            // with it, and a sentence would simply vanish from the transcript.
            let captioned = (captioned_until * SAMPLE_RATE_F) as usize;
            window.drain(..captioned.min(window.len()));
            captioned_until = 0.0;
            agreement.reset();
            continue;
        }

        let stable_len = hypothesis.len().saturating_sub(UNSTABLE_TAIL_WORDS);
        let (stable, tail) = hypothesis.split_at(stable_len);

        let newly = agreement.advance(stable);
        row.words.extend(newly.iter().cloned());
        if let Some(last) = newly.last() {
            captioned_until = captioned_until.max(last.end);
        }
        remember(&mut recent, &newly);

        let mut provisional = agreement.tentative().to_vec();
        provisional.extend_from_slice(tail);

        // A pause ends the phrase whether or not anything is waiting to be
        // shown. The window must be cleared either way: audio that has already
        // been captioned and then goes quiet would otherwise sit there and be
        // transcribed a second time when speech resumes.
        let phrase_ended = speech.trailing_silence >= FLUSH_SILENCE_SECONDS;

        log::debug!(
            "window={:.1}s speech={:.1}s hypothesis={} committed={} tentative={} silence={:.2}s inference={inference_ms}ms",
            seconds(window.len()),
            speech.total_seconds,
            hypothesis.len(),
            row.words.len(),
            agreement.tentative().len(),
            speech.trailing_silence
        );

        if phrase_ended || overlong {
            // Nothing more is coming for this phrase, so the tail that was held
            // back is as good as it will get.
            let flushed = agreement.flush();
            remember(&mut recent, &flushed);
            remember(&mut recent, tail);
            row.words.extend(flushed);
            row.words.extend_from_slice(tail);
            close_rows(
                app,
                subscribers,
                &mut row,
                &mut history,
                inference_ms,
                seconds(window.len()),
                true,
            );

            agreement.reset();
            window.clear();
            captioned_until = 0.0;
            continue;
        }

        // Committed audio must leave the window before the next run, or whisper
        // transcribes it again and LocalAgreement, which compares the tail of
        // the last hypothesis against the head of the new one, finds no overlap
        // and commits the same words twice.
        //
        // Only words committed in this round may drive the trim: their
        // timestamps come from the window as it is right now. Earlier words
        // carry timestamps of windows that no longer exist.
        let trim_to = newly.last().map(|word| word.end);

        // Rows that have reached a boundary are closed, and then everything
        // committed since the last segment goes to the translator whether the
        // row it belongs to is finished or not.
        close_rows(
            app,
            subscribers,
            &mut row,
            &mut history,
            inference_ms,
            seconds(window.len()),
            false,
        );
        let committed = row.words.len();
        send_segments(
            subscribers,
            &mut row,
            committed,
            false,
            &mut skip_reporter(app),
        );

        emit(
            app,
            TranscriptEvent {
                line: row.id,
                segments: row.seq,
                text: join(&row.words),
                tentative: join(&provisional),
                is_final: false,
                inference_ms,
                window_seconds: seconds(window.len()),
            },
        );

        if let Some(boundary) = trim_to {
            let cut = ((boundary - TRIM_MARGIN_SECONDS).max(0.0) * SAMPLE_RATE_F) as usize;
            // Never cut the whole window: a badly placed timestamp would throw
            // away audio nobody has read yet.
            let cut = cut.min(window.len() * 4 / 5);
            if cut > 0 {
                window.drain(..cut);
                captioned_until = (captioned_until - cut as f64 / SAMPLE_RATE_F).max(0.0);
            }
        }
    }

    Ok(())
}

/// The row of the transcript currently being built.
#[derive(Default)]
struct Row {
    /// Committed words of this row. They never change; the row is open only in
    /// the sense that more may be added.
    words: Vec<Word>,
    /// How many of `words` have already gone to the translator.
    sent: usize,
    /// Which segment of this row comes next.
    seq: u32,
    /// Matches the `line` of the transcript events, so the UI can attach
    /// translations to the right row.
    id: u64,
}

/// Hands the translator every segment of `row.words[..limit]` that is ready.
///
/// This is what stops the translation lagging a row behind: it runs on every
/// pass, on committed text, without waiting for the row to be finished. With
/// `force` the whole range goes, however short the last piece is - used when the
/// row is closing and nothing more is coming.
fn send_segments(
    subscribers: &Arc<Mutex<Vec<Sender<Caption>>>>,
    row: &mut Row,
    limit: usize,
    force: bool,
    on_skip: &mut dyn FnMut(u64, u32),
) {
    while row.sent < limit {
        let available = &row.words[row.sent..limit];
        let end = match segment_break(available) {
            Some(end) => end,
            None if force => available.len() - 1,
            None => break,
        };

        let text = join(&available[..=end]);
        row.sent += end + 1;
        if text.is_empty() {
            continue;
        }

        let delivered = broadcast(
            subscribers,
            Caption {
                line: row.id,
                seq: row.seq,
                text,
            },
        );
        if !delivered {
            on_skip(row.id, row.seq);
        }
        row.seq += 1;
    }
}

/// Closes every row that has reached a boundary and starts the next one. With
/// `force`, whatever is left becomes a row of its own - a pause has ended the
/// phrase, so there is nothing left to wait for.
fn close_rows(
    app: &AppHandle,
    subscribers: &Arc<Mutex<Vec<Sender<Caption>>>>,
    row: &mut Row,
    history: &mut Vec<String>,
    inference_ms: u64,
    window_seconds: f32,
    force: bool,
) {
    while let Some(end) = next_break(&row.words) {
        // On a forced close the whole phrase is already in hand, so a break
        // that would leave two words behind is not worth taking - they would go
        // out as a row of their own.
        if force && (1..MIN_WORDS_PER_LINE).contains(&(row.words.len() - end - 1)) {
            break;
        }
        finish_row(
            app,
            subscribers,
            row,
            end + 1,
            history,
            inference_ms,
            window_seconds,
        );
    }

    if force && !row.words.is_empty() {
        let all = row.words.len();
        finish_row(
            app,
            subscribers,
            row,
            all,
            history,
            inference_ms,
            window_seconds,
        );
    }
}

/// Emits the first `count` words of `row` as a finished row and moves on to the
/// next one. Anything in them the translator has not seen goes first, so closing
/// a row never loses a segment.
#[allow(clippy::too_many_arguments)]
fn finish_row(
    app: &AppHandle,
    subscribers: &Arc<Mutex<Vec<Sender<Caption>>>>,
    row: &mut Row,
    count: usize,
    history: &mut Vec<String>,
    inference_ms: u64,
    window_seconds: f32,
) {
    send_segments(subscribers, row, count, true, &mut skip_reporter(app));

    let segments = row.seq;
    let text = join(&row.words[..count]);
    row.words.drain(..count);
    row.sent = row.sent.saturating_sub(count);
    row.seq = 0;

    if text.is_empty() {
        return;
    }

    emit(
        app,
        TranscriptEvent {
            line: row.id,
            segments,
            text: text.clone(),
            tentative: String::new(),
            is_final: true,
            inference_ms,
            window_seconds,
        },
    );
    history.push(text);
    row.id += 1;
}

/// Index of the last word of the next translation segment, if enough has been
/// committed to be worth translating.
fn segment_break(words: &[Word]) -> Option<usize> {
    boundary_of(
        words,
        MIN_SEGMENT_SENTENCE_WORDS,
        MIN_SEGMENT_CLAUSE_WORDS,
        MAX_SEGMENT_WORDS,
    )
}

/// Index of the last word of the next caption, if the committed text has
/// reached somewhere it can be cut.
///
/// A full stop is the best place and a comma is the next best; the word count
/// is a valve for speech that has neither. Whichever comes first wins, because
/// the point is to hand the translator something as soon as there is something
/// worth handing over.
fn next_break(words: &[Word]) -> Option<usize> {
    boundary_of(
        words,
        MIN_WORDS_PER_LINE,
        MIN_CLAUSE_WORDS,
        MAX_WORDS_PER_LINE,
    )
}

/// Where a run of committed words may be cut, given how long each kind of
/// boundary has to wait. Rows and segments differ only in these numbers.
fn boundary_of(
    words: &[Word],
    min_sentence: usize,
    min_clause: usize,
    max_words: usize,
) -> Option<usize> {
    let sentence = boundary_at_or_after(words, min_sentence, |word| {
        ends_sentence(std::slice::from_ref(word))
    });
    let clause = boundary_at_or_after(words, min_clause, |word| ends_clause(&word.text));

    match (sentence, clause) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (found, other) => found.or(other),
    }
    .or_else(|| (words.len() >= max_words).then_some(max_words - 1))
}

/// The first word past a minimum length that satisfies `is_boundary`.
fn boundary_at_or_after(
    words: &[Word],
    min_words: usize,
    is_boundary: impl Fn(&Word) -> bool,
) -> Option<usize> {
    words
        .iter()
        .enumerate()
        .skip(min_words.saturating_sub(1))
        .find(|(_, word)| is_boundary(word))
        .map(|(index, _)| index)
}

/// Whether a word closes a clause. A dash is left out deliberately: whisper
/// uses it for hesitation as often as for punctuation, and cutting on it splits
/// mid-thought.
fn ends_clause(text: &str) -> bool {
    text.trim_end().ends_with([',', ';', ':'])
}

/// Removes the leading words of a hypothesis that repeat the end of what has
/// already been captioned.
///
/// The longest overlap wins, so a whole re-read clause comes off in one piece
/// rather than leaving its tail behind. Timestamps look like the obvious way to
/// do this and are not: whisper's word times shift by a tenth of a second
/// between windows, which is the width of a short word, so cutting on them
/// takes real words with it - measured, and it cost "by **a** failure".
///
/// Matching text alone is not enough to call a word a re-read - a speaker may
/// genuinely repeat themselves - so the words must also sit inside the audio
/// that was captioned before, `captioned_until` seconds of it, with slack for
/// the timestamp drift that makes timing useless on its own.
fn drop_repeat_of(recent: &[Word], hypothesis: Vec<Word>, captioned_until: f64) -> Vec<Word> {
    if captioned_until <= 0.0 || recent.is_empty() {
        return hypothesis;
    }

    let limit = captioned_until + RE_READ_SLACK_SECONDS;
    let longest = recent.len().min(hypothesis.len());
    for overlap in (1..=longest).rev() {
        if hypothesis[overlap - 1].end > limit {
            continue;
        }
        let tail = &recent[recent.len() - overlap..];
        let head = &hypothesis[..overlap];
        if tail
            .iter()
            .zip(head)
            .all(|(a, b)| same_word(&a.text, &b.text))
        {
            return hypothesis[overlap..].to_vec();
        }
    }

    hypothesis
}

/// Keeps the tail of what has been captioned, bounded.
fn remember(recent: &mut Vec<Word>, words: &[Word]) {
    recent.extend_from_slice(words);
    if recent.len() > RECENT_WORDS {
        recent.drain(..recent.len() - RECENT_WORDS);
    }
}

/// Two renderings of the same spoken word. Whisper moves punctuation and
/// capitalization around between windows even when it heard the same thing.
fn same_word(a: &str, b: &str) -> bool {
    let letters = |text: &str| -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    };
    let (a, b) = (letters(a), letters(b));
    !a.is_empty() && a == b
}

/// Hands a segment to every downstream stage. A stage that has fallen behind
/// loses the segment rather than stalling recognition - stale subtitles are
/// worse than missing ones.
///
/// Returns false if any subscriber had to drop it, so the caller can say so.
/// Dropping quietly is what left rows on screen waiting forever for a
/// translation that was never sent.
fn broadcast(subscribers: &Arc<Mutex<Vec<Sender<Caption>>>>, caption: Caption) -> bool {
    let mut delivered = true;
    let mut subscribers = subscribers.lock();
    subscribers.retain(|sender| match sender.try_send(caption.clone()) {
        Ok(()) => true,
        Err(TrySendError::Full(_)) => {
            log::warn!(
                "translation is behind; skipped segment {}.{}",
                caption.line,
                caption.seq
            );
            delivered = false;
            true
        }
        Err(TrySendError::Disconnected(_)) => false,
    });
    delivered
}

/// Reports a skipped segment to the interface.
fn skip_reporter(app: &AppHandle) -> impl FnMut(u64, u32) + '_ {
    |line, seq| {
        let _ = app.emit(SKIPPED_EVENT, SkippedEvent { line, seq });
    }
}

/// Tells the interface a segment is not coming, so the row it belongs to can
/// settle instead of showing "translating…" for the rest of the session.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkippedEvent {
    line: u64,
    seq: u32,
}

/// Titles and abbreviations that end in a period without ending a sentence.
const ABBREVIATIONS: [&str; 12] = [
    "mr", "mrs", "ms", "dr", "prof", "st", "vs", "etc", "inc", "ltd", "jr", "sr",
];

/// Whether the committed text has reached a full stop.
fn ends_sentence(words: &[Word]) -> bool {
    let Some(last) = words.last() else {
        return false;
    };
    let text = last.text.trim_end();

    if text.ends_with(['!', '?', '…']) {
        return true;
    }
    if !text.ends_with('.') {
        return false;
    }

    let core = text.trim_end_matches('.');
    let letters: String = core
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();

    if ABBREVIATIONS.contains(&letters.as_str()) {
        return false;
    }

    // Dotted initialisms such as "U.S." or "e.g." - a period inside a very
    // short word means the word is not over.
    !(core.contains('.') && letters.len() <= 4)
}

fn transcribe(
    state: &mut whisper_rs::WhisperState,
    samples: &[f32],
    language: Option<&str>,
    prompt: &str,
    threads: i32,
) -> Result<Vec<Word>, AsrError> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(threads);
    params.set_language(language);
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_token_timestamps(true);
    params.set_suppress_blank(true);
    // Each run re-reads the whole window, so whisper's own carry-over context
    // would be duplicated work; the prompt below does that job explicitly.
    params.set_no_context(true);
    // Temperature fallback re-decodes a window several times. For live captions
    // a single fast pass beats a slow perfect one.
    params.set_temperature_inc(0.0);
    if !prompt.is_empty() {
        params.set_initial_prompt(prompt);
    }

    state.full(params, samples)?;

    let mut tokens = Vec::new();
    for index in 0..state.full_n_segments() {
        let Some(segment) = state.get_segment(index) else {
            continue;
        };
        for token_index in 0..segment.n_tokens() {
            let Some(token) = segment.get_token(token_index) else {
                continue;
            };
            let Ok(text) = token.to_str_lossy() else {
                continue;
            };
            if is_special(&text) {
                continue;
            }
            let data = token.token_data();
            // Token timestamps are in centiseconds from the window start.
            tokens.push((text.to_string(), data.t1 as f64 / 100.0));
        }
    }

    Ok(words_from_tokens(&tokens))
}

/// Whisper emits subword pieces: "seaboard" arrives as " se", "ab", "oard".
/// A leading space marks the start of a new word, which is what lets us glue
/// them back together - and word-level units are also what LocalAgreement
/// should compare, since half a word is never worth showing.
fn words_from_tokens(tokens: &[(String, f64)]) -> Vec<Word> {
    let mut words: Vec<Word> = Vec::new();

    for (text, end) in tokens {
        if text.is_empty() {
            continue;
        }

        if text.starts_with(' ') || words.is_empty() {
            words.push(Word {
                text: text.trim_start().to_string(),
                end: *end,
            });
        } else if let Some(last) = words.last_mut() {
            last.text.push_str(text);
            last.end = *end;
        }
    }

    words.retain(|word| !word.text.trim().is_empty());
    words
}

struct Speech {
    /// Total speech in the window.
    total_seconds: f32,
    /// Silence between the last speech and the end of the window.
    trailing_silence: f32,
}

fn detect_speech(vad: &mut WhisperVadContext, window: &[f32]) -> Result<Speech, AsrError> {
    let mut params = WhisperVadParams::new();
    params.set_min_silence_duration(100);
    params.set_speech_pad(0);

    let segments = vad.segments_from_samples(params, window)?;

    let mut total = 0.0f32;
    let mut last_end = 0.0f32;
    for index in 0..segments.num_segments() {
        // Segment timestamps are centiseconds.
        let start = segments.get_segment_start_timestamp(index).unwrap_or(0.0) / 100.0;
        let end = segments.get_segment_end_timestamp(index).unwrap_or(0.0) / 100.0;
        total += (end - start).max(0.0);
        last_end = last_end.max(end);
    }

    Ok(Speech {
        total_seconds: total,
        trailing_silence: (seconds(window.len()) - last_end).max(0.0),
    })
}

/// Detects whisper's repetition failure, where it loops a phrase until it runs
/// out of tokens.
///
/// The loop always sits at the end of the hypothesis, so this looks for a block
/// of any length that the tail repeats: "and he could get out of the ground"
/// three times over is the model stuck, not the speaker.
fn looks_degenerate(words: &[Word]) -> bool {
    const MIN_REPEATS: usize = 3;
    const MAX_PERIOD: usize = 12;
    /// A single word said over and over is common enough in speech ("no no
    /// no") that it takes more evidence.
    const MIN_SINGLE_WORD_REPEATS: usize = 5;

    let normalized: Vec<String> = words.iter().map(|word| word.text.to_lowercase()).collect();
    let length = normalized.len();

    for period in 1..=MAX_PERIOD.min(length / 2) {
        let needed = if period == 1 {
            MIN_SINGLE_WORD_REPEATS
        } else {
            MIN_REPEATS
        };

        let mut repeats = 1;
        let mut end = length;
        while end >= 2 * period
            && normalized[end - period..end] == normalized[end - 2 * period..end - period]
        {
            repeats += 1;
            end -= period;
            if repeats >= needed {
                return true;
            }
        }
    }

    false
}

/// Recent text handed back to whisper so it keeps names and terminology
/// consistent across windows.
fn prompt_from(history: &[String], committed: &[Word]) -> String {
    let mut words: Vec<&str> = Vec::new();
    for line in history.iter().rev().take(3) {
        words.extend(line.split_whitespace());
    }
    let mut text: Vec<&str> = words.into_iter().collect();
    text.extend(committed.iter().map(|w| w.text.trim()));

    let start = text.len().saturating_sub(PROMPT_WORDS);
    text[start..].join(" ").trim().to_string()
}

fn is_special(text: &str) -> bool {
    text.starts_with("[_") || text.starts_with("<|") || text.starts_with("[BLANK")
}

fn seconds(samples: usize) -> f32 {
    samples as f32 / TARGET_SAMPLE_RATE as f32
}

/// Whisper scales poorly past a handful of threads and we share the machine
/// with the audio pipeline and the browser being transcribed.
fn recommended_threads() -> i32 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    cores.saturating_sub(2).clamp(2, 8) as i32
}

fn emit(app: &AppHandle, event: TranscriptEvent) {
    let _ = app.emit(TRANSCRIPT_EVENT, event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_tokens_are_filtered() {
        assert!(is_special("[_BEG_]"));
        assert!(is_special("<|endoftext|>"));
        assert!(!is_special("market"));
        assert!(!is_special("[laughs]"));
    }

    #[test]
    fn thread_count_leaves_room_for_the_rest_of_the_app() {
        let threads = recommended_threads();
        assert!((2..=8).contains(&threads));
    }

    #[test]
    fn prompt_uses_the_most_recent_words() {
        let history = vec!["one two three".to_string()];
        let committed = vec![
            Word {
                text: "four".into(),
                end: 1.0,
            },
            Word {
                text: "five".into(),
                end: 1.2,
            },
        ];

        let prompt = prompt_from(&history, &committed);

        assert!(prompt.ends_with("four five"));
        assert!(prompt.contains("one"));
    }

    #[test]
    fn prompt_is_bounded() {
        let history = vec![(0..500)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" ")];

        let prompt = prompt_from(&history, &[]);

        assert!(prompt.split_whitespace().count() <= PROMPT_WORDS);
    }

    #[test]
    fn empty_history_produces_no_prompt() {
        assert!(prompt_from(&[], &[]).is_empty());
    }

    #[test]
    fn seconds_conversion_matches_the_pipeline_rate() {
        assert!((seconds(TARGET_SAMPLE_RATE as usize) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn subword_tokens_are_glued_back_into_words() {
        let tokens = vec![
            (" the".to_string(), 0.30),
            (" eastern".to_string(), 0.60),
            (" se".to_string(), 0.80),
            ("ab".to_string(), 0.90),
            ("oard".to_string(), 1.05),
        ];

        let words = words_from_tokens(&tokens);

        assert_eq!(words.len(), 3);
        assert_eq!(words[2].text, "seaboard");
        // A word ends when its last piece ends.
        assert!((words[2].end - 1.05).abs() < 1e-9);
    }

    #[test]
    fn punctuation_attaches_to_the_word_before_it() {
        let tokens = vec![
            (" evening".to_string(), 0.4),
            (",".to_string(), 0.4),
            (" everyone".to_string(), 0.9),
        ];

        let words = words_from_tokens(&tokens);

        assert_eq!(words.len(), 2);
        assert_eq!(words[0].text, "evening,");
    }

    #[test]
    fn ordinary_speech_is_not_flagged_as_degenerate() {
        let words: Vec<Word> =
            "the central bank left interest rates unchanged for the third consecutive meeting"
                .split_whitespace()
                .enumerate()
                .map(|(i, text)| Word {
                    text: text.to_string(),
                    end: i as f64 * 0.3,
                })
                .collect();

        assert!(!looks_degenerate(&words));
    }

    #[test]
    fn a_looping_hypothesis_is_flagged() {
        let words: Vec<Word> = "good evening everyone "
            .repeat(6)
            .split_whitespace()
            .enumerate()
            .map(|(i, text)| Word {
                text: text.to_string(),
                end: i as f64 * 0.3,
            })
            .collect();

        assert!(looks_degenerate(&words));
    }

    fn words_of(text: &str) -> Vec<Word> {
        text.split_whitespace()
            .enumerate()
            .map(|(i, word)| Word {
                text: word.to_string(),
                end: i as f64 * 0.3,
            })
            .collect()
    }

    #[test]
    fn a_full_stop_ends_a_sentence() {
        assert!(ends_sentence(&words_of(
            "service will be restored by Friday."
        )));
        assert!(ends_sentence(&words_of("are you sure?")));
        assert!(!ends_sentence(&words_of("service will be restored by")));
    }

    #[test]
    fn abbreviations_do_not_end_a_sentence() {
        assert!(!ends_sentence(&words_of("a report from the U.S.")));
        assert!(!ends_sentence(&words_of("the meeting with Mr.")));
        assert!(!ends_sentence(&words_of("batteries, cables, etc.")));
        assert!(ends_sentence(&words_of("that is what the report says.")));
    }

    #[test]
    fn an_empty_line_never_ends_a_sentence() {
        assert!(!ends_sentence(&[]));
    }

    #[test]
    fn a_full_stop_closes_a_caption_before_a_later_comma() {
        let words = words_of("service will be restored by Friday. The officials said,");

        // The full stop at index 5 wins over the comma at index 8.
        assert_eq!(next_break(&words), Some(5));
    }

    #[test]
    fn a_comma_closes_a_caption_when_no_full_stop_has_arrived() {
        let words = words_of(
            "in economic news the central bank left interest rates unchanged, citing steady inflation",
        );

        assert_eq!(next_break(&words), Some(9));
    }

    #[test]
    fn a_short_clause_waits_for_more_words() {
        assert_eq!(next_break(&words_of("good evening,")), None);
        assert_eq!(next_break(&words_of("so, anyway, the committee met")), None);
    }

    #[test]
    fn speech_without_any_punctuation_is_cut_by_length() {
        let words = words_of(&"word ".repeat(MAX_WORDS_PER_LINE + 3));

        assert_eq!(next_break(&words), Some(MAX_WORDS_PER_LINE - 1));
    }

    #[test]
    fn a_caption_that_has_not_reached_a_boundary_is_not_closed() {
        assert_eq!(
            next_break(&words_of("the committee met on Monday to")),
            None
        );
    }

    /// Runs `words` through the segmenter the way the loop does, committing
    /// them a few at a time, and returns the segments handed to the translator.
    fn segments_of(words: &[Word], commit_batch: usize) -> Vec<String> {
        let (sender, receiver) = bounded(256);
        let subscribers = Arc::new(Mutex::new(vec![sender]));
        let mut row = Row::default();

        for batch in words.chunks(commit_batch) {
            row.words.extend_from_slice(batch);
            let committed = row.words.len();
            send_segments(&subscribers, &mut row, committed, false, &mut |_, _| {});
        }

        drop(subscribers);
        receiver.try_iter().map(|caption| caption.text).collect()
    }

    #[test]
    fn a_segment_goes_out_before_its_row_is_finished() {
        // The row this belongs to is nowhere near a boundary - 8 words, no
        // punctuation - and the translator has work anyway.
        let words = words_of("So the thing everyone keeps missing about this");

        let segments = segments_of(&words, 2);

        assert_eq!(
            segments,
            vec!["So the thing everyone keeps missing about this"]
        );
    }

    #[test]
    fn a_segment_ends_at_a_comma_rather_than_at_the_word_count() {
        let words = words_of("the central bank left rates unchanged, citing steady inflation");

        let segments = segments_of(&words, 9);

        assert_eq!(segments[0], "the central bank left rates unchanged,");
    }

    #[test]
    fn a_comma_too_early_in_a_segment_is_not_a_boundary() {
        // Three words is not enough to place a translation, so the segment runs
        // on to the length valve instead.
        let words = words_of("In economic news, the central bank left rates");

        assert_eq!(
            segments_of(&words, 8)[0],
            "In economic news, the central bank left rates"
        );
    }

    #[test]
    fn nothing_is_sent_until_there_is_enough_to_translate() {
        assert!(segments_of(&words_of("So the thing"), 1).is_empty());
    }

    #[test]
    fn committed_words_are_sent_once_and_in_order() {
        let words = words_of(
            "the committee met on Monday to review the proposal and the numbers did not add up at all and the vote was postponed again",
        );

        let segments = segments_of(&words, 3);

        // Whatever the cuts are, joining them back must give the words that went
        // in, in order and with nothing repeated or lost.
        let sent = segments.join(" ");
        assert!(join(&words).starts_with(&sent), "{sent:?}");
        assert!(segments.len() >= 3);
    }

    #[test]
    fn closing_a_row_sends_the_words_that_were_still_short_of_a_segment() {
        let (sender, receiver) = bounded(256);
        let subscribers = Arc::new(Mutex::new(vec![sender]));
        let mut row = Row {
            words: words_of("and that was all"),
            ..Default::default()
        };

        // Four words: below the six that make a segment on their own.
        let committed = row.words.len();
        send_segments(&subscribers, &mut row, committed, false, &mut |_, _| {});
        assert_eq!(receiver.try_iter().count(), 0);

        send_segments(&subscribers, &mut row, committed, true, &mut |_, _| {});
        drop(subscribers);
        let sent: Vec<String> = receiver.try_iter().map(|c| c.text).collect();
        assert_eq!(sent, vec!["and that was all"]);
    }

    #[test]
    fn a_segment_the_translator_cannot_take_is_reported_skipped() {
        // A queue of one, already full: the next segment has nowhere to go.
        let (sender, _receiver) = bounded(1);
        let subscribers = Arc::new(Mutex::new(vec![sender]));
        let mut row = Row {
            words: words_of(
                "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen",
            ),
            ..Default::default()
        };

        let mut skipped = Vec::new();
        let committed = row.words.len();
        send_segments(
            &subscribers,
            &mut row,
            committed,
            false,
            &mut |line, seq| skipped.push((line, seq)),
        );

        // The first fills the queue, the second is skipped - and says so, so the
        // row does not sit waiting for a translation nobody is writing.
        assert_eq!(skipped, vec![(0, 1)]);
    }

    #[test]
    fn segments_are_numbered_within_their_row() {
        let (sender, receiver) = bounded(256);
        let subscribers = Arc::new(Mutex::new(vec![sender]));
        let mut row = Row {
            words: words_of(
                "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen",
            ),
            ..Default::default()
        };

        let committed = row.words.len();
        send_segments(&subscribers, &mut row, committed, false, &mut |_, _| {});
        drop(subscribers);

        let seqs: Vec<u32> = receiver.try_iter().map(|c| c.seq).collect();
        assert_eq!(seqs, vec![0, 1]);
    }

    #[test]
    fn a_re_read_word_is_taken_off_the_front() {
        let recent = words_of("say the power outage that hit");
        // `words_of` puts the first word at 0.0 and each next one 0.3 later.
        let hypothesis = words_of("hit the eastern seaboard on Tuesday");

        let kept = drop_repeat_of(&recent, hypothesis, 0.3);

        assert_eq!(join(&kept), "the eastern seaboard on Tuesday");
    }

    #[test]
    fn the_longest_re_read_run_comes_off_in_one_piece() {
        let recent = words_of("caused by a failure at a single substation");
        let hypothesis = words_of("at a single substation. Officials expect service");

        let kept = drop_repeat_of(&recent, hypothesis, 0.6);

        assert_eq!(join(&kept), "Officials expect service");
    }

    #[test]
    fn punctuation_does_not_hide_a_re_read() {
        let recent = words_of("restored by Friday");
        let hypothesis = vec![
            Word {
                text: "Friday,".into(),
                end: 0.3,
            },
            Word {
                text: "morning".into(),
                end: 0.7,
            },
        ];

        assert_eq!(join(&drop_repeat_of(&recent, hypothesis, 0.3)), "morning");
    }

    #[test]
    fn nothing_comes_off_a_window_with_no_captioned_audio_in_it() {
        let recent = words_of("and the market");
        let hypothesis = words_of("the market fell again");

        // The window was cleared, so the repeat is the speaker's, not whisper's.
        assert_eq!(drop_repeat_of(&recent, hypothesis.clone(), 0.0), hypothesis);
    }

    #[test]
    fn a_repeat_too_late_in_the_window_to_be_a_re_read_is_kept() {
        let recent = words_of("we asked the finance team");
        // The same words, but spoken again well past the captioned audio.
        let hypothesis = words_of("the finance team checked the numbers");

        assert_eq!(
            join(&drop_repeat_of(&recent, hypothesis, 0.05)),
            "the finance team checked the numbers"
        );
    }

    #[test]
    fn unrelated_words_are_left_alone() {
        let recent = words_of("interest rates unchanged");
        let hypothesis = words_of("citing steady inflation");

        assert_eq!(drop_repeat_of(&recent, hypothesis.clone(), 0.5), hypothesis);
    }

    #[test]
    fn a_forced_close_does_not_leave_a_stub_caption_behind() {
        // "…until next month" - breaking at the length valve would send "month"
        // out on its own.
        let words = words_of(&"word ".repeat(MAX_WORDS_PER_LINE + 1));

        assert_eq!(next_break(&words), Some(MAX_WORDS_PER_LINE - 1));
        assert!((1..MIN_WORDS_PER_LINE).contains(&(words.len() - MAX_WORDS_PER_LINE)));
    }

    #[test]
    fn the_remembered_tail_stays_bounded() {
        let mut recent = Vec::new();
        for _ in 0..5 {
            remember(&mut recent, &words_of("one two three"));
        }

        assert_eq!(recent.len(), RECENT_WORDS);
        assert_eq!(recent.last().unwrap().text, "three");
    }

    #[test]
    fn a_short_loop_is_flagged() {
        // Seen live: whisper stuck on one clause for several seconds.
        let words = words_of(
            "he could get out of the ground and he could get out of the ground and he could get out of the ground and",
        );

        assert!(looks_degenerate(&words));
    }

    #[test]
    fn a_repeated_word_pair_is_not_enough_to_flag() {
        let words: Vec<Word> = "so so so it goes and so it goes again"
            .split_whitespace()
            .enumerate()
            .map(|(i, text)| Word {
                text: text.to_string(),
                end: i as f64 * 0.3,
            })
            .collect();

        assert!(!looks_degenerate(&words));
    }
}
