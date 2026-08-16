//! Finding out whether a newer Marswind has been released, and fetching it.
//!
//! The app is otherwise silent on the network: models are downloaded once, on
//! request, and after that nothing leaves the machine. An update check is a
//! network request, so it happens **only when somebody presses the button** in
//! Settings. There is no timer, no check on launch, and nothing to turn off,
//! because there is nothing running.
//!
//! What it does not do is replace the running app. That would want
//! `tauri-plugin-updater`, a signing key and a manifest - and with an ad-hoc
//! signature every new build is a new identity to macOS, so a silent swap would
//! still end with the Audio Recording permission being asked for again. Instead
//! the image is downloaded, verified, and shown in Finder: the same two drags a
//! user already knows, with the download and the checksum done for them.
//!
//! The checksum is the point of downloading it here rather than opening the
//! release page in a browser. `scripts/build-dmg.sh` writes a `.dmg.sha256`
//! beside the image and the release carries both, so the file can be checked
//! against the digest the same way a model is. A release without that sidecar
//! is refused rather than trusted.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;

const LATEST_RELEASE: &str = "https://api.github.com/repos/glenau/marswind/releases/latest";
const PROGRESS_EVENT: &str = "update://progress";
const PROGRESS_INTERVAL: Duration = Duration::from_millis(200);
/// GitHub rejects an API request without one, and a request that says who is
/// asking is the polite form anyway.
const USER_AGENT: &str = concat!("Marswind/", env!("CARGO_PKG_VERSION"));
/// The check is a button press, not a background task; a request that hangs
/// should give up rather than leave a spinner running.
const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("could not reach GitHub: {0}")]
    Unreachable(String),
    #[error("GitHub answered {0}")]
    Status(String),
    #[error("GitHub is rate limiting this network. Try again in about {0} minutes.")]
    RateLimited(u64),
    #[error("the latest release has no disk image for this Mac")]
    NoAssetForThisMachine,
    #[error("the latest release published no checksum, so it was not downloaded")]
    NoChecksum,
    #[error("the downloaded image does not match its published checksum - nothing was saved")]
    ChecksumMismatch,
    #[error("{0}")]
    Io(String),
}

impl Serialize for UpdateError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// What the release page says, reduced to what the window needs to show.
///
/// It travels back from the frontend to start the download, so it deserializes
/// too - the alternative is asking GitHub a second time for what the check
/// already read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Without the tag's `v`, so it sits next to the running version.
    pub version: String,
    pub page_url: String,
    pub asset_name: String,
    pub asset_url: String,
    pub checksum_url: String,
    pub size_bytes: u64,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Deserialize, Clone)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    downloaded_bytes: u64,
    total_bytes: u64,
    done: bool,
}

/// Asks GitHub for the latest release and reports one only if it is newer than
/// what is running. `Ok(None)` is the good, common answer.
pub async fn check(current: &str) -> Result<Option<UpdateInfo>, UpdateError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(TIMEOUT)
        .build()
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?;

    let response = client
        .get(LATEST_RELEASE)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?;

    if !response.status().is_success() {
        return Err(rejection(&response));
    }

    // Read as text and parse here rather than turning on reqwest's `json`
    // feature: serde_json is already a direct dependency, and the feature would
    // add a second copy of the same work to the build.
    let body = response
        .text()
        .await
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?;

    select(&body, current, asset_arch())
}

/// Decides what a release page means, given no network.
///
/// Split out from [`check`] so the deciding can be tested and the fetching
/// cannot get in the way. Everything that could be wrong about an update is
/// decided here - is it newer, is there an image for this machine, is there a
/// digest to check it against - and none of it needs a request to exercise.
fn select(body: &str, current: &str, arch: &str) -> Result<Option<UpdateInfo>, UpdateError> {
    let release: Release =
        serde_json::from_str(body).map_err(|e| UpdateError::Unreachable(e.to_string()))?;

    if release.draft || release.prerelease {
        return Ok(None);
    }

    let version = release.tag_name.trim_start_matches('v').to_string();
    if !is_newer(&version, current) {
        return Ok(None);
    }

    // One image per architecture, named by `uname -m`, which spells Apple
    // Silicon differently from Rust's own target constant.
    let suffix = format!("-{arch}.dmg");
    let image = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(&suffix))
        .ok_or(UpdateError::NoAssetForThisMachine)?;

    let checksum_name = format!("{}.sha256", image.name);
    let checksum = release
        .assets
        .iter()
        .find(|a| a.name == checksum_name)
        .ok_or(UpdateError::NoChecksum)?;

    Ok(Some(UpdateInfo {
        version,
        page_url: release.html_url,
        asset_name: image.name.clone(),
        asset_url: image.browser_download_url.clone(),
        checksum_url: checksum.browser_download_url.clone(),
        size_bytes: image.size,
    }))
}

