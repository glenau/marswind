//! Tauri commands exposed to the frontend.

use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::asr::{AsrConfig, AsrEngine, AsrError, AsrState};
use crate::audio::{AudioEngine, AudioError, CaptureFormat, CaptureState, SourceInfo};
use crate::history::recorder::Recorder;
use crate::history::{Format, HistoryError, HistoryStore, Meta, Session, SessionSummary};
use crate::models::{catalog, ModelError, ModelStatus, ModelStore};
use crate::samples::{SampleError, SampleInfo, SamplePlayer};
use crate::translate::language::{self, Language};
use crate::translate::{
    Engine, TranslateConfig, TranslateError, TranslateState, TranslationEngine,
};

#[tauri::command]
pub fn list_audio_sources(engine: State<'_, AudioEngine>) -> Result<Vec<SourceInfo>, AudioError> {
    engine.list_sources()
}

#[tauri::command]
pub fn start_capture(
    app: AppHandle,
    engine: State<'_, AudioEngine>,
    source_id: String,
) -> Result<CaptureFormat, AudioError> {
    engine.start(app, &source_id)
}

#[tauri::command]
pub fn stop_capture(
    engine: State<'_, AudioEngine>,
    asr: State<'_, AsrEngine>,
    translation: State<'_, TranslationEngine>,
) -> Result<(), AudioError> {
    // Recognition and translation without audio are just loaded models burning
    // memory.
    let _ = translation.stop();
    let _ = asr.stop();
    engine.stop()
}

#[tauri::command]
pub fn capture_state(engine: State<'_, AudioEngine>) -> CaptureState {
    engine.state()
}

// Writing captured audio to a file is deliberately *not* a command. The one
// honest check that capture hears what the machine is playing needs a
// recording, and `AudioEngine::record_wav` makes one - but it is reached only
// through `MARSWIND_SELFTEST=capture:<sec>`, which is a developer running
// tests/run-capture.sh and not a window a user has open. Exposed here it would
// be the single path by which audio leaves memory, reachable from the frontend,
// and called by nothing.

#[tauri::command]
pub fn list_models(store: State<'_, Arc<ModelStore>>) -> Vec<ModelStatus> {
    store.list()
}

#[tauri::command]
pub fn models_disk_usage(store: State<'_, Arc<ModelStore>>) -> u64 {
    store.disk_usage()
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    store: State<'_, Arc<ModelStore>>,
    model_id: String,
) -> Result<(), ModelError> {
    let store = Arc::clone(&store);
    store.download(&model_id, app).await.map(|_| ())
}

#[tauri::command]
pub fn cancel_download(store: State<'_, Arc<ModelStore>>, model_id: String) {
    store.cancel(&model_id);
}

#[tauri::command]
pub fn remove_model(store: State<'_, Arc<ModelStore>>, model_id: String) -> Result<(), ModelError> {
    store.remove(&model_id)
}

#[tauri::command]
pub fn asr_state(asr: State<'_, AsrEngine>) -> AsrState {
    asr.state()
}

#[tauri::command]
pub fn start_recognition(
    app: AppHandle,
    audio: State<'_, AudioEngine>,
    asr: State<'_, AsrEngine>,
    store: State<'_, Arc<ModelStore>>,
    model_id: String,
    language: Option<String>,
) -> Result<(), AsrError> {
    if !audio.state().running {
        return Err(AsrError::CaptureNotRunning);
    }

    let model_path = store
        .installed_path(&model_id)
        .map_err(|e| AsrError::Other(e.to_string()))?;
    let vad_model_path = store
        .installed_path(catalog::VAD_MODEL_ID)
        .map_err(|e| AsrError::Other(e.to_string()))?;

    asr.start(
        app,
        audio.subscribe(),
        AsrConfig {
            model_path,
            vad_model_path,
            model_id,
            language,
            use_prompt: true,
        },
    )
}

// Recognition and translation have no stop command of their own. The window
// starts all three stages together and stops them together, and `stop_capture`
// already brings the two above it down in order - a second way to stop half the
// pipeline is a state the interface cannot reach and nothing else asks for.
// When there is a control that turns translation off mid-session, this is where
// its command goes.

