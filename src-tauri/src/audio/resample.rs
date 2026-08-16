//! Sample rate conversion from the device rate down to the pipeline rate.
//!
//! Devices hand us 44.1 or 48 kHz (sometimes 96), whisper wants 16 kHz. The FFT
//! resampler is used rather than a polynomial one because downsampling without
//! an anti-aliasing filter folds high frequencies back into the speech band,
//! and that noise costs recognition accuracy.

use rubato::{FftFixedIn, Resampler};

use super::AudioError;

/// Input frames handed to the resampler at a time. At 48 kHz this is 21 ms -
/// small enough to keep latency negligible, large enough that the FFT is cheap.
const CHUNK_FRAMES: usize = 1024;

pub struct MonoResampler {
    /// `None` when input and output rates match and no conversion is needed.
    inner: Option<FftFixedIn<f32>>,
    /// Input samples not yet forming a full chunk.
    pending: Vec<f32>,
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>,
}

impl MonoResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self, AudioError> {
        if input_rate == 0 || output_rate == 0 {
            return Err(AudioError::Other(format!(
                "invalid sample rates: {input_rate} -> {output_rate}"
            )));
        }

        if input_rate == output_rate {
            return Ok(Self {
                inner: None,
                pending: Vec::new(),
                input: Vec::new(),
                output: Vec::new(),
            });
        }

        let resampler = FftFixedIn::<f32>::new(
            input_rate as usize,
            output_rate as usize,
            CHUNK_FRAMES,
            2,
            1,
        )
        .map_err(|e| AudioError::Other(format!("resampler setup failed: {e}")))?;

        let output_capacity = resampler.output_frames_max();

        Ok(Self {
            inner: Some(resampler),
            pending: Vec::with_capacity(CHUNK_FRAMES * 2),
            input: vec![Vec::with_capacity(CHUNK_FRAMES)],
            output: vec![vec![0.0; output_capacity]],
        })
    }

    /// Feeds input samples and appends whatever complete output they produced.
    /// Samples that do not fill a chunk are held until the next call.
    pub fn push(&mut self, samples: &[f32], out: &mut Vec<f32>) -> Result<(), AudioError> {
        let Some(resampler) = self.inner.as_mut() else {
            out.extend_from_slice(samples);
            return Ok(());
        };

        self.pending.extend_from_slice(samples);

        while self.pending.len() >= CHUNK_FRAMES {
            self.input[0].clear();
            self.input[0].extend_from_slice(&self.pending[..CHUNK_FRAMES]);
            self.pending.drain(..CHUNK_FRAMES);

            let (_, written) = resampler
                .process_into_buffer(&self.input, &mut self.output, None)
                .map_err(|e| AudioError::Other(format!("resampling failed: {e}")))?;

            out.extend_from_slice(&self.output[0][..written]);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_rates_pass_through_untouched() {
        let mut r = MonoResampler::new(16_000, 16_000).unwrap();
        let mut out = Vec::new();
        r.push(&[0.1, 0.2, 0.3], &mut out).unwrap();
        assert_eq!(out, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn downsamples_by_the_rate_ratio() {
        let mut r = MonoResampler::new(48_000, 16_000).unwrap();
        let input = vec![0.0f32; 48_000];
        let mut out = Vec::new();
        r.push(&input, &mut out).unwrap();

        // One second in, roughly a third of a second out. The trailing partial
        // chunk stays buffered and the FFT resampler holds a little more for its
        // overlap, so allow a few percent of slack below the nominal count.
        let expected = 16_000;
        assert!(
            out.len() > expected * 95 / 100 && out.len() <= expected,
            "expected ~{expected} samples, got {}",
            out.len()
        );
    }

    #[test]
    fn partial_chunks_are_buffered_not_dropped() {
        let mut r = MonoResampler::new(48_000, 16_000).unwrap();
        let mut out = Vec::new();

        // Less than one chunk: nothing comes out yet.
        r.push(&vec![0.0f32; 512], &mut out).unwrap();
        assert!(out.is_empty());

        // The rest of the chunk arrives and output appears.
        r.push(&vec![0.0f32; 512], &mut out).unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn preserves_a_sine_wave_frequency() {
        let mut r = MonoResampler::new(48_000, 16_000).unwrap();
        let input: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin())
            .collect();

        let mut out = Vec::new();
        r.push(&input, &mut out).unwrap();

        // A 440 Hz sine keeps its amplitude through a correct resample.
        let peak = out.iter().skip(1000).fold(0.0f32, |a, &s| a.max(s.abs()));
        assert!(peak > 0.9 && peak < 1.1, "peak amplitude was {peak}");
    }

    #[test]
    fn rejects_zero_sample_rate() {
        assert!(MonoResampler::new(0, 16_000).is_err());
    }
}