/// Downloads the image into the user's Downloads folder, checking the digest as
/// it goes, and returns the file so the caller can show it in Finder.
pub async fn download(app: AppHandle, info: UpdateInfo) -> Result<PathBuf, UpdateError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?;

    // Fetched first: a checksum that will not load is a reason not to spend a
    // user's bandwidth, and it is a few dozen bytes.
    let expected = fetch_checksum(&client, &info.checksum_url).await?;

    let directory = app
        .path()
        .download_dir()
        .map_err(|e| UpdateError::Io(format!("no Downloads folder: {e}")))?;
    let final_path = directory.join(&info.asset_name);
    let partial = directory.join(format!("{}.part", info.asset_name));

    let response = client
        .get(&info.asset_url)
        .send()
        .await
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?;
    if !response.status().is_success() {
        return Err(rejection(&response));
    }

    let total = response.content_length().unwrap_or(info.size_bytes);
    let mut file = tokio::fs::File::create(&partial)
        .await
        .map_err(|e| UpdateError::Io(e.to_string()))?;

    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut last_emit = Instant::now();
    let mut stream = response.bytes_stream();

    let result = async {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| UpdateError::Unreachable(e.to_string()))?;
            hasher.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|e| UpdateError::Io(e.to_string()))?;
            downloaded += chunk.len() as u64;

            if last_emit.elapsed() >= PROGRESS_INTERVAL {
                let _ = app.emit(
                    PROGRESS_EVENT,
                    ProgressEvent {
                        downloaded_bytes: downloaded,
                        total_bytes: total,
                        done: false,
                    },
                );
                last_emit = Instant::now();
            }
        }
        file.flush()
            .await
            .map_err(|e| UpdateError::Io(e.to_string()))
    }
    .await;

    drop(file);

    if let Err(e) = result {
        let _ = std::fs::remove_file(&partial);
        return Err(e);
    }

    let digest: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if digest != expected {
        log::error!("update checksum mismatch: expected {expected}, got {digest}");
        let _ = std::fs::remove_file(&partial);
        return Err(UpdateError::ChecksumMismatch);
    }

    std::fs::rename(&partial, &final_path).map_err(|e| UpdateError::Io(e.to_string()))?;
    log::info!("update {} saved to {}", info.version, final_path.display());

    let _ = app.emit(
        PROGRESS_EVENT,
        ProgressEvent {
            downloaded_bytes: downloaded,
            total_bytes: total,
            done: true,
        },
    );

    Ok(final_path)
}

/// Turns a refused response into something worth showing.
///
/// The one worth telling apart is the rate limit. GitHub allows 60 unsigned API
/// calls an hour **per address**, and answers 403 rather than 429 when they run
/// out - so an office, a university or a VPN can exhaust it between them and
/// leave somebody staring at "Forbidden" for a thing they did nothing wrong to
/// deserve. The reset time comes back in a header; the only useful thing to do
/// with it is say how long to wait.
fn rejection(response: &reqwest::Response) -> UpdateError {
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
    };

    if header("x-ratelimit-remaining") == Some(0) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let reset = header("x-ratelimit-reset").unwrap_or(now);
        // Rounded up, and never zero: "try again in 0 minutes" is not advice.
        let minutes = reset.saturating_sub(now).div_ceil(60).max(1);
        return UpdateError::RateLimited(minutes);
    }

    UpdateError::Status(response.status().to_string())
}

/// The sidecar `shasum` writes: `<64 hex>  <file name>`.
async fn fetch_checksum(client: &reqwest::Client, url: &str) -> Result<String, UpdateError> {
    let text = client
        .get(url)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?
        .text()
        .await
        .map_err(|e| UpdateError::Unreachable(e.to_string()))?;

    let digest = text.split_whitespace().next().unwrap_or_default();
    if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(UpdateError::NoChecksum);
    }
    Ok(digest.to_ascii_lowercase())
}

/// `uname -m`, which is what `scripts/build-dmg.sh` puts in the file name.
fn asset_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    }
}

