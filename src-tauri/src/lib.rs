pub mod asr;
pub mod audio;
mod commands;
pub mod history;
pub mod models;
pub mod samples;
mod selftest;
pub mod translate;

use std::sync::Arc;

use tauri::{Manager, RunEvent, WindowEvent};

use asr::AsrEngine;
use audio::AudioEngine;
use history::recorder::Recorder;
use history::HistoryStore;
use models::ModelStore;
use samples::SamplePlayer;
use translate::TranslationEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AudioEngine::new())
        .manage(AsrEngine::new())
        .manage(TranslationEngine::new())
        .manage(Recorder::new())
        .manage(SamplePlayer::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_audio_sources,
            commands::start_capture,
            commands::stop_capture,
            commands::capture_state,
            commands::list_models,
            commands::models_disk_usage,
            commands::download_model,
            commands::cancel_download,
            commands::remove_model,
            commands::asr_state,
            commands::start_recognition,
            commands::translate_state,
            commands::start_translation,
            commands::list_languages,
            commands::start_session,
            commands::stop_session,
            commands::list_sessions,
            commands::read_session,
            commands::remove_session,
            commands::export_session,
            commands::list_samples,
            commands::play_sample,
            commands::stop_sample,
            commands::playing_sample,
        ])
        .setup(|app| {
            // Without this the window can open behind whatever the user is
            // watching - which is always, since that is the point of the app.
            // Centred before it is shown rather than after: the configuration
            // asks for it too, but that is the primary display, and moving a
            // window the reader can already see is a jump they watch happen.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
            }

            let models_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("models");
            app.manage(Arc::new(ModelStore::new(models_dir)));

            let transcripts_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("transcripts");
            app.manage(Arc::new(HistoryStore::new(transcripts_dir)));

            if let Ok(mode) = std::env::var("MARSWIND_SELFTEST") {
                selftest::spawn(app.handle().clone(), mode);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. }) {
                shut_down(window.app_handle());
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            // The window may never see a close: quitting from the menu, a
            // logout, or `app.exit()` all arrive here instead.
            if matches!(event, RunEvent::ExitRequested { .. }) {
                shut_down(app);
            }
        });
}

/// Brings the pipeline down in order before the process goes.
///
/// Left to run, closing the window while listening crashed the app: the audio
/// tap keeps calling into the ring buffer, and whisper keeps a Metal command
/// buffer in flight, while the managed state they belong to is being dropped
/// around them. Dropping in the right order is not something the runtime can
/// know to do - translation before recognition before capture, because each one
/// is fed by the one after it.
///
/// It also means a session that was being recorded is written out rather than
/// lost, which is what anyone closing the window in the middle of one expects.
fn shut_down(app: &tauri::AppHandle) {
    let _ = app.state::<TranslationEngine>().stop();
    let _ = app.state::<AsrEngine>().stop();
    let _ = app.state::<AudioEngine>().stop();
    app.state::<SamplePlayer>().stop();

    match app.state::<Recorder>().stop(app) {
        Ok(Some(id)) => log::info!("session {id} saved on exit"),
        Ok(None) => {}
        Err(e) => log::warn!("could not save the session on exit: {e}"),
    }
}
