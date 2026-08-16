//! Recording a listening session so it can be read back later.
//!
//! Subtitles are gone the moment they scroll past, which is fine while you are
//! watching and useless afterwards - for checking what was said, and for
//! judging whether a change to the pipeline made anything better. A session is
//! therefore written to disk as it happens: every row with its translation, the
//! timings behind it, and which models produced it.
//!
//! Sessions live in `transcripts/` beside the models. Nothing here leaves the
//! machine, and a session is only written when the user has been listening.

pub mod recorder;

use std::fs;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// Sessions with nothing in them are not worth a file.
const MIN_ROWS: usize = 1;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("no session is being recorded")]
    NotRecording,
    #[error("no session with id '{0}'")]
    NotFound(String),
    #[error("{0}")]
    Io(String),
}

impl Serialize for HistoryError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for HistoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

/// One row of a transcript, as it ended up.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub line: u64,
    /// Seconds from the start of the session when recognition finished the row.
    pub at: f64,
    pub source: String,
    pub translation: String,
    /// How long the recognition pass that closed this row took.
    pub recognition_ms: u64,
    /// Time spent translating the row, summed over its segments.
    pub translation_ms: u64,
    /// Segments whose translation never arrived. Recorded rather than hidden:
    /// a row with a hole in it should be visibly incomplete when read back.
    pub skipped_segments: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    /// Local time the session started, as text - the file is meant to be read
    /// by a person as much as by the app.
    pub started_at: String,
    pub duration_seconds: f64,
    pub source: String,
    pub asr_model: String,
    pub spoken_language: String,
    pub mt_model: String,
    pub target_language: String,
    pub rows: Vec<Row>,
}

impl Session {
    pub fn words(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.source.split_whitespace().count())
            .sum()
    }
}

/// What the session list shows without reading every file back.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub started_at: String,
    pub duration_seconds: f64,
    pub rows: usize,
    pub words: usize,
    pub asr_model: String,
    pub target_language: String,
    pub translated: bool,
}

impl From<&Session> for SessionSummary {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.clone(),
            started_at: session.started_at.clone(),
            duration_seconds: session.duration_seconds,
            rows: session.rows.len(),
            words: session.words(),
            asr_model: session.asr_model.clone(),
            target_language: session.target_language.clone(),
            translated: session.rows.iter().any(|row| !row.translation.is_empty()),
        }
    }
}

/// The session being recorded now, if any.
#[derive(Default)]
struct Recording {
    session: Session,
    started: Option<std::time::Instant>,
}

pub struct HistoryStore {
    directory: PathBuf,
    recording: Mutex<Option<Recording>>,
}

impl HistoryStore {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            recording: Mutex::new(None),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn is_recording(&self) -> bool {
        self.recording.lock().is_some()
    }

    /// Starts a session. Anything already being recorded is finished first, so
    /// a crash-free restart never loses the previous one.
    pub fn start(&self, id: String, started_at: String, meta: Meta) -> Result<(), HistoryError> {
        let _ = self.finish();

        *self.recording.lock() = Some(Recording {
            session: Session {
                id,
                started_at,
                source: meta.source,
                asr_model: meta.asr_model,
                spoken_language: meta.spoken_language,
                mt_model: meta.mt_model,
                target_language: meta.target_language,
                ..Session::default()
            },
            started: Some(std::time::Instant::now()),
        });
        Ok(())
    }

    /// Adds or updates a row. Recognition delivers the row first and translation
    /// fills it in afterwards, so this merges rather than appends.
    pub fn record(&self, update: RowUpdate) {
        let mut guard = self.recording.lock();
        let Some(recording) = guard.as_mut() else {
            return;
        };

        let at = recording
            .started
            .map(|start| start.elapsed().as_secs_f64())
            .unwrap_or_default();

        let row = match recording
            .session
            .rows
            .iter_mut()
            .find(|row| row.line == update.line)
        {
            Some(row) => row,
            None => {
                recording.session.rows.push(Row {
                    line: update.line,
                    at,
                    ..Row::default()
                });
                recording.session.rows.last_mut().expect("just pushed")
            }
        };

        if let Some(source) = update.source {
            row.source = source;
            row.at = at;
        }
        if let Some(recognition_ms) = update.recognition_ms {
            row.recognition_ms = recognition_ms;
        }
        if let Some(translation) = update.translation {
            // Segments arrive one at a time and are appended in order.
            if !row.translation.is_empty() {
                row.translation.push(' ');
            }
            row.translation.push_str(&translation);
        }
        if let Some(translation_ms) = update.translation_ms {
            row.translation_ms += translation_ms;
        }
        if update.skipped {
            row.skipped_segments += 1;
        }
    }

