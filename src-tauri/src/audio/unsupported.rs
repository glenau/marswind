//! Capture backend for platforms that do not have one yet.
//!
//! Windows will use WASAPI loopback and Linux PipeWire; neither is written. The
//! app still builds and runs on both - everything above capture is
//! platform-independent - and only capture reports that it is unavailable.

use super::{AudioError, CaptureFormat, SampleSink, SourceInfo, SourceSelector};

pub fn list_sources() -> Result<Vec<SourceInfo>, AudioError> {
    Err(AudioError::Unsupported)
}

pub fn source_name(_selector: &SourceSelector) -> Result<String, AudioError> {
    Err(AudioError::Unsupported)
}

pub struct Capture;

impl Capture {
    pub fn start(
        _selector: &SourceSelector,
        _sink: SampleSink,
    ) -> Result<(Self, CaptureFormat), AudioError> {
        Err(AudioError::Unsupported)
    }
}
