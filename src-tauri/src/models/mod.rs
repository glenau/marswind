//! Downloading and storing the models the app runs on.
//!
//! Models live in the app data directory, never in the installer: shipping a
//! 1.6 GB binary would make every update a 1.6 GB update, and most users only
//! need one model.

pub mod catalog;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use parking_lot::Mutex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

pub use catalog::{ModelKind, ModelSpec, VAD_MODEL_ID};

const PROGRESS_EVENT: &str = "models://progress";
const PROGRESS_INTERVAL: Duration = Duration::from_millis(200);
/// Suffix for a download in flight. A partial file must never be mistaken for
/// an installed model.
const PARTIAL_SUFFIX: &str = ".part";

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("unknown model '{0}'")]
    Unknown(String),
    #[error("model '{0}' is not installed")]
    NotInstalled(String),
    #[error("download failed: {0}")]
    Download(String),
    #[error("the downloaded file does not match its published checksum - nothing was installed")]
    ChecksumMismatch,
    #[error("download cancelled")]
    Cancelled,
    #[error("{0}")]
    Io(String),
}

impl Serialize for ModelError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub name: String,
    pub note: String,
    pub kind: ModelKind,
    pub size_bytes: u64,
    pub installed: bool,
    pub downloading: bool,
    pub recommended: bool,
    /// The terms the weights come under, and where they are stated. Shown on
    /// the row, because the download button is where a user accepts them.
    pub license: String,
    pub license_url: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    id: String,
    downloaded_bytes: u64,
    total_bytes: u64,
    done: bool,
    error: Option<String>,
}

pub struct ModelStore {
    dir: PathBuf,
    in_flight: Mutex<HashSet<String>>,
    cancelled: Mutex<HashSet<String>>,
}

impl ModelStore {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            in_flight: Mutex::new(HashSet::new()),
            cancelled: Mutex::new(HashSet::new()),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.dir
    }

    pub fn path_for(&self, spec: &ModelSpec) -> PathBuf {
        self.dir.join(spec.file_name)
    }

    /// A model counts as installed when the file is there at exactly the
    /// published size. Full hashing on every check would mean reading gigabytes
    /// to draw a list.
    pub fn is_installed(&self, spec: &ModelSpec) -> bool {
        std::fs::metadata(self.path_for(spec))
            .map(|m| m.is_file() && m.len() == spec.size_bytes)
            .unwrap_or(false)
    }

    /// Path to an installed model, or an error naming what is missing.
    pub fn installed_path(&self, id: &str) -> Result<PathBuf, ModelError> {
        let spec = catalog::find(id).ok_or_else(|| ModelError::Unknown(id.to_string()))?;
        if !self.is_installed(spec) {
            return Err(ModelError::NotInstalled(id.to_string()));
        }
        Ok(self.path_for(spec))
    }

    pub fn list(&self) -> Vec<ModelStatus> {
        let recommended = catalog::recommended_asr(total_memory_bytes());
        let in_flight = self.in_flight.lock();

        catalog::CATALOG
            .iter()
            .map(|spec| ModelStatus {
                id: spec.id.to_string(),
                name: spec.name.to_string(),
                note: spec.note.to_string(),
                kind: spec.kind,
                size_bytes: spec.size_bytes,
                installed: self.is_installed(spec),
                downloading: in_flight.contains(spec.id),
                recommended: spec.id == recommended,
                license: spec.license.to_string(),
                license_url: spec.license_url.to_string(),
            })
            .collect()
    }

    pub fn disk_usage(&self) -> u64 {
        catalog::CATALOG
            .iter()
            .filter(|spec| self.is_installed(spec))
            .map(|spec| spec.size_bytes)
            .sum()
    }

    pub fn remove(&self, id: &str) -> Result<(), ModelError> {
        let spec = catalog::find(id).ok_or_else(|| ModelError::Unknown(id.to_string()))?;
        let path = self.path_for(spec);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| ModelError::Io(e.to_string()))?;
            log::info!("removed model {id}");
        }
        Ok(())
    }

    pub fn cancel(&self, id: &str) {
        self.cancelled.lock().insert(id.to_string());
    }

    pub async fn download(
        self: &Arc<Self>,
        id: &str,
        app: AppHandle,
    ) -> Result<PathBuf, ModelError> {
        let spec = catalog::find(id).ok_or_else(|| ModelError::Unknown(id.to_string()))?;

        if self.is_installed(spec) {
            return Ok(self.path_for(spec));
        }
        if !self.in_flight.lock().insert(spec.id.to_string()) {
            return Err(ModelError::Download("already downloading".into()));
        }
        self.cancelled.lock().remove(spec.id);

        let result = self.download_inner(spec, &app).await;

        self.in_flight.lock().remove(spec.id);
        self.cancelled.lock().remove(spec.id);

        if let Err(e) = &result {
            // Never leave a partial file behind to be misread later.
            let _ = std::fs::remove_file(self.partial_path(spec));
            let _ = app.emit(
                PROGRESS_EVENT,
                ProgressEvent {
                    id: spec.id.to_string(),
                    downloaded_bytes: 0,
                    total_bytes: spec.size_bytes,
                    done: true,
                    error: Some(e.to_string()),
                },
            );
        }

        result
    }

    async fn download_inner(
        &self,
        spec: &ModelSpec,
        app: &AppHandle,
    ) -> Result<PathBuf, ModelError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| ModelError::Io(e.to_string()))?;

        let partial = self.partial_path(spec);
        let final_path = self.path_for(spec);

        log::info!("downloading {} from {}", spec.id, spec.url);

        let response = reqwest::get(spec.url)
            .await
            .map_err(|e| ModelError::Download(e.to_string()))?;
        if !response.status().is_success() {
            return Err(ModelError::Download(format!(
                "server responded {}",
                response.status()
            )));
        }

        let total = response.content_length().unwrap_or(spec.size_bytes);
        let mut file = tokio::fs::File::create(&partial)
            .await
            .map_err(|e| ModelError::Io(e.to_string()))?;

        let mut hasher = Sha256::new();
        let mut downloaded = 0u64;
        let mut last_emit = Instant::now();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            if self.cancelled.lock().contains(spec.id) {
                return Err(ModelError::Cancelled);
            }

            let chunk = chunk.map_err(|e| ModelError::Download(e.to_string()))?;
            hasher.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|e| ModelError::Io(e.to_string()))?;
            downloaded += chunk.len() as u64;

            if last_emit.elapsed() >= PROGRESS_INTERVAL {
                let _ = app.emit(
                    PROGRESS_EVENT,
                    ProgressEvent {
                        id: spec.id.to_string(),
                        downloaded_bytes: downloaded,
                        total_bytes: total,
                        done: false,
                        error: None,
                    },
                );
                last_emit = Instant::now();
            }
        }

        file.flush()
            .await
            .map_err(|e| ModelError::Io(e.to_string()))?;
        drop(file);

        let digest: String = hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        if digest != spec.sha256 {
            log::error!(
                "checksum mismatch for {}: expected {}, got {digest}",
                spec.id,
                spec.sha256
            );
            return Err(ModelError::ChecksumMismatch);
        }

        std::fs::rename(&partial, &final_path).map_err(|e| ModelError::Io(e.to_string()))?;
        log::info!("installed model {} ({} bytes)", spec.id, downloaded);

        let _ = app.emit(
            PROGRESS_EVENT,
            ProgressEvent {
                id: spec.id.to_string(),
                downloaded_bytes: downloaded,
                total_bytes: total,
                done: true,
                error: None,
            },
        );

        Ok(final_path)
    }

    fn partial_path(&self, spec: &ModelSpec) -> PathBuf {
        self.dir.join(format!("{}{PARTIAL_SUFFIX}", spec.file_name))
    }
}