#[tauri::command]
pub fn list_languages() -> Vec<Language> {
    language::LANGUAGES.to_vec()
}

#[tauri::command]
pub fn translate_state(translation: State<'_, TranslationEngine>) -> TranslateState {
    translation.state()
}

#[tauri::command]
pub fn start_translation(
    app: AppHandle,
    asr: State<'_, AsrEngine>,
    translation: State<'_, TranslationEngine>,
    store: State<'_, Arc<ModelStore>>,
    model_id: String,
    target_language: String,
) -> Result<(), TranslateError> {
    if !asr.state().running {
        return Err(TranslateError::Other(
            "recognition must be running before translation".into(),
        ));
    }

    let target = *language::find(&target_language)
        .ok_or_else(|| TranslateError::UnknownLanguage(target_language.clone()))?;
    let model_path = store
        .installed_path(&model_id)
        .map_err(|e| TranslateError::Other(e.to_string()))?;
    let prompt = catalog::find(&model_id)
        .map(|spec| spec.prompt.as_str())
        .ok_or_else(|| TranslateError::Other(format!("unknown model '{model_id}'")))?;

    translation.start(
        app,
        asr.subscribe(),
        TranslateConfig {
            engine: Engine::Llm,
            model_path,
            model_id,
            prompt,
            target,
        },
    )
}

// ---------------------------------------------------------------- sessions

/// Begins recording. `started_at` comes from the frontend because that is where
/// a clock that knows the user's locale and time zone lives; the id is made
/// here, from the epoch second, so files sort by time on their name alone.
// The argument list of a command is its wire format: every name here is a key
// the frontend sends. Folding six of them into a struct would only move the
// same six names one level down and add a wrapper on both sides.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn start_session(
    app: AppHandle,
    recorder: State<'_, Recorder>,
    started_at: String,
    source: String,
    asr_model: String,
    spoken_language: String,
    mt_model: String,
    target_language: String,
) -> Result<String, HistoryError> {
    recorder.start(
        &app,
        started_at,
        Meta {
            source,
            asr_model,
            spoken_language,
            mt_model,
            target_language,
        },
    )
}

#[tauri::command]
pub fn stop_session(
    app: AppHandle,
    recorder: State<'_, Recorder>,
) -> Result<Option<String>, HistoryError> {
    recorder.stop(&app)
}

#[tauri::command]
pub fn list_sessions(
    store: State<'_, Arc<HistoryStore>>,
) -> Result<Vec<SessionSummary>, HistoryError> {
    store.list()
}

#[tauri::command]
pub fn read_session(
    store: State<'_, Arc<HistoryStore>>,
    id: String,
) -> Result<Session, HistoryError> {
    store.read(&id)
}

#[tauri::command]
pub fn remove_session(store: State<'_, Arc<HistoryStore>>, id: String) -> Result<(), HistoryError> {
    store.remove(&id)
}

/// Writes a session out beside the others and returns the file. Exporting into
/// the app's own directory rather than asking for a location keeps this to one
/// click; the folder is one button away.
#[tauri::command]
pub fn export_session(
    store: State<'_, Arc<HistoryStore>>,
    id: String,
    format: Format,
) -> Result<String, HistoryError> {
    let to = store
        .directory()
        .join("exports")
        .join(format!("{id}.{}", format.extension()));
    let written = store.export(&id, format, &to)?;
    Ok(written.display().to_string())
}

// ----------------------------------------------------------------- samples

#[tauri::command]
pub fn list_samples(app: AppHandle, player: State<'_, SamplePlayer>) -> Vec<SampleInfo> {
    player.list(&app)
}

#[tauri::command]
pub fn play_sample(
    app: AppHandle,
    player: State<'_, SamplePlayer>,
    id: String,
) -> Result<(), SampleError> {
    player.play(&app, &id)
}

#[tauri::command]
pub fn stop_sample(player: State<'_, SamplePlayer>) {
    player.stop()
}

#[tauri::command]
pub fn playing_sample(player: State<'_, SamplePlayer>) -> Option<String> {
    player.playing()
}
