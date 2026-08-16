//! The line protocol between the app and this worker.
//!
//! One JSON object per line in each direction. Deliberately dull: a translation
//! is a small request with a small answer, and anything cleverer would be a
//! second thing to debug when subtitles stop appearing.

use serde::{Deserialize, Serialize};

/// A previously translated caption, kept as context for the next one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    /// Echoed back so the app can match answers to questions.
    pub id: u64,
    pub source: String,
    #[serde(default)]
    pub history: Vec<Pair>,
    /// Target language in English, as the model should see it: "Russian".
    pub target: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub ms: u64,
}

/// A piece of a translation, sent while it is still being generated.
///
/// Waiting for a finished sentence costs the reader the whole generation time -
/// one to four seconds - before a single word appears. Chunks arrive between the
/// request and its `Response`, which stays the authoritative text.
#[derive(Debug, Serialize)]
pub struct Chunk {
    pub id: u64,
    pub delta: String,
}

/// Sent once the model is loaded, so the app knows the worker is usable rather
/// than merely started.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub ready: bool,
    pub load_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_parses_without_history() {
        let request: Request =
            serde_json::from_str(r#"{"id":7,"source":"Good evening.","target":"Russian"}"#)
                .unwrap();

        assert_eq!(request.id, 7);
        assert!(request.history.is_empty());
    }

    #[test]
    fn a_request_parses_with_history() {
        let request: Request = serde_json::from_str(
            r#"{"id":1,"source":"They voted.","target":"Russian",
                "history":[{"source":"The committee met.","target":"Комитет собрался."}]}"#,
        )
        .unwrap();

        assert_eq!(request.history.len(), 1);
        assert_eq!(request.history[0].target, "Комитет собрался.");
    }

    #[test]
    fn a_successful_response_omits_the_error_field() {
        let json = serde_json::to_string(&Response {
            id: 3,
            text: Some("Добрый вечер.".into()),
            error: None,
            ms: 120,
        })
        .unwrap();

        assert!(json.contains("\"text\""));
        assert!(!json.contains("error"));
    }

    #[test]
    fn a_failed_response_omits_the_text_field() {
        let json = serde_json::to_string(&Response {
            id: 4,
            text: None,
            error: Some("out of memory".into()),
            ms: 0,
        })
        .unwrap();

        assert!(json.contains("\"error\""));
        assert!(!json.contains("\"text\""));
    }

    #[test]
    fn a_chunk_carries_the_id_of_its_request() {
        let json = serde_json::to_string(&Chunk {
            id: 9,
            delta: "Добрый ".into(),
        })
        .unwrap();

        assert!(json.contains("\"id\":9"));
        assert!(json.contains("\"delta\""));
        // The app tells a chunk from an answer by the field, so it must not
        // look like one.
        assert!(!json.contains("\"text\""));
    }

    #[test]
    fn responses_stay_on_one_line() {
        let json = serde_json::to_string(&Response {
            id: 5,
            text: Some("Первая строка.\nВторая строка.".into()),
            error: None,
            ms: 10,
        })
        .unwrap();

        assert!(!json.contains('\n'), "a newline would break the framing");
    }
}