/// Physical memory, used to pick a sensible default model.
pub fn total_memory_bytes() -> u64 {
    const FALLBACK: u64 = 8 * 1024 * 1024 * 1024;

    #[cfg(target_os = "macos")]
    {
        let mut value: u64 = 0;
        let mut size = std::mem::size_of::<u64>();
        let name = c"hw.memsize";
        // SAFETY: the out buffer is exactly the u64 this sysctl returns.
        let status = unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                &mut value as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        };
        if status == 0 && value > 0 {
            return value;
        }
    }

    FALLBACK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> ModelStore {
        let dir = std::env::temp_dir().join(format!("marswind-models-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        ModelStore::new(dir)
    }

    #[test]
    fn a_missing_file_is_not_installed() {
        let store = temp_store("missing");
        let spec = catalog::find("base").unwrap();
        assert!(!store.is_installed(spec));
        assert!(store.installed_path("base").is_err());
        std::fs::remove_dir_all(store.directory()).ok();
    }

    #[test]
    fn a_truncated_file_is_not_installed() {
        let store = temp_store("truncated");
        let spec = catalog::find("base").unwrap();
        std::fs::write(store.path_for(spec), b"not the whole model").unwrap();

        assert!(!store.is_installed(spec));
        std::fs::remove_dir_all(store.directory()).ok();
    }

    #[test]
    fn a_full_size_file_counts_as_installed() {
        let store = temp_store("complete");
        let spec = catalog::find(VAD_MODEL_ID).unwrap();
        std::fs::write(store.path_for(spec), vec![0u8; spec.size_bytes as usize]).unwrap();

        assert!(store.is_installed(spec));
        assert_eq!(store.disk_usage(), spec.size_bytes);
        assert!(store.installed_path(VAD_MODEL_ID).is_ok());
        std::fs::remove_dir_all(store.directory()).ok();
    }

    #[test]
    fn partial_downloads_are_never_mistaken_for_models() {
        let store = temp_store("partial");
        let spec = catalog::find(VAD_MODEL_ID).unwrap();
        std::fs::write(
            store.partial_path(spec),
            vec![0u8; spec.size_bytes as usize],
        )
        .unwrap();

        assert!(!store.is_installed(spec));
        std::fs::remove_dir_all(store.directory()).ok();
    }

    #[test]
    fn removing_a_model_is_idempotent() {
        let store = temp_store("remove");
        assert!(store.remove("base").is_ok());
        assert!(store.remove("base").is_ok());
        assert!(store.remove("nonexistent").is_err());
        std::fs::remove_dir_all(store.directory()).ok();
    }

    #[test]
    fn reports_physical_memory() {
        // Any machine that can build this has more than a gigabyte.
        assert!(total_memory_bytes() > 1024 * 1024 * 1024);
    }
}
