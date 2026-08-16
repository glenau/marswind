//! Headless checks driven by `MARSWIND_SELFTEST`.
//!
//! The app can exercise its own pipeline against real audio without a human
//! clicking through the UI, which is how capture, recognition and translation
//! get verified against real audio and what the scripts in `tests/` drive.
//!
//! Modes:
//!   `list`             - print the available audio sources
//!   `capture:<sec>`    - record system audio to a WAV and report its level
//!   `download:<id>`    - install a model from the catalog
//!   `asr:<sec>`        - capture and transcribe, printing every phrase
//!   `pipeline:<sec>`   - capture, transcribe and translate, printing both
//!   `quit:<sec>`       - start everything and then exit with it still running
//!
//! A bare number is shorthand for `capture:<sec>`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tauri::{AppHandle, Listener, Manager};

use crate::asr::{AsrConfig, AsrEngine};
use crate::audio::{AudioEngine, TARGET_SAMPLE_RATE};
use crate::history::recorder::Recorder;
use crate::history::Meta;
use crate::models::{catalog, ModelStore};
use crate::translate::{language, Engine, TranslateConfig, TranslationEngine};

/// One finished translation, with the two numbers that matter: when its first
/// word reached the screen and when the sentence was complete.
struct Translated {
    #[allow(dead_code)]
    text: String,
    total_ms: u64,
    first_word_ms: u64,
}

