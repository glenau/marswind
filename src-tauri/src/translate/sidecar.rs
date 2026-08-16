//! Talking to the translation worker.
//!
//! Translation runs in its own process, not out of caution but out of
//! necessity: whisper.cpp and llama.cpp each bundle a copy of ggml, and linking
//! both into one executable silently corrupts whichever loses the symbol race.
//! The failure is not a link error - recognition simply starts emitting
//! "!!!!!!!" instead of words.
//!
//! The upside of being forced here: a translator that runs out of memory or
//! crashes takes nothing else with it.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde::{Deserialize, Serialize};

use super::language::Language;
use super::{Pair, TranslateError, Translator};

const BINARY_NAME: &str = "marswind-translator";

#[derive(Serialize)]
struct Request<'a> {
    id: u64,
    source: &'a str,
    history: &'a [Pair],
    target: &'a str,
}

/// One line back from the worker. A `delta` is a piece of a translation still
/// being generated; anything else is the finished answer.
#[derive(Deserialize)]
struct Response {
    id: u64,
    #[serde(default)]
    delta: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct Ready {
    ready: bool,
    #[serde(rename = "loadMs", default)]
    load_ms: u64,
}

impl Serialize for Pair {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut pair = serializer.serialize_struct("Pair", 2)?;
        pair.serialize_field("source", &self.source)?;
        pair.serialize_field("target", &self.target)?;
        pair.end()
    }
}

pub struct SidecarTranslator {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl SidecarTranslator {
    pub fn spawn(model: &Path, threads: i32, template: &str) -> Result<Self, TranslateError> {
        let binary = locate_binary()?;
        log::info!("starting the translation worker at {}", binary.display());

        let mut child = Command::new(&binary)
            .arg("--model")
            .arg(model)
            .arg("--threads")
            .arg(threads.to_string())
            .arg("--template")
            .arg(template)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                TranslateError::Load(format!("could not start {}: {e}", binary.display()))
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TranslateError::Load("the worker has no stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TranslateError::Load("the worker has no stdout".into()))?;

        // llama.cpp is chatty on stderr. Draining it into the log keeps the
        // pipe from filling up and stalling the worker mid-sentence.
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    log::debug!("translator: {line}");
                }
            });
        }

        let mut translator = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        translator.wait_until_ready()?;

        Ok(translator)
    }

    /// The worker announces itself once the model is loaded, so a slow load
    /// looks like a slow start rather than a translation that never arrives.
    fn wait_until_ready(&mut self) -> Result<(), TranslateError> {
        let mut line = String::new();
        if self
            .stdout
            .read_line(&mut line)
            .map_err(|e| TranslateError::Load(e.to_string()))?
            == 0
        {
            return Err(TranslateError::Load(
                "the worker exited before it was ready - check the log for its error".into(),
            ));
        }

        let ready: Ready = serde_json::from_str(line.trim())
            .map_err(|e| TranslateError::Load(format!("unexpected greeting {line:?}: {e}")))?;
        if !ready.ready {
            return Err(TranslateError::Load("the worker reported a failure".into()));
        }

        log::info!(
            "translation worker ready, model loaded in {} ms",
            ready.load_ms
        );
        Ok(())
    }
}

impl Drop for SidecarTranslator {
    fn drop(&mut self) {
        // Closing stdin ends the worker's read loop; the kill is for a worker
        // stuck mid-generation.
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Translator for SidecarTranslator {
    fn name(&self) -> &'static str {
        "llm"
    }

    fn translate(
        &mut self,
        source: &str,
        history: &[Pair],
        target: Language,
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<String, TranslateError> {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::to_string(&Request {
            id,
            source,
            history,
            target: target.name,
        })
        .map_err(|e| TranslateError::Generation(e.to_string()))?;

        writeln!(self.stdin, "{request}")
            .and_then(|()| self.stdin.flush())
            .map_err(|e| TranslateError::Generation(format!("the worker is gone: {e}")))?;

        // Requests are answered in order, so anything with a different id is a
        // stale answer from a request we gave up on.
        loop {
            let mut line = String::new();
            if self
                .stdout
                .read_line(&mut line)
                .map_err(|e| TranslateError::Generation(e.to_string()))?
                == 0
            {
                return Err(TranslateError::Generation(
                    "the worker exited mid-translation".into(),
                ));
            }

            let response: Response = match serde_json::from_str(line.trim()) {
                Ok(response) => response,
                Err(e) => {
                    log::warn!("unparseable answer from the worker {line:?}: {e}");
                    continue;
                }
            };

            if response.id != id {
                continue;
            }
            if let Some(delta) = response.delta {
                on_delta(&delta);
                continue;
            }
            if let Some(error) = response.error {
                return Err(TranslateError::Generation(error));
            }
            return Ok(response.text.unwrap_or_default());
        }
    }
}

/// Finds the worker binary next to the app, which is where both a bundle and a
/// development build put it.
fn locate_binary() -> Result<PathBuf, TranslateError> {
    let mut candidates = Vec::new();

    if let Ok(executable) = std::env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.push(directory.join(BINARY_NAME));
            candidates.push(directory.join(format!("{BINARY_NAME}-{}", target_triple())));
        }
    }

    // Running from a checkout, before anything has been bundled.
    if let Ok(directory) = std::env::current_dir() {
        candidates.push(
            directory
                .join("translator/target/release")
                .join(BINARY_NAME),
        );
        candidates.push(
            directory
                .join("../translator/target/release")
                .join(BINARY_NAME),
        );
    }

    candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .ok_or_else(|| {
            TranslateError::Load(format!(
                "could not find the translation worker; looked in {}",
                candidates
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}

fn target_triple() -> &'static str {
    // Tauri names bundled sidecars after the target they were built for.
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_serialises_with_the_language_name() {
        let language = crate::translate::language::find("ru").unwrap();
        let json = serde_json::to_string(&Request {
            id: 1,
            source: "Good evening.",
            history: &[],
            target: language.name,
        })
        .unwrap();

        assert!(json.contains("\"target\":\"Russian\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn history_serialises_as_pairs() {
        let history = vec![Pair {
            source: "The committee met.".into(),
            target: "Комитет собрался.".into(),
        }];
        let json = serde_json::to_string(&Request {
            id: 2,
            source: "They voted.",
            history: &history,
            target: "Russian",
        })
        .unwrap();

        assert!(json.contains("Комитет собрался."));
    }

    #[test]
    fn a_response_parses_either_way() {
        let ok: Response =
            serde_json::from_str(r#"{"id":1,"text":"Добрый вечер.","ms":10}"#).unwrap();
        assert_eq!(ok.text.as_deref(), Some("Добрый вечер."));
        assert!(ok.error.is_none());

        let failed: Response = serde_json::from_str(r#"{"id":2,"error":"boom","ms":0}"#).unwrap();
        assert_eq!(failed.error.as_deref(), Some("boom"));
    }

    #[test]
    fn a_chunk_is_told_apart_from_an_answer() {
        let chunk: Response = serde_json::from_str(r#"{"id":3,"delta":"Добрый "}"#).unwrap();
        assert_eq!(chunk.delta.as_deref(), Some("Добрый "));
        assert!(chunk.text.is_none());

        let answer: Response =
            serde_json::from_str(r#"{"id":3,"text":"Добрый вечер.","ms":900}"#).unwrap();
        assert!(answer.delta.is_none());
    }

    #[test]
    fn the_target_triple_is_known_on_this_platform() {
        assert_ne!(target_triple(), "unknown");
    }
}
