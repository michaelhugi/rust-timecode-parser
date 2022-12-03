use std::cmp::{max, min};
use std::ops::Deref;

use crate::ltc_decoder::Sample;

/// When reading audio samples, the SampleBounds calculate what high and low means in the audio signal for detecting LTC
pub(crate) struct SampleBounds<T: Sample> {
    /// Tells, if the last received audio-samples determine a valid high and low status
    valid: bool,
    /// The max value of the last received samples
    max_value: T,
    /// The min value of the last received samples
    min_value: T,
    /// The treshold between high and low value for samples
    threshold: T,
    /// Keeps the received samples
    sample_history: [T; 255],
    /// Received samples since the last recalculation
    received_count: u8,
}

impl<T: Sample> SampleBounds<T> {
    /// Creates a new starter instance of SampleBounds
    pub(crate) fn new() -> SampleBounds<T> {
        Self {
            valid: false,
            max_value: T::zero(),
            min_value: T::zero(),
            threshold: T::zero(),
            sample_history: [T::zero(); 255],
            received_count: 0,
        }
    }
    /// Every received sample should be pushed here for history purposes.
    /// Every 255 samples it will recalculated
    fn push_sample(&mut self, sample: T) {
        self.sample_history.rotate_left(1);
        self.sample_history[0] = sample;
        self.received_count += 1;
        if self.received_count == u8::MAX {
            self.received_count = 0;
            self.recalculate();
        }
    }
    /// Recalculates min_value, max_value and threshold
    pub fn recalculate(&mut self) {
        let mut min_val = self.sample_history.iter().min();
        let mut max_val = self.sample_history.iter().max();
        if min_val.is_none() || max_val.is_none() {
            self.invalidate();
            return;
        }
        let mut min_val = min_val.unwrap().clone();
        let mut max_val = max_val.unwrap().clone();

        self.min_value = min_val;
        self.max_value = max_val;
        self.recalculate_threshold();
    }
    /// Recalculates the threshold from max_value and min_value
    fn recalculate_threshold(&mut self) {
        let max_half = self.max_value.to_i128();
        let min_half = self.min_value.to_i128();
        if min_half.is_none() || max_half.is_none() {
            self.valid = false;
            return;
        }
        let max_half = max_half.unwrap() / 2;
        let min_half = min_half.unwrap() / 2;
        let average_value = T::from_i128(max_half + min_half);

        if average_value.is_none() {
            self.valid = false;
            return;
        }
        self.valid = true;
        self.threshold = average_value.unwrap();
    }
    /// Tells if a sample is high or low. May return None if the state of sample_bounds is not valid
    /// The function stores the sample to calibrate (and recalibrate periodially) what high or low means
    pub(crate) fn is_high(&mut self, sample: T) -> Option<bool> {
        self.push_sample(sample);
        if !self.valid {
            None
        } else {
            Some(self.threshold < sample)
        }
    }
    /// In case of any unexpected event in the audio stream, invalidate helps to reset the system
    /// and start from the beginning again
    pub(crate) fn invalidate(&mut self) {
        self.threshold = T::zero();
        self.max_value = T::zero();
        self.min_value = T::zero();
        self.valid = false;
        self.received_count = 0;
    }
}


#[cfg(test)]
mod tests {
    use crate::ltc_decoder::bit_decoder::sample_bounds::SampleBounds;
    use crate::ltc_decoder::get_test_samples_48_14;

    #[test]
    fn test_recalculate_threshold() {
        let mut b = SampleBounds::<i32>::new();
        b.max_value = 12;
        b.min_value = -8;
        b.recalculate_threshold();
        assert_eq!(b.threshold, 2);
    }

    #[test]
    fn test_recalculate() {
        let mut b = SampleBounds::<i32>::new();
        assert!(!b.valid);
        let mut samples = [0; 255];
        samples[102] = 234;
        samples[23] = -1;
        for sample in samples {
            b.push_sample(sample);
        }
        assert_eq!(b.max_value, 234);
        assert_eq!(b.min_value, -1);
        assert_eq!(b.threshold, 117);
        assert!(b.valid)
    }

    #[test]
    fn test_and_print_counts() {
        let (sampling_rate, samples) = get_test_samples_48_14();
        let mut b = SampleBounds::new();
        let mut is_current_high = false;
        let mut change_count = 0;
        let mut bit_count = 0;
        let mut sample_count = 0;
        let samples_len = samples.len();
        let mut validated = false;
        for sample in samples {
            sample_count += 1;
            if let Some(is_high) = b.is_high(sample) {
                validated = true;
                if is_high != is_current_high {
                    is_current_high = is_high;
                    bit_count += 1;

                    if (change_count < 7 || (change_count > 14 && change_count < 17) || change_count > 27) {
                        let percent = sample_count as f32 / samples_len as f32;
                        println!("{} -> {} -> {}%", change_count, bit_count, percent);
                    }
                    change_count = 0;
                } else {
                    change_count += 1;
                }
            } else {
                if validated {
                    println!("???");
                }
            }
        }
    }
}