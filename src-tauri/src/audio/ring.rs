//! Fixed-capacity ring buffer holding the most recent audio at the pipeline rate.
//!
//! The capture callback never touches this buffer - it is written by the pump
//! thread after resampling and read by downstream consumers, the VAD and ASR
//! stages. Overwriting the oldest samples is intentional: for live subtitles,
//! stale audio is worthless.

pub struct AudioRing {
    buf: Vec<f32>,
    /// Index of the next slot to write.
    write: usize,
    /// Number of valid samples, saturating at capacity.
    filled: usize,
}

impl AudioRing {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "ring capacity must be non-zero");
        Self {
            buf: vec![0.0; capacity],
            write: 0,
            filled: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.filled
    }

    pub fn is_empty(&self) -> bool {
        self.filled == 0
    }

    pub fn clear(&mut self) {
        self.write = 0;
        self.filled = 0;
    }

    pub fn push_slice(&mut self, samples: &[f32]) {
        let cap = self.buf.len();

        // A burst larger than the whole ring can only leave its tail behind.
        let samples = if samples.len() > cap {
            &samples[samples.len() - cap..]
        } else {
            samples
        };

        let first = (cap - self.write).min(samples.len());
        self.buf[self.write..self.write + first].copy_from_slice(&samples[..first]);
        let rest = samples.len() - first;
        if rest > 0 {
            self.buf[..rest].copy_from_slice(&samples[first..]);
        }

        self.write = (self.write + samples.len()) % cap;
        self.filled = (self.filled + samples.len()).min(cap);
    }

    /// Copies the most recent `n` samples into `out` in chronological order.
    /// Returns how many samples were written.
    pub fn read_last(&self, n: usize, out: &mut Vec<f32>) -> usize {
        let n = n.min(self.filled);
        out.clear();
        out.reserve(n);

        let cap = self.buf.len();
        let start = (self.write + cap - n) % cap;
        let first = (cap - start).min(n);
        out.extend_from_slice(&self.buf[start..start + first]);
        if n > first {
            out.extend_from_slice(&self.buf[..n - first]);
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_back_in_chronological_order() {
        let mut ring = AudioRing::new(4);
        ring.push_slice(&[1.0, 2.0, 3.0]);

        let mut out = Vec::new();
        assert_eq!(ring.read_last(3, &mut out), 3);
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn overwrites_oldest_samples_when_full() {
        let mut ring = AudioRing::new(4);
        ring.push_slice(&[1.0, 2.0, 3.0]);
        ring.push_slice(&[4.0, 5.0]);

        assert_eq!(ring.len(), 4);
        let mut out = Vec::new();
        ring.read_last(4, &mut out);
        assert_eq!(out, vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn keeps_tail_of_oversized_burst() {
        let mut ring = AudioRing::new(3);
        ring.push_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let mut out = Vec::new();
        ring.read_last(3, &mut out);
        assert_eq!(out, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn read_last_clamps_to_available() {
        let mut ring = AudioRing::new(8);
        ring.push_slice(&[1.0, 2.0]);

        let mut out = Vec::new();
        assert_eq!(ring.read_last(5, &mut out), 2);
        assert_eq!(out, vec![1.0, 2.0]);
    }
}
