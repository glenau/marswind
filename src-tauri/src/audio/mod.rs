//! System audio capture pipeline.
//!
//! Audio flows: OS callback -> lock-free queue -> pump thread (downmix already
//! done, resample to 16 kHz) -> ring buffer + level meter + optional recorder.
//! Recognition subscribes to the ring buffer and nothing about this layer knows
//! that it does.

pub mod level;
pub mod resample;
pub mod ring;

#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(not(target_os = "macos"), path = "unsupported.rs")]
mod platform;

use std::cell::UnsafeCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use parking_lot::Mutex;
use rtrb::{Consumer, Producer, RingBuffer};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use level::Level;
use resample::MonoResampler;
use ring::AudioRing;

/// Sample rate the whole pipeline runs at. Whisper is trained on 16 kHz and
/// resamples internally anyway, so converting once here is strictly cheaper.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// How much recent audio stays available to downstream consumers.
const RING_SECONDS: usize = 30;

/// Capacity of the queue between the OS callback and the pump thread, in
/// samples at the device rate. Roughly two seconds at 96 kHz - deep enough to
/// ride out a scheduling hiccup, shallow enough that a stalled pump is noticed.
const QUEUE_CAPACITY: usize = 192_000;

/// Level events are emitted at this rate. Faster looks no smoother and just
/// wakes the UI thread for nothing.
const LEVEL_EVENT_INTERVAL: Duration = Duration::from_millis(50);

/// Blocks a downstream stage may fall behind by before it starts losing audio.
/// At ~10 blocks per second this is several seconds of slack.
const SUBSCRIBER_CAPACITY: usize = 64;

const LEVEL_EVENT: &str = "audio://level";
const RECORDING_EVENT: &str = "audio://recording-finished";

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("system audio capture is not supported on this platform yet")]
    Unsupported,
    #[error("permission to record system audio was denied - grant it in System Settings > Privacy & Security > Audio Recording")]
    PermissionDenied,
    #[error("audio source '{0}' is no longer available")]
    SourceNotFound(String),
    #[error("capture is already running")]
    AlreadyRunning,
    #[error("capture is not running")]
    NotRunning,
    #[error("{op} failed with Core Audio status {code} ({fourcc})")]
    Os {
        op: String,
        code: i32,
        fourcc: String,
    },
    #[error("{0}")]
    Other(String),
}

impl Serialize for AudioError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// Everything the machine plays.
    System,
    /// A single application.
    Process,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    /// Stable identifier passed back to `start`: `system` or `process:<id>`.
    pub id: String,
    pub name: String,
    /// Secondary line for the picker, typically the bundle identifier.
    pub detail: Option<String>,
    pub kind: SourceKind,
    /// Whether this process is currently playing audio.
    pub active: bool,
}

/// What the user picked, after parsing the id.
#[derive(Debug, Clone)]
pub enum SourceSelector {
    System,
    Process(u32),
}

impl SourceSelector {
    pub fn parse(id: &str) -> Result<Self, AudioError> {
        if id == "system" {
            return Ok(Self::System);
        }
        id.strip_prefix("process:")
            .and_then(|rest| rest.parse::<u32>().ok())
            .map(Self::Process)
            .ok_or_else(|| AudioError::SourceNotFound(id.to_string()))
    }
}