fn median(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

pub fn spawn(app: AppHandle, mode: String) {
    std::thread::spawn(move || {
        let code = match dispatch(&app, &mode) {
            Ok(()) => 0,
            Err(message) => {
                eprintln!("SELFTEST FAIL {message}");
                1
            }
        };
        app.exit(code);
    });
}

fn dispatch(app: &AppHandle, mode: &str) -> Result<(), String> {
    let (kind, argument) = mode.split_once(':').unwrap_or(("capture", mode));

    match kind {
        "list" => list_sources(app),
        "capture" => capture(app, parse_seconds(argument)?),
        "download" => download(app, argument),
        "asr" => transcribe(app, parse_seconds(argument)?, false),
        "pipeline" => transcribe(app, parse_seconds(argument)?, true),
        "quit" => quit_while_running(app, parse_seconds(argument)?),
        other => Err(format!("unknown self-test mode '{other}'")),
    }
}

fn parse_seconds(value: &str) -> Result<f32, String> {
    value
        .parse::<f32>()
        .map_err(|_| format!("'{value}' is not a number of seconds"))
}

fn list_sources(app: &AppHandle) -> Result<(), String> {
    let engine = app.state::<AudioEngine>();
    for source in engine.list_sources().map_err(|e| e.to_string())? {
        println!(
            "SOURCE {} active={} name={} detail={}",
            source.id,
            source.active,
            source.name,
            source.detail.unwrap_or_default()
        );
    }
    Ok(())
}

fn download(app: &AppHandle, model_id: &str) -> Result<(), String> {
    let store = Arc::clone(&app.state::<Arc<ModelStore>>());
    let handle = app.clone();

    println!("SELFTEST download {model_id}");
    let path = tauri::async_runtime::block_on(store.download(model_id, handle))
        .map_err(|e| e.to_string())?;

    println!("SELFTEST PASS installed {}", path.display());
    Ok(())
}

fn capture(app: &AppHandle, seconds: f32) -> Result<(), String> {
    let engine = app.state::<AudioEngine>();
    let source = source_from_env();
    let output = output_path(app, "selftest.wav");

    println!(
        "SELFTEST start source={source} seconds={seconds} output={}",
        output.display()
    );

    let format = engine
        .start(app.clone(), &source)
        .map_err(|e| e.to_string())?;
    println!(
        "SELFTEST format sample_rate={} channels={}",
        format.sample_rate, format.channels
    );

    engine
        .record_wav(seconds, output.clone())
        .map_err(|e| e.to_string())?;

    let mut playback = start_playback()?;

    let deadline = Instant::now() + Duration::from_secs_f32(seconds + 5.0);
    while Instant::now() < deadline && engine.state().recording {
        std::thread::sleep(Duration::from_millis(200));
    }

    if let Some(child) = playback.as_mut() {
        let _ = child.kill();
    }

    let state = engine.state();
    let still_recording = state.recording;
    engine.stop().map_err(|e| e.to_string())?;

    if still_recording {
        return Err("recording did not complete - no audio reached the pipeline".into());
    }

    report_wav(&output, state.dropped_samples)
}

fn transcribe(app: &AppHandle, seconds: f32, with_translation: bool) -> Result<(), String> {
    let audio = app.state::<AudioEngine>();
    let asr = app.state::<AsrEngine>();
    let store = Arc::clone(&app.state::<Arc<ModelStore>>());

    let model_id = std::env::var("MARSWIND_SELFTEST_MODEL")
        .unwrap_or_else(|_| catalog::recommended_asr(crate::models::total_memory_bytes()).into());
    let model_path = store.installed_path(&model_id).map_err(|e| e.to_string())?;
    let vad_model_path = store
        .installed_path(catalog::VAD_MODEL_ID)
        .map_err(|e| e.to_string())?;

    let source = source_from_env();
    let model_for_history = model_id.clone();
    println!("SELFTEST asr source={source} model={model_id} seconds={seconds}");

    // Collected from the transcript events the UI also consumes, so this checks
    // the real path rather than a private shortcut.
    let phrases = Arc::new(Mutex::new(Vec::<String>::new()));
    let updates = Arc::new(AtomicU64::new(0));
    let slowest = Arc::new(AtomicU64::new(0));
    // Set once playback starts, so every caption can be timed against the audio
    // that produced it.
    let playback_start: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

    {
        let phrases = Arc::clone(&phrases);
        let updates = Arc::clone(&updates);
        let slowest = Arc::clone(&slowest);
        let playback_start = Arc::clone(&playback_start);
        app.listen("asr://transcript", move |event| {
            let elapsed = playback_start
                .lock()
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(0.0);
            updates.fetch_add(1, Ordering::Relaxed);
            let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) else {
                return;
            };

            let inference = payload["inferenceMs"].as_u64().unwrap_or(0);
            slowest.fetch_max(inference, Ordering::Relaxed);

            let text = payload["text"].as_str().unwrap_or_default();
            let tentative = payload["tentative"].as_str().unwrap_or_default();

            if payload["final"].as_bool().unwrap_or(false) {
                println!("SELFTEST phrase t={elapsed:.2} [{inference} ms] {text}");
                phrases.lock().push(text.to_string());
            } else if !text.is_empty() || !tentative.is_empty() {
                println!("SELFTEST partial t={elapsed:.2} [{inference} ms] {text} | {tentative}");
            }
        });
    }

    let translations = Arc::new(Mutex::new(Vec::<Translated>::new()));
    if with_translation {
        // The first word of a translation is when the reader stops waiting, so
        // it is timed separately from the finished sentence.
        let started_lines = Arc::new(Mutex::new(Vec::<(u64, u64)>::new()));
        {
            let started_lines = Arc::clone(&started_lines);
            let playback_start = Arc::clone(&playback_start);
            app.listen("translate://partial", move |event| {
                let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) else {
                    return;
                };
                let line = payload["line"].as_u64().unwrap_or(0);
                let seq = payload["seq"].as_u64().unwrap_or(0);
                let mut started_lines = started_lines.lock();
                if started_lines.contains(&(line, seq)) {
                    return;
                }
                started_lines.push((line, seq));

                let elapsed = playback_start
                    .lock()
                    .map(|start| start.elapsed().as_secs_f32())
                    .unwrap_or(0.0);
                let text = payload["text"].as_str().unwrap_or_default();
                println!("SELFTEST translating t={elapsed:.2} line={line}.{seq} {text}");
            });
        }

        let translations = Arc::clone(&translations);
        let playback_start = Arc::clone(&playback_start);
        app.listen("translate://line", move |event| {
            let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) else {
                return;
            };
            let elapsed = playback_start
                .lock()
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(0.0);
            let line = payload["line"].as_u64().unwrap_or(0);
            let seq = payload["seq"].as_u64().unwrap_or(0);
            let source = payload["source"].as_str().unwrap_or_default().to_string();
            let text = payload["text"].as_str().unwrap_or_default().to_string();
            let ms = payload["translationMs"].as_u64().unwrap_or(0);
            let first_word_ms = payload["firstWordMs"].as_u64().unwrap_or(0);
            println!(
                "SELFTEST translated t={elapsed:.2} line={line}.{seq} [{first_word_ms} ms to first word, {ms} ms total] {source}  ->  {text}"
            );
            translations.lock().push(Translated {
                text,
                total_ms: ms,
                first_word_ms,
            });
        });
    }

    audio
        .start(app.clone(), &source)
        .map_err(|e| e.to_string())?;
    asr.start(
        app.clone(),
        audio.subscribe(),
        AsrConfig {
            model_path,
            vad_model_path,
            model_id,
            language: std::env::var("MARSWIND_SELFTEST_LANGUAGE")
                .ok()
                .or_else(|| Some("en".to_string())),
            use_prompt: std::env::var("MARSWIND_SELFTEST_NO_PROMPT").is_err(),
        },
    )
    .map_err(|e| e.to_string())?;

    if with_translation {
        let translation = app.state::<TranslationEngine>();
        let mt_model_id = std::env::var("MARSWIND_SELFTEST_MT_MODEL").unwrap_or_else(|_| {
            catalog::recommended_mt(crate::models::total_memory_bytes()).into()
        });
        let mt_model_path = store
            .installed_path(&mt_model_id)
            .map_err(|e| e.to_string())?;
        let target_code =
            std::env::var("MARSWIND_SELFTEST_TARGET").unwrap_or_else(|_| "ru".to_string());
        let target = *language::find(&target_code)
            .ok_or_else(|| format!("unknown target language '{target_code}'"))?;

        println!(
            "SELFTEST translate model={mt_model_id} target={}",
            target.code
        );
        translation
            .start(
                app.clone(),
                asr.subscribe(),
                TranslateConfig {
                    engine: Engine::Llm,
                    model_path: mt_model_path,
                    model_id: mt_model_id,
                    target,
                },
            )
            .map_err(|e| e.to_string())?;
    }

    // Recorded like a real session, so the self-test covers the history path
    // rather than leaving it to be discovered broken by a user.
    let recorder = app.state::<Recorder>();
    let session = recorder
        .start(
            app,
            "self-test".to_string(),
            Meta {
                source: source.clone(),
                asr_model: model_for_history.clone(),
                spoken_language: "en".into(),
                ..Meta::default()
            },
        )
        .map_err(|e| e.to_string())?;
    println!("SELFTEST session {session}");

    // Playing the reference audio from here means the clock starts at a known
    // instant, which is what makes the reported lag meaningful.
    let mut playback = start_playback()?;
    if playback.is_some() {
        *playback_start.lock() = Some(Instant::now());
    }

    std::thread::sleep(Duration::from_secs_f32(seconds));
    if let Some(child) = playback.as_mut() {
        let _ = child.kill();
    }

    if with_translation {
        // Give the last caption time to come back translated.
        std::thread::sleep(Duration::from_secs(5));
        let _ = app.state::<TranslationEngine>().stop();
    }
    let _ = asr.stop();
    let dropped = audio.state().dropped_samples;
    let _ = audio.stop();

    match app.state::<Recorder>().stop(app) {
        Ok(Some(id)) => println!("SELFTEST session saved {id}"),
        Ok(None) => println!("SELFTEST session empty"),
        Err(e) => println!("SELFTEST session failed {e}"),
    }

    let phrases = phrases.lock().clone();
    let words: usize = phrases.iter().map(|p| p.split_whitespace().count()).sum();

    println!(
        "SELFTEST result phrases={} words={} updates={} slowest_inference_ms={} dropped={}",
        phrases.len(),
        words,
        updates.load(Ordering::Relaxed),
        slowest.load(Ordering::Relaxed),
        dropped
    );

    if phrases.is_empty() {
        return Err("no phrases were transcribed".into());
    }

    if with_translation {
        let translations = translations.lock();
        let slowest = translations.iter().map(|t| t.total_ms).max().unwrap_or(0);
        println!(
            "SELFTEST translation lines={} slowest_ms={slowest} first_word_ms median={} worst={}",
            translations.len(),
            median(
                &translations
                    .iter()
                    .map(|t| t.first_word_ms)
                    .collect::<Vec<_>>()
            ),
            translations
                .iter()
                .map(|t| t.first_word_ms)
                .max()
                .unwrap_or(0),
        );
        if translations.is_empty() {
            return Err("nothing was translated".into());
        }
        if translations.iter().all(|t| t.first_word_ms == t.total_ms) {
            return Err("no translation arrived in pieces - streaming is not working".into());
        }
    }

    println!("SELFTEST PASS");
    Ok(())
}