    /// Ends the session and writes it out. Returns its id, or `None` if there
    /// was nothing worth keeping.
    pub fn finish(&self) -> Result<Option<String>, HistoryError> {
        let Some(mut recording) = self.recording.lock().take() else {
            return Ok(None);
        };

        recording.session.duration_seconds = recording
            .started
            .map(|start| start.elapsed().as_secs_f64())
            .unwrap_or_default();
        // Rows with nothing recognized in them are the tail of a session that
        // was stopped mid-phrase, not content.
        recording.session.rows.retain(|row| !row.source.is_empty());

        if recording.session.rows.len() < MIN_ROWS {
            return Ok(None);
        }

        let id = recording.session.id.clone();
        self.write(&recording.session)?;
        Ok(Some(id))
    }

    fn write(&self, session: &Session) -> Result<(), HistoryError> {
        fs::create_dir_all(&self.directory)?;
        let json =
            serde_json::to_string_pretty(session).map_err(|e| HistoryError::Io(e.to_string()))?;
        fs::write(self.path_of(&session.id), json)?;
        Ok(())
    }

    fn path_of(&self, id: &str) -> PathBuf {
        self.directory.join(format!("{id}.json"))
    }

    /// Newest first, which is the order anyone wants to see them in.
    ///
    pub fn list(&self) -> Result<Vec<SessionSummary>, HistoryError> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }

        let mut sessions: Vec<Session> = Vec::new();
        for entry in fs::read_dir(&self.directory)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Some(session) = read_session(&path) {
                    sessions.push(session);
                }
            }
        }

        // Ids are the epoch second the session started, zero-padded to the same
        // width, so sorting them as text sorts them by time.
        sessions.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(sessions.iter().map(SessionSummary::from).collect())
    }

    pub fn read(&self, id: &str) -> Result<Session, HistoryError> {
        read_session(&self.path_of(id)).ok_or_else(|| HistoryError::NotFound(id.to_string()))
    }

    pub fn remove(&self, id: &str) -> Result<(), HistoryError> {
        let path = self.path_of(id);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Writes a session out in `format` and returns the file it wrote.
    pub fn export(&self, id: &str, format: Format, to: &Path) -> Result<PathBuf, HistoryError> {
        let session = self.read(id)?;
        let body = match format {
            Format::Text => as_text(&session),
            Format::Srt => as_srt(&session),
            Format::Json => serde_json::to_string_pretty(&session)
                .map_err(|e| HistoryError::Io(e.to_string()))?,
        };

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(to, body)?;
        Ok(to.to_path_buf())
    }
}

/// What is known about a session when it starts.
#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub source: String,
    pub asr_model: String,
    pub spoken_language: String,
    pub mt_model: String,
    pub target_language: String,
}

/// A change to one row. Every field is optional because the two stages fill in
/// different parts of it at different times.
#[derive(Debug, Clone, Default)]
pub struct RowUpdate {
    pub line: u64,
    pub source: Option<String>,
    pub translation: Option<String>,
    pub recognition_ms: Option<u64>,
    pub translation_ms: Option<u64>,
    pub skipped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Text,
    Srt,
    Json,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Text => "txt",
            Format::Srt => "srt",
            Format::Json => "json",
        }
    }
}

fn read_session(path: &Path) -> Option<Session> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Both languages, one row per paragraph - the form that is readable without
/// the app.
fn as_text(session: &Session) -> String {
    let mut out = format!(
        "Marswind session {}\n{}  ·  {:.0} s  ·  {} rows\nRecognition: {}  Translation: {} → {}\n\n",
        session.id,
        session.started_at,
        session.duration_seconds,
        session.rows.len(),
        session.asr_model,
        session.mt_model,
        session.target_language,
    );

    for row in &session.rows {
        out.push_str(&format!("[{}]\n{}\n", timestamp(row.at), row.source));
        if !row.translation.is_empty() {
            out.push_str(&format!("{}\n", row.translation));
        }
        out.push('\n');
    }
    out
}

