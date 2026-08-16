//! Filling the session log from the events the interface already receives.
//!
//! The recorder listens to the same `asr://` and `translate://` events the
//! transcript is drawn from, rather than being called from inside the pipeline.
//! That keeps recognition and translation ignorant of it - and it means what is
//! written down is exactly what was shown, not a second version of the truth
//! assembled from a private path.

use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, EventId, Listener, Manager};

use super::{HistoryError, HistoryStore, Meta, RowUpdate};

/// Holds the listeners for as long as a session is being recorded.
#[derive(Default)]
pub struct Recorder {
    listeners: Mutex<Vec<EventId>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Begins a session and starts listening. The id is the epoch second it
    /// started, which makes the files sort by time on their name alone.
    pub fn start(
        &self,
        app: &AppHandle,
        started_at: String,
        meta: Meta,
    ) -> Result<String, HistoryError> {
        self.detach(app);

        let id = epoch_id();
        let store = Arc::clone(&app.state::<Arc<HistoryStore>>());
        store.start(id.clone(), started_at, meta)?;

        let mut listeners = self.listeners.lock();

        {
            let store = Arc::clone(&store);
            listeners.push(app.listen("asr://transcript", move |event| {
                let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) else {
                    return;
                };
                // Only a finished row is worth writing down: an unfinished one is
                // still growing, and its text is a prefix of what will arrive.
                if !payload["final"].as_bool().unwrap_or(false) {
                    return;
                }
                store.record(RowUpdate {
                    line: payload["line"].as_u64().unwrap_or(0),
                    source: payload["text"].as_str().map(str::to_string),
                    recognition_ms: payload["inferenceMs"].as_u64(),
                    ..RowUpdate::default()
                });
            }));
        }

        {
            let store = Arc::clone(&store);
            listeners.push(app.listen("translate://line", move |event| {
                let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) else {
                    return;
                };
                store.record(RowUpdate {
                    line: payload["line"].as_u64().unwrap_or(0),
                    translation: payload["text"].as_str().map(str::to_string),
                    translation_ms: payload["translationMs"].as_u64(),
                    ..RowUpdate::default()
                });
            }));
        }

        {
            let store = Arc::clone(&store);
            listeners.push(app.listen("translate://skipped", move |event| {
                let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) else {
                    return;
                };
                store.record(RowUpdate {
                    line: payload["line"].as_u64().unwrap_or(0),
                    skipped: true,
                    ..RowUpdate::default()
                });
            }));
        }

        Ok(id)
    }

    /// Ends the session and writes it out. Returns its id, or `None` if nothing
    /// was recognized and there is nothing worth keeping.
    pub fn stop(&self, app: &AppHandle) -> Result<Option<String>, HistoryError> {
        self.detach(app);
        app.state::<Arc<HistoryStore>>().finish()
    }

    fn detach(&self, app: &AppHandle) {
        for id in self.listeners.lock().drain(..) {
            app.unlisten(id);
        }
    }
}

/// Seconds since the epoch, zero-padded so ids of different lengths never sort
/// out of order - which they would not until the year 2286, but the padding
/// costs nothing and the sort depends on it.
fn epoch_id() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{seconds:012}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_is_padded_to_a_fixed_width() {
        let id = epoch_id();

        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn ids_sort_as_text_in_the_order_they_were_made() {
        let mut ids = [
            format!("{:012}", 1_800_000_000u64),
            format!("{:012}", 999u64),
        ];
        ids.sort();

        assert_eq!(ids[0], format!("{:012}", 999u64));
    }
}
