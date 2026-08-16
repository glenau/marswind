//! Marswind's translation worker.
//!
//! Reads one JSON request per line on stdin, writes one JSON response per line
//! on stdout, and keeps a loaded model in between. It exists as a separate
//! process because llama.cpp and whisper.cpp cannot share one - see Cargo.toml.
//!
//! Usage: marswind-translator --model <path.gguf> [--threads N] [--template NAME]

mod engine;
mod protocol;

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Instant;

use llama_cpp_2::llama_backend::LlamaBackend;

use engine::{LlmTranslator, PromptTemplate};
use protocol::{Chunk, Ready, Request, Response};

fn main() {
    if let Err(message) = run() {
        eprintln!("marswind-translator: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let (model_path, threads, template) = parse_arguments()?;

    let loading = Instant::now();
    let backend = LlamaBackend::init().map_err(|e| e.to_string())?;
    let mut translator = LlmTranslator::load(&backend, &model_path, threads, template)?;

    // Everything the app needs to know before it sends work.
    emit(&Ready {
        ready: true,
        load_ms: loading.elapsed().as_millis() as u64,
    });

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("could not read a request: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(e) => {
                eprintln!("marswind-translator: malformed request: {e}");
                continue;
            }
        };

        let started = Instant::now();
        // Each piece goes out as it is generated, so the subtitle grows instead
        // of appearing whole a few seconds later.
        let id = request.id;
        let mut on_delta = |delta: &str| {
            emit(&Chunk {
                id,
                delta: delta.to_string(),
            });
        };
        let response = match translator.translate(
            &request.source,
            &request.history,
            &request.target,
            &mut on_delta,
        ) {
            Ok(text) => Response {
                id: request.id,
                text: Some(text),
                error: None,
                ms: started.elapsed().as_millis() as u64,
            },
            Err(error) => Response {
                id: request.id,
                text: None,
                error: Some(error),
                ms: started.elapsed().as_millis() as u64,
            },
        };

        emit(&response);
    }

    Ok(())
}

fn emit<T: serde::Serialize>(value: &T) {
    let mut stdout = std::io::stdout().lock();
    match serde_json::to_string(value) {
        Ok(line) => {
            let _ = writeln!(stdout, "{line}");
            // The app is waiting on this line; buffering it would look like a
            // hung translator.
            let _ = stdout.flush();
        }
        Err(e) => eprintln!("marswind-translator: could not serialise a response: {e}"),
    }
}

fn parse_arguments() -> Result<(PathBuf, i32, PromptTemplate), String> {
    let mut model = None;
    let mut threads = 4;
    // ChatML unless told otherwise: it is what the models this shipped with
    // use, and a wrong guess here is not a crash but an answer with the turn
    // markers written out as text.
    let mut template = PromptTemplate::ChatMl;

    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--model" => {
                model = arguments.next().map(PathBuf::from);
            }
            "--threads" => {
                threads = arguments
                    .next()
                    .and_then(|value| value.parse().ok())
                    .ok_or("--threads needs a number")?;
            }
            "--template" => {
                let name = arguments.next().ok_or("--template needs a name")?;
                template = PromptTemplate::parse(&name)
                    .ok_or_else(|| format!("unknown prompt template '{name}'"))?;
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }

    let model = model.ok_or("--model is required")?;
    if !model.is_file() {
        return Err(format!("no model at {}", model.display()));
    }

    Ok((model, threads, template))
}