/// Subtitles, translation first. Each row runs until the next one starts, and
/// the last is given a few seconds of its own.
fn as_srt(session: &Session) -> String {
    const TAIL_SECONDS: f64 = 3.0;
    let mut out = String::new();

    for (index, row) in session.rows.iter().enumerate() {
        let start = row.at;
        let end = session
            .rows
            .get(index + 1)
            .map(|next| next.at)
            .unwrap_or(start + TAIL_SECONDS)
            .max(start + 0.5);

        let text = if row.translation.is_empty() {
            row.source.clone()
        } else {
            format!("{}\n{}", row.translation, row.source)
        };

        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            index + 1,
            srt_time(start),
            srt_time(end),
            text
        ));
    }
    out
}

fn timestamp(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn srt_time(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    let whole = seconds as u64;
    let millis = ((seconds - whole as f64) * 1000.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02},{:03}",
        whole / 3600,
        (whole % 3600) / 60,
        whole % 60,
        millis
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each test gets its own directory, named after itself, wiped first - the
    /// same shape the model store's tests use.
    fn store(name: &str) -> HistoryStore {
        let directory = std::env::temp_dir().join(format!("marswind-history-{name}"));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        HistoryStore::new(directory)
    }

    fn recorded(name: &str) -> HistoryStore {
        let store = store(name);
        store
            .start(
                "2026-08-02-140000".into(),
                "2 Aug 2026, 14:00".into(),
                Meta {
                    asr_model: "large-v3-turbo-q5_0".into(),
                    mt_model: "qwen3-4b-instruct-q4".into(),
                    target_language: "ru".into(),
                    ..Meta::default()
                },
            )
            .unwrap();
        store
    }

    #[test]
    fn a_row_is_filled_in_by_two_stages() {
        let store = recorded("two-stages");

        store.record(RowUpdate {
            line: 0,
            source: Some("Good evening.".into()),
            recognition_ms: Some(670),
            ..RowUpdate::default()
        });
        store.record(RowUpdate {
            line: 0,
            translation: Some("Добрый вечер.".into()),
            translation_ms: Some(900),
            ..RowUpdate::default()
        });

        store.finish().unwrap();
        let session = store.read("2026-08-02-140000").unwrap();

        assert_eq!(session.rows.len(), 1);
        assert_eq!(session.rows[0].source, "Good evening.");
        assert_eq!(session.rows[0].translation, "Добрый вечер.");
        assert_eq!(session.rows[0].recognition_ms, 670);
    }

    #[test]
    fn the_segments_of_a_row_are_joined_in_order() {
        let store = recorded("segments");

        store.record(RowUpdate {
            line: 0,
            source: Some("Good evening, federal investigators say".into()),
            ..RowUpdate::default()
        });
        store.record(RowUpdate {
            line: 0,
            translation: Some("Добрый вечер,".into()),
            translation_ms: Some(400),
            ..RowUpdate::default()
        });
        store.record(RowUpdate {
            line: 0,
            translation: Some("федеральные следователи сообщают".into()),
            translation_ms: Some(500),
            ..RowUpdate::default()
        });

        store.finish().unwrap();
        let session = store.read("2026-08-02-140000").unwrap();

        assert_eq!(
            session.rows[0].translation,
            "Добрый вечер, федеральные следователи сообщают"
        );
        assert_eq!(session.rows[0].translation_ms, 900);
    }

    #[test]
    fn a_skipped_segment_is_recorded_not_hidden() {
        let store = recorded("skipped");
        store.record(RowUpdate {
            line: 0,
            source: Some("Good evening.".into()),
            ..RowUpdate::default()
        });
        store.record(RowUpdate {
            line: 0,
            skipped: true,
            ..RowUpdate::default()
        });

        store.finish().unwrap();

        assert_eq!(
            store.read("2026-08-02-140000").unwrap().rows[0].skipped_segments,
            1
        );
    }

    #[test]
    fn a_session_with_nothing_in_it_is_not_written() {
        let store = recorded("empty");

        assert!(store.finish().unwrap().is_none());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn rows_that_never_got_any_text_are_dropped() {
        let store = recorded("no-text");
        store.record(RowUpdate {
            line: 0,
            source: Some("Good evening.".into()),
            ..RowUpdate::default()
        });
        // A translation arrived for a row recognition never finished.
        store.record(RowUpdate {
            line: 9,
            translation: Some("Что-то".into()),
            ..RowUpdate::default()
        });

        store.finish().unwrap();

        assert_eq!(store.read("2026-08-02-140000").unwrap().rows.len(), 1);
    }

    #[test]
    fn recording_nothing_at_all_is_not_an_error() {
        let store = store("idle");
        assert!(store.finish().unwrap().is_none());
        store.record(RowUpdate::default());
    }

    #[test]
    fn starting_a_session_finishes_the_one_before_it() {
        let store = recorded("restart");
        store.record(RowUpdate {
            line: 0,
            source: Some("First.".into()),
            ..RowUpdate::default()
        });

        store
            .start(
                "2026-08-02-150000".into(),
                "2 Aug 2026, 15:00".into(),
                Meta::default(),
            )
            .unwrap();
        store.record(RowUpdate {
            line: 0,
            source: Some("Second.".into()),
            ..RowUpdate::default()
        });
        store.finish().unwrap();

        let sessions = store.list().unwrap();
        assert_eq!(sessions.len(), 2);
        // Newest first.
        assert_eq!(sessions[0].id, "2026-08-02-150000");
    }

    #[test]
    fn a_session_can_be_removed() {
        let store = recorded("remove");
        store.record(RowUpdate {
            line: 0,
            source: Some("Good evening.".into()),
            ..RowUpdate::default()
        });
        store.finish().unwrap();

        store.remove("2026-08-02-140000").unwrap();

        assert!(store.list().unwrap().is_empty());
        // Removing something that is already gone is not a failure.
        store.remove("2026-08-02-140000").unwrap();
    }

    /// `name` and not just the format: two tests may export the same format,
    /// and tests run in parallel - sharing a directory means one wipes the
    /// other's session out from under it.
    fn exported(name: &str, format: Format) -> String {
        let store = recorded(&format!("export-{name}"));
        store.record(RowUpdate {
            line: 0,
            source: Some("Good evening.".into()),
            ..RowUpdate::default()
        });
        store.record(RowUpdate {
            line: 0,
            translation: Some("Добрый вечер.".into()),
            ..RowUpdate::default()
        });
        store.record(RowUpdate {
            line: 1,
            source: Some("Officials expect service restored.".into()),
            ..RowUpdate::default()
        });
        store.finish().unwrap();

        let to = store
            .directory()
            .join(format!("out.{}", format.extension()));
        store.export("2026-08-02-140000", format, &to).unwrap();
        fs::read_to_string(to).unwrap()
    }

    #[test]
    fn text_export_carries_both_languages() {
        let text = exported("text", Format::Text);

        assert!(text.contains("Good evening."));
        assert!(text.contains("Добрый вечер."));
        assert!(text.contains("large-v3-turbo-q5_0"));
    }

    #[test]
    fn srt_export_is_numbered_and_timed() {
        let srt = exported("srt", Format::Srt);

        assert!(srt.starts_with("1\n00:00:"), "{srt}");
        assert!(srt.contains(" --> "));
        assert!(srt.contains("2\n"));
        // Translation leads, original underneath.
        let first = srt.find("Добрый вечер.").unwrap();
        assert!(first < srt.find("Good evening.").unwrap());
    }

    #[test]
    fn json_export_can_be_read_back() {
        let session: Session = serde_json::from_str(&exported("json", Format::Json)).unwrap();

        assert_eq!(session.rows.len(), 2);
        assert_eq!(session.asr_model, "large-v3-turbo-q5_0");
    }

    #[test]
    fn a_row_with_no_translation_still_appears_in_subtitles() {
        assert!(exported("srt-untranslated", Format::Srt)
            .contains("Officials expect service restored."));
    }

    #[test]
    fn srt_times_are_hours_minutes_seconds_milliseconds() {
        assert_eq!(srt_time(0.0), "00:00:00,000");
        assert_eq!(srt_time(3661.5), "01:01:01,500");
        // A negative time cannot happen, and must not produce nonsense if it does.
        assert_eq!(srt_time(-1.0), "00:00:00,000");
    }

    #[test]
    fn a_missing_session_is_an_error_not_a_panic() {
        let store = store("missing");
        assert!(matches!(store.read("nope"), Err(HistoryError::NotFound(_))));
    }

    #[test]
    fn a_directory_that_does_not_exist_lists_nothing() {
        let store = HistoryStore::new(std::env::temp_dir().join("marswind-history-absent"));
        let _ = fs::remove_dir_all(store.directory());
        assert!(store.list().unwrap().is_empty());
    }
}
