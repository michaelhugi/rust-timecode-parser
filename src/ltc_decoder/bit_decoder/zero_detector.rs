use crate::ltc_decoder::bit_decoder::BitDecoder;
use crate::ltc_decoder::bit_decoder::sample_rater::{max_sample_count_for_bit, min_sample_count_for_bit};

/// Can be used to detect when a zero ends.
/// A zero in LTC smpte is defined as a period of one bit without a change of the high or low state
/// of the audio signal. Once detected the end of a zero, it is possible to detect ones two, who are
/// bits with one state change
pub(crate) struct ZeroDetector {
    /// The minimum number of samples without state change to define a zeroe depending on the sample
    /// rate of the audio signal,
    min_sample_count: usize,
    /// The maximum number of samples without state change to define a zero depending on the sample
    /// rate of the audio signal,
    max_sample_count: usize,
    /// During detection tells if the current state is high
    is_high: bool,
    /// During detection tells how many samples have been received since the last state change
    sample_count: usize,
}

impl ZeroDetector {
    /// A new instance of bit detector
    pub(crate) fn new(sample_rate: f64) -> Self {
        let min_sample_count = min_sample_count_for_bit(&sample_rate);
        let max_sample_count = max_sample_count_for_bit(&sample_rate);
        Self {
            min_sample_count: min_sample_count as usize,
            max_sample_count: max_sample_count as usize,
            is_high: false,
            sample_count: 0,
        }
    }
    /// By pushing every received sample, the function will eventually return true when the end of
    /// a 0 (state not changed for a bit duration) is detected. This can be used to set the but
    /// heartbeat during ltc-decoding
    pub(crate) fn is_end_of_zero(&mut self, is_high: &bool) -> bool {
        if self.sample_count == 0 {
            self.is_high = *is_high;
            self.sample_count += 1;
            return false;
        }
        self.sample_count += 1;
        if self.is_state_change(is_high) {
            if self.sample_count >= self.min_sample_count && self.sample_count <= self.max_sample_count {
                self.invalidate();
                return true;
            }
            self.invalidate();
            return false;
        }
        if self.sample_count > self.max_sample_count {
            self.invalidate();
        }
        return false;
    }

    fn is_state_change(&mut self, is_high: &bool) -> bool {
        if &self.is_high != is_high {
            self.is_high = *is_high;
            return true;
        }
        return false;
    }

    /// Resets the zero finder
    pub(crate) fn invalidate(&mut self) {
        self.sample_count = 0;
    }
}