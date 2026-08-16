//! Signal level measurement for the UI meter.

/// Everything quieter than this reads as silence on the meter.
const FLOOR_DB: f32 = -60.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct Level {
    pub peak: f32,
    pub rms: f32,
}

impl Level {
    pub fn measure(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut peak = 0.0f32;
        let mut sum_sq = 0.0f64;
        for &s in samples {
            peak = peak.max(s.abs());
            sum_sq += (s as f64) * (s as f64);
        }

        Self {
            peak,
            rms: (sum_sq / samples.len() as f64).sqrt() as f32,
        }
    }

    /// Peak mapped to 0.0..1.0 on a decibel scale, which is what a level meter
    /// should show - a linear amplitude bar spends most of its travel looking
    /// empty.
    pub fn peak_normalized(&self) -> f32 {
        normalize_db(self.peak)
    }

    pub fn rms_normalized(&self) -> f32 {
        normalize_db(self.rms)
    }
}

fn normalize_db(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * amplitude.log10();
    ((db - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_measures_zero() {
        let level = Level::measure(&[0.0; 16]);
        assert_eq!(level.peak, 0.0);
        assert_eq!(level.peak_normalized(), 0.0);
    }

    #[test]
    fn full_scale_measures_one() {
        let level = Level::measure(&[1.0, -1.0, 0.5]);
        assert_eq!(level.peak, 1.0);
        assert!((level.peak_normalized() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rms_of_constant_signal_equals_amplitude() {
        let level = Level::measure(&[0.5; 100]);
        assert!((level.rms - 0.5).abs() < 1e-6);
    }

    #[test]
    fn below_floor_reads_as_silence() {
        // -80 dBFS, well under the meter floor.
        let level = Level::measure(&[0.0001; 8]);
        assert_eq!(level.peak_normalized(), 0.0);
    }

    #[test]
    fn empty_input_is_safe() {
        let level = Level::measure(&[]);
        assert_eq!(level.rms, 0.0);
    }
}