/// Compares two `major.minor.patch` strings numerically.
///
/// Nothing here parses build metadata or pre-release suffixes: this project's
/// tags are three numbers, and a comparison that quietly mishandles a shape it
/// was never given is worse than one that treats it as not newer.
fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse(candidate), parse(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

fn parse(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_higher_version_is_newer() {
        assert!(is_newer("0.1.2", "0.1.1"));
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("1.0.0", "0.9.9"));
    }

    #[test]
    fn the_same_version_is_not_an_update() {
        assert!(!is_newer("0.1.1", "0.1.1"));
    }

    #[test]
    fn an_older_version_is_not_an_update() {
        assert!(!is_newer("0.1.0", "0.1.1"));
        assert!(!is_newer("0.9.9", "1.0.0"));
    }

    /// Ten is not one followed by zero. Comparing these as text is the classic
    /// way to tell somebody they are up to date when they are nine versions
    /// behind.
    #[test]
    fn versions_compare_as_numbers_rather_than_text() {
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(is_newer("0.10.0", "0.9.0"));
    }

    /// A tag this code was not designed for must read as "no update" rather
    /// than as a newer release nobody can install.
    #[test]
    fn an_unparseable_version_is_never_newer() {
        assert!(!is_newer("nightly", "0.1.1"));
        assert!(!is_newer("0.1.1-rc.1", "0.1.1"));
        assert!(!is_newer("0.1.1.1", "0.1.1"));
        assert!(!is_newer("", "0.1.1"));
    }

    /// Shaped like what `releases/latest` actually returns, trimmed to the
    /// fields this code reads.
    fn release_json(tag: &str, assets: &[&str]) -> String {
        let assets: Vec<String> = assets
            .iter()
            .map(|name| {
                format!(
                    r#"{{"name":"{name}","browser_download_url":"https://example.invalid/{name}","size":13631488}}"#
                )
            })
            .collect();
        format!(
            r#"{{"tag_name":"{tag}","html_url":"https://example.invalid/releases/{tag}",
                 "draft":false,"prerelease":false,"assets":[{}]}}"#,
            assets.join(",")
        )
    }

    #[test]
    fn a_newer_release_with_both_files_is_an_update() {
        let body = release_json(
            "v0.1.2",
            &[
                "Marswind-0.1.2-arm64.dmg",
                "Marswind-0.1.2-arm64.dmg.sha256",
            ],
        );
        let found = select(&body, "0.1.1", "arm64").unwrap().unwrap();
        assert_eq!(found.version, "0.1.2");
        assert_eq!(found.asset_name, "Marswind-0.1.2-arm64.dmg");
        assert!(found.checksum_url.ends_with(".sha256"));
    }

    /// The image is picked by architecture, not by being the only `.dmg` there.
    /// An Intel Mac handed the Apple Silicon build would download something it
    /// cannot run.
    #[test]
    fn an_image_for_another_machine_is_not_offered() {
        let body = release_json(
            "v0.1.2",
            &[
                "Marswind-0.1.2-arm64.dmg",
                "Marswind-0.1.2-arm64.dmg.sha256",
            ],
        );
        assert!(matches!(
            select(&body, "0.1.1", "x86_64"),
            Err(UpdateError::NoAssetForThisMachine)
        ));
    }

    /// The reason to download inside the app rather than open a browser is that
    /// the file gets checked. A release that published no digest is refused, or
    /// that reason evaporates.
    #[test]
    fn a_release_without_a_checksum_is_refused() {
        let body = release_json("v0.1.2", &["Marswind-0.1.2-arm64.dmg"]);
        assert!(matches!(
            select(&body, "0.1.1", "arm64"),
            Err(UpdateError::NoChecksum)
        ));
    }

    #[test]
    fn the_running_version_is_not_an_update_to_itself() {
        let body = release_json(
            "v0.1.1",
            &[
                "Marswind-0.1.1-arm64.dmg",
                "Marswind-0.1.1-arm64.dmg.sha256",
            ],
        );
        assert!(select(&body, "0.1.1", "arm64").unwrap().is_none());
    }

    /// A draft or a pre-release is something being worked on, and answering
    /// "up to date" is the right thing to tell somebody about it.
    #[test]
    fn drafts_and_prereleases_are_ignored() {
        for flag in ["draft", "prerelease"] {
            let body = release_json(
                "v0.9.0",
                &[
                    "Marswind-0.9.0-arm64.dmg",
                    "Marswind-0.9.0-arm64.dmg.sha256",
                ],
            )
            .replace(&format!(r#""{flag}":false"#), &format!(r#""{flag}":true"#));
            assert!(
                select(&body, "0.1.1", "arm64").unwrap().is_none(),
                "a {flag} was offered as an update"
            );
        }
    }

    #[test]
    fn the_asset_name_uses_the_shell_spelling_of_the_architecture() {
        // `uname -m` says arm64 where Rust says aarch64, and the file on the
        // release is named by the shell.
        assert_ne!(asset_arch(), "aarch64");
        assert!(matches!(asset_arch(), "arm64" | "x86_64"));
    }
}
