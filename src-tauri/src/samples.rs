//! The speech clips shipped with the app, and playing one of them.
//!
//! The app is hard to try without something to listen to: it captures what the
//! machine is playing, so an empty desktop produces an empty transcript, and
//! "it does not work" and "nothing is making a sound" look identical. These are
//! the same clips the test corpus uses - synthesised speech with a known
//! transcript - played through the normal system output, so they go through the
//! tap and the whole pipeline exactly as a video would.

use std::path::PathBuf;
use std::process::Child;

use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Manager};

/// Clips bundled as resources, with what each one is for.
const SAMPLES: &[(&str, &str, &str)] = &[
    (
        "news-bulletin",
        "News bulletin",
        "A clear read at broadcast pace. The one to try first.",
    ),
    (
        "named-entities",
        "Names and numbers",
        "Place names, dates and figures - where recognition struggles most.",
    ),
    (
        "fast-conversational",
        "Fast speech",
        "Long sentences, few pauses. Stresses the streaming path.",
    ),
    (
        "two-speakers",
        "Two speakers",
        "The voice changes every sentence.",
    ),
];

#[derive(Debug, thiserror::Error)]
pub enum SampleError {
    #[error("no sample named '{0}'")]
    NotFound(String),
    #[error("the sample clips are missing from the application bundle")]
    Missing,
    #[error("could not play the sample: {0}")]
    Playback(String),
    #[error("playing samples is not supported on this platform")]
    Unsupported,
}

impl Serialize for SampleError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    /// The exact words spoken, so what came out can be compared with what went
    /// in without leaving the app.
    pub transcript: String,
}

#[derive(Default)]
pub struct SamplePlayer {
    playing: Mutex<Option<(String, Child)>>,
}

impl SamplePlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self, app: &AppHandle) -> Vec<SampleInfo> {
        SAMPLES
            .iter()
            .map(|(id, name, note)| SampleInfo {
                id,
                name,
                note,
                transcript: resource(app, &format!("samples/{id}.txt"))
                    .and_then(|path| std::fs::read_to_string(path).ok())
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            })
            .collect()
    }

    /// Which sample is playing, if any.
    pub fn playing(&self) -> Option<String> {
        let mut guard = self.playing.lock();
        let (id, child) = guard.as_mut()?;

        // A finished clip leaves its process behind; reap it so the interface
        // does not keep showing a Stop button for something already over.
        match child.try_wait() {
            Ok(Some(_)) => {
                let done = id.clone();
                *guard = None;
                let _ = done;
                None
            }
            _ => Some(id.clone()),
        }
    }

    pub fn play(&self, app: &AppHandle, id: &str) -> Result<(), SampleError> {
        if !SAMPLES.iter().any(|(known, _, _)| *known == id) {
            return Err(SampleError::NotFound(id.to_string()));
        }
        let path = resource(app, &format!("samples/{id}.wav")).ok_or(SampleError::Missing)?;

        self.stop();
        let child = spawn_player(&path)?;
        *self.playing.lock() = Some((id.to_string(), child));
        Ok(())
    }

    pub fn stop(&self) {
        if let Some((_, mut child)) = self.playing.lock().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for SamplePlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn resource(app: &AppHandle, relative: &str) -> Option<PathBuf> {
    let path = app
        .path()
        .resolve(relative, tauri::path::BaseDirectory::Resource)
        .ok()?;
    path.is_file().then_some(path)
}

#[cfg(target_os = "macos")]
fn spawn_player(path: &PathBuf) -> Result<Child, SampleError> {
    // `afplay` goes to the default output device, which is what the tap is
    // listening to. Anything that played the file privately would test nothing.
    std::process::Command::new("afplay")
        .arg(path)
        .spawn()
        .map_err(|e| SampleError::Playback(e.to_string()))
}

#[cfg(not(target_os = "macos"))]
fn spawn_player(_path: &PathBuf) -> Result<Child, SampleError> {
    Err(SampleError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sample_has_a_name_and_a_note() {
        assert!(!SAMPLES.is_empty());
        for (id, name, note) in SAMPLES {
            assert!(!id.is_empty() && !name.is_empty() && !note.is_empty());
        }
    }

    #[test]
    fn sample_ids_are_unique() {
        let mut ids: Vec<&str> = SAMPLES.iter().map(|(id, _, _)| *id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();

        assert_eq!(ids.len(), count);
    }

    #[test]
    fn nothing_is_playing_to_begin_with() {
        let player = SamplePlayer::new();
        assert!(player.playing().is_none());
        // Stopping when nothing is playing is not an error.
        player.stop();
    }
}