/// Starts the whole pipeline and then quits with all of it still running -
/// which is what closing the window while listening does.
///
/// Nothing is stopped here on purpose. The exit handler has to bring the stages
/// down in the right order; without it the audio tap is still calling into a
/// ring buffer, and whisper still has work on the GPU, while the state they
/// belong to is dropped around them, and the app dies on the way out.
fn quit_while_running(app: &AppHandle, seconds: f32) -> Result<(), String> {
    let audio = app.state::<AudioEngine>();
    let asr = app.state::<AsrEngine>();
    let store = Arc::clone(&app.state::<Arc<ModelStore>>());

    let model_id = std::env::var("MARSWIND_SELFTEST_MODEL")
        .unwrap_or_else(|_| catalog::recommended_asr(crate::models::total_memory_bytes()).into());
    let model_path = store.installed_path(&model_id).map_err(|e| e.to_string())?;
    let vad_model_path = store
        .installed_path(catalog::VAD_MODEL_ID)
        .map_err(|e| e.to_string())?;

    let source = source_from_env();
    audio
        .start(app.clone(), &source)
        .map_err(|e| e.to_string())?;
    asr.start(
        app.clone(),
        audio.subscribe(),
        AsrConfig {
            model_path,
            vad_model_path,
            model_id: model_id.clone(),
            language: Some("en".to_string()),
            use_prompt: true,
        },
    )
    .map_err(|e| e.to_string())?;

    let translation = app.state::<TranslationEngine>();
    let mt_model_id = std::env::var("MARSWIND_SELFTEST_MT_MODEL")
        .unwrap_or_else(|_| catalog::recommended_mt(crate::models::total_memory_bytes()).into());
    if let Ok(mt_model_path) = store.installed_path(&mt_model_id) {
        let target = *language::find("ru").ok_or("unknown target language")?;
        translation
            .start(
                app.clone(),
                asr.subscribe(),
                TranslateConfig {
                    engine: Engine::Llm,
                    model_path: mt_model_path,
                    model_id: mt_model_id,
                    target,
                },
            )
            .map_err(|e| e.to_string())?;
    }

    let recorder = app.state::<Recorder>();
    recorder
        .start(
            app,
            "quit-test".to_string(),
            Meta {
                source,
                asr_model: model_id,
                ..Meta::default()
            },
        )
        .map_err(|e| e.to_string())?;

    let mut playback = start_playback()?;
    std::thread::sleep(Duration::from_secs_f32(seconds));
    if let Some(child) = playback.as_mut() {
        let _ = child.kill();
    }

    println!("SELFTEST quitting with the pipeline still running");
    println!("SELFTEST PASS");
    Ok(())
}