/// Format the device actually gave us, which is not necessarily what we asked
/// for.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureState {
    pub running: bool,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub format: Option<CaptureFormat>,
    /// Samples lost because the pump thread could not keep up. Anything other
    /// than zero here is a bug worth chasing.
    pub dropped_samples: u64,
    pub recording: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LevelEvent {
    peak: f32,
    rms: f32,
    dropped_samples: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingFinishedEvent {
    path: String,
    seconds: f32,
}

/// Write end of the queue feeding the pump thread, handed to the OS callback.
///
/// The callback runs on a real-time audio thread where allocating or blocking
/// causes audible glitches, so this deliberately avoids both: samples are
/// downmixed in place and pushed into a lock-free queue, and an overrun bumps a
/// counter instead of waiting for room.
pub struct SampleSink {
    producer: UnsafeCell<Producer<f32>>,
    dropped: Arc<AtomicU64>,
}

// SAFETY: exactly one thread - the OS audio callback - ever touches the
// producer. The engine hands the sink to the backend before capture starts and
// drops it after capture stops, so no other thread can observe it while the
// callback is live.
unsafe impl Send for SampleSink {}
unsafe impl Sync for SampleSink {}

impl SampleSink {
    fn new(producer: Producer<f32>, dropped: Arc<AtomicU64>) -> Self {
        Self {
            producer: UnsafeCell::new(producer),
            dropped,
        }
    }

    /// Downmixes an interleaved frame block to mono and queues it.
    pub fn push_interleaved(&self, data: &[f32], channels: usize) {
        if channels == 0 || data.is_empty() {
            return;
        }

        // SAFETY: see the Send/Sync justification above.
        let producer = unsafe { &mut *self.producer.get() };

        let mut lost = 0u64;
        if channels == 1 {
            for &sample in data {
                if producer.push(sample).is_err() {
                    lost += 1;
                }
            }
        } else {
            let scale = 1.0 / channels as f32;
            for frame in data.chunks_exact(channels) {
                let mono = frame.iter().sum::<f32>() * scale;
                if producer.push(mono).is_err() {
                    lost += 1;
                }
            }
        }

        if lost > 0 {
            self.dropped.fetch_add(lost, Ordering::Relaxed);
        }
    }

    /// Mixes one buffer per channel down to mono and queues it. Kept separate
    /// from the interleaved path so neither needs a scratch allocation.
    pub fn push_planar(&self, planes: &[&[f32]]) {
        if planes.is_empty() {
            return;
        }

        let frames = planes.iter().map(|p| p.len()).min().unwrap_or(0);
        if frames == 0 {
            return;
        }

        // SAFETY: see the Send/Sync justification above.
        let producer = unsafe { &mut *self.producer.get() };

        let scale = 1.0 / planes.len() as f32;
        let mut lost = 0u64;
        for frame in 0..frames {
            let mut sum = 0.0;
            for plane in planes {
                sum += plane[frame];
            }
            if producer.push(sum * scale).is_err() {
                lost += 1;
            }
        }

        if lost > 0 {
            self.dropped.fetch_add(lost, Ordering::Relaxed);
        }
    }
}

struct Recorder {
    samples: Vec<f32>,
    remaining: usize,
    path: PathBuf,
}

struct Running {
    /// Platform capture handle. Dropping it tears down the OS resources.
    _capture: platform::Capture,
    stop: Arc<AtomicBool>,
    pump: Option<JoinHandle<()>>,
    source_id: String,
    source_name: String,
    format: CaptureFormat,
}

/// Downstream stages - recognition today, anything else later. Blocks are
/// `Arc<[f32]>` so one is handed to every stage without being copied per stage.
type Subscribers = Arc<Mutex<Vec<Sender<Arc<[f32]>>>>>;

pub struct AudioEngine {
    running: Mutex<Option<Running>>,
    ring: Arc<Mutex<AudioRing>>,
    recorder: Arc<Mutex<Option<Recorder>>>,
    dropped: Arc<AtomicU64>,
    /// Every resampled block goes to each of these.
    subscribers: Subscribers,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(None),
            ring: Arc::new(Mutex::new(AudioRing::new(
                RING_SECONDS * TARGET_SAMPLE_RATE as usize,
            ))),
            recorder: Arc::new(Mutex::new(None)),
            dropped: Arc::new(AtomicU64::new(0)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn list_sources(&self) -> Result<Vec<SourceInfo>, AudioError> {
        platform::list_sources()
    }

    /// Receives every block of 16 kHz mono audio from now on. Dropping the
    /// receiver unsubscribes.
    pub fn subscribe(&self) -> Receiver<Arc<[f32]>> {
        let (sender, receiver) = bounded(SUBSCRIBER_CAPACITY);
        self.subscribers.lock().push(sender);
        receiver
    }

    pub fn start(&self, app: AppHandle, source_id: &str) -> Result<CaptureFormat, AudioError> {
        let mut running = self.running.lock();
        if running.is_some() {
            return Err(AudioError::AlreadyRunning);
        }

        let selector = SourceSelector::parse(source_id)?;
        let source_name = platform::source_name(&selector)?;

        self.dropped.store(0, Ordering::Relaxed);
        self.ring.lock().clear();

        let (producer, consumer) = RingBuffer::<f32>::new(QUEUE_CAPACITY);
        let sink = SampleSink::new(producer, Arc::clone(&self.dropped));

        let (capture, format) = platform::Capture::start(&selector, sink)?;

        let stop = Arc::new(AtomicBool::new(false));
        let pump = spawn_pump(
            consumer,
            format,
            Arc::clone(&self.ring),
            Arc::clone(&self.recorder),
            Arc::clone(&self.dropped),
            Arc::clone(&self.subscribers),
            Arc::clone(&stop),
            app,
        )?;

        log::info!(
            "capture started: source={source_id} name={source_name} rate={} channels={}",
            format.sample_rate,
            format.channels
        );

        *running = Some(Running {
            _capture: capture,
            stop,
            pump: Some(pump),
            source_id: source_id.to_string(),
            source_name,
            format,
        });

        Ok(format)
    }

    pub fn stop(&self) -> Result<(), AudioError> {
        let mut guard = self.running.lock();
        let Some(mut running) = guard.take() else {
            return Err(AudioError::NotRunning);
        };

        running.stop.store(true, Ordering::Relaxed);
        if let Some(pump) = running.pump.take() {
            let _ = pump.join();
        }
        // `running` drops here, tearing down the tap and aggregate device.

        *self.recorder.lock() = None;
        log::info!("capture stopped");
        Ok(())
    }

    pub fn state(&self) -> CaptureState {
        let running = self.running.lock();
        CaptureState {
            running: running.is_some(),
            source_id: running.as_ref().map(|r| r.source_id.clone()),
            source_name: running.as_ref().map(|r| r.source_name.clone()),
            format: running.as_ref().map(|r| r.format),
            dropped_samples: self.dropped.load(Ordering::Relaxed),
            recording: self.recorder.lock().is_some(),
        }
    }

    /// Captures the next `seconds` of audio to a WAV file. Used to verify by ear
    /// that what the app hears matches what the machine is playing.
    pub fn record_wav(&self, seconds: f32, path: PathBuf) -> Result<PathBuf, AudioError> {
        if self.running.lock().is_none() {
            return Err(AudioError::NotRunning);
        }
        if !(0.1..=300.0).contains(&seconds) {
            return Err(AudioError::Other(
                "recording length must be between 0.1 and 300 seconds".into(),
            ));
        }

        let frames = (seconds * TARGET_SAMPLE_RATE as f32) as usize;
        *self.recorder.lock() = Some(Recorder {
            samples: Vec::with_capacity(frames),
            remaining: frames,
            path: path.clone(),
        });

        Ok(path)
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_pump(
    mut consumer: Consumer<f32>,
    format: CaptureFormat,
    ring: Arc<Mutex<AudioRing>>,
    recorder: Arc<Mutex<Option<Recorder>>>,
    dropped: Arc<AtomicU64>,
    subscribers: Subscribers,
    stop: Arc<AtomicBool>,
    app: AppHandle,
) -> Result<JoinHandle<()>, AudioError> {
    let mut resampler = MonoResampler::new(format.sample_rate, TARGET_SAMPLE_RATE)?;

    std::thread::Builder::new()
        .name("marswind-audio-pump".into())
        .spawn(move || {
            let mut staging: Vec<f32> = Vec::with_capacity(QUEUE_CAPACITY / 8);
            let mut resampled: Vec<f32> = Vec::with_capacity(4096);
            let mut last_event = Instant::now();
            let mut pending_level = Level::default();

            while !stop.load(Ordering::Relaxed) {
                let available = consumer.slots();
                if available == 0 {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                }

                staging.clear();
                match consumer.read_chunk(available) {
                    Ok(chunk) => {
                        let (first, second) = chunk.as_slices();
                        staging.extend_from_slice(first);
                        staging.extend_from_slice(second);
                        chunk.commit_all();
                    }
                    Err(e) => {
                        log::warn!("audio queue read failed: {e}");
                        continue;
                    }
                }

                resampled.clear();
                if let Err(e) = resampler.push(&staging, &mut resampled) {
                    log::error!("resampling failed, stopping pump: {e}");
                    break;
                }
                if resampled.is_empty() {
                    continue;
                }

                let level = Level::measure(&resampled);
                pending_level = Level {
                    peak: pending_level.peak.max(level.peak),
                    rms: level.rms,
                };

                ring.lock().push_slice(&resampled);
                feed_recorder(&recorder, &resampled, &app);
                broadcast(&subscribers, &resampled);

                if last_event.elapsed() >= LEVEL_EVENT_INTERVAL {
                    let _ = app.emit(
                        LEVEL_EVENT,
                        LevelEvent {
                            peak: pending_level.peak_normalized(),
                            rms: pending_level.rms_normalized(),
                            dropped_samples: dropped.load(Ordering::Relaxed),
                        },
                    );
                    pending_level = Level::default();
                    last_event = Instant::now();
                }
            }
        })
        .map_err(|e| AudioError::Other(format!("could not start audio pump thread: {e}")))
}

/// Hands a block to every downstream stage. A stage that has fallen too far
/// behind loses the block rather than stalling the pipeline, and one that has
/// gone away is forgotten.
fn broadcast(subscribers: &Subscribers, samples: &[f32]) {
    let mut subscribers = subscribers.lock();
    if subscribers.is_empty() {
        return;
    }

    let block: Arc<[f32]> = Arc::from(samples);
    subscribers.retain(|sender| match sender.try_send(Arc::clone(&block)) {
        Ok(()) => true,
        Err(TrySendError::Full(_)) => {
            log::warn!("a downstream audio stage is behind; dropped a block");
            true
        }
        Err(TrySendError::Disconnected(_)) => false,
    });
}

fn feed_recorder(recorder: &Arc<Mutex<Option<Recorder>>>, samples: &[f32], app: &AppHandle) {
    let finished = {
        let mut guard = recorder.lock();
        let Some(rec) = guard.as_mut() else {
            return;
        };

        let take = samples.len().min(rec.remaining);
        rec.samples.extend_from_slice(&samples[..take]);
        rec.remaining -= take;

        if rec.remaining > 0 {
            return;
        }
        guard.take()
    };

    let Some(rec) = finished else { return };
    let seconds = rec.samples.len() as f32 / TARGET_SAMPLE_RATE as f32;

    match write_wav(&rec.path, &rec.samples) {
        Ok(()) => {
            log::info!("wrote {:.1}s of audio to {}", seconds, rec.path.display());
            let _ = app.emit(
                RECORDING_EVENT,
                RecordingFinishedEvent {
                    path: rec.path.to_string_lossy().to_string(),
                    seconds,
                },
            );
        }
        Err(e) => log::error!("failed to write {}: {e}", rec.path.display()),
    }
}

fn write_wav(path: &PathBuf, samples: &[f32]) -> Result<(), AudioError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AudioError::Other(format!("could not create {}: {e}", parent.display()))
        })?;
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| AudioError::Other(format!("could not open wav writer: {e}")))?;
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        writer
            .write_sample((clamped * i16::MAX as f32) as i16)
            .map_err(|e| AudioError::Other(format!("wav write failed: {e}")))?;
    }
    writer
        .finalize()
        .map_err(|e| AudioError::Other(format!("wav finalize failed: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_source_ids() {
        assert!(matches!(
            SourceSelector::parse("system").unwrap(),
            SourceSelector::System
        ));
        assert!(matches!(
            SourceSelector::parse("process:42").unwrap(),
            SourceSelector::Process(42)
        ));
    }

    #[test]
    fn rejects_malformed_source_ids() {
        assert!(SourceSelector::parse("process:").is_err());
        assert!(SourceSelector::parse("process:abc").is_err());
        assert!(SourceSelector::parse("").is_err());
    }

    #[test]
    fn downmixes_stereo_to_mono() {
        let (producer, mut consumer) = RingBuffer::<f32>::new(16);
        let dropped = Arc::new(AtomicU64::new(0));
        let sink = SampleSink::new(producer, Arc::clone(&dropped));

        // Two stereo frames: (1.0, 0.0) and (0.5, 0.5).
        sink.push_interleaved(&[1.0, 0.0, 0.5, 0.5], 2);

        assert_eq!(consumer.pop().unwrap(), 0.5);
        assert_eq!(consumer.pop().unwrap(), 0.5);
        assert_eq!(dropped.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn counts_samples_dropped_on_overrun() {
        let (producer, _consumer) = RingBuffer::<f32>::new(2);
        let dropped = Arc::new(AtomicU64::new(0));
        let sink = SampleSink::new(producer, Arc::clone(&dropped));

        sink.push_interleaved(&[0.1, 0.2, 0.3, 0.4, 0.5], 1);

        assert_eq!(dropped.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn writes_a_readable_wav_file() {
        let dir = std::env::temp_dir().join("marswind-test-wav");
        let path = dir.join("tone.wav");
        let samples: Vec<f32> = (0..1600)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / 16_000.0).sin())
            .collect();

        write_wav(&path, &samples).unwrap();

        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().sample_rate, TARGET_SAMPLE_RATE);
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(reader.len(), 1600);

        std::fs::remove_dir_all(&dir).ok();
    }
}