fn report_wav(path: &PathBuf, dropped: u64) -> Result<(), String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| format!("cannot read wav: {e}"))?;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .filter_map(Result::ok)
        .map(|s| s as f32 / i16::MAX as f32)
        .collect();

    if samples.is_empty() {
        return Err("recorded file is empty".into());
    }

    let level = crate::audio::level::Level::measure(&samples);
    let seconds = samples.len() as f32 / TARGET_SAMPLE_RATE as f32;
    let silent_ratio =
        samples.iter().filter(|s| s.abs() < 1e-4).count() as f32 / samples.len() as f32;

    println!(
        "SELFTEST result seconds={seconds:.2} peak={:.4} rms={:.4} silent_ratio={silent_ratio:.3} dropped={dropped}",
        level.peak, level.rms
    );

    if level.rms < 1e-4 {
        return Err("captured audio is silent - the tap produced no signal".into());
    }

    println!("SELFTEST PASS {}", path.display());
    Ok(())
}

/// Plays the reference recording named by `MARSWIND_SELFTEST_PLAY`, if any.
fn start_playback() -> Result<Option<std::process::Child>, String> {
    let Ok(path) = std::env::var("MARSWIND_SELFTEST_PLAY") else {
        return Ok(None);
    };

    let child = std::process::Command::new("afplay")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("could not play {path}: {e}"))?;
    println!("SELFTEST playing {path}");
    Ok(Some(child))
}

fn source_from_env() -> String {
    std::env::var("MARSWIND_SELFTEST_SOURCE").unwrap_or_else(|_| "system".into())
}

fn output_path(app: &AppHandle, name: &str) -> PathBuf {
    if let Ok(path) = std::env::var("MARSWIND_SELFTEST_OUT") {
        return PathBuf::from(path);
    }
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("recordings")
        .join(name)
}
