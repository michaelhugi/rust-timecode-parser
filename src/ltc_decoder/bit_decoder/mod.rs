#![no_std]

use crate::ltc_decoder::bit_decoder::sample_bounds::SampleBounds;
use crate::ltc_decoder::bit_decoder::sample_rater::*;
use crate::ltc_decoder::bit_decoder::zero_detector::ZeroDetector;
use crate::ltc_decoder::print_decoder::AudioImage;
use crate::ltc_decoder::Sample;

mod sample_bounds;
mod zero_detector;
mod sample_rater;
mod threshold_cross_detector;

/// Reads sample by sample, detects the heartbeat of bits in ltc stream and returns 0s and 1s
pub(crate) struct BitDecoder<T: Sample> {
    /// Tells how many samples per bit are at least expected
    min_number_of_samples_per_bit: usize,
    /// Tells how many samples per bit are at most expected
    max_number_of_samples_per_bit: usize,
    /// Tells how many samples per half-bit are at least expected
    min_number_of_samples_per_half_bit: usize,
    /// Tells how many samples per half-bit are at most expected
    max_number_of_samples_per_half_bit: usize,
    /// Helps to define what audio level is considered high and low
    sample_bounds: SampleBounds<T>,
    /// Helps to detect the end of a zero. The end of the zero defines the heartbeat of bits
    zero_detector: ZeroDetector,
    /// Tells if the heartbeat is in sync (detected by zero_detector)
    heartbeat_in_sync: bool,
    /// The current sample count into the bit
    sample_count_in_bit: usize,
    /// The current state of the received audio signal
    is_high: bool,
    /// Tells if a state change in the middle of the bit is currently detected. Would be a 0
    has_change_in_middle_bit: bool,
}


impl<T: Sample> BitDecoder<T> {
    /// Constructor
    pub(crate) fn new(sample_rate: f64) -> Self {
        Self {
            max_number_of_samples_per_bit: max_sample_count_for_bit(&sample_rate),
            min_number_of_samples_per_bit: min_sample_count_for_bit(&sample_rate),
            max_number_of_samples_per_half_bit: max_sample_count_for_halfbit(&sample_rate),
            min_number_of_samples_per_half_bit: min_sample_count_for_halfbit(&sample_rate),
            sample_bounds: SampleBounds::new(),
            zero_detector: ZeroDetector::new(sample_rate),
            heartbeat_in_sync: false,
            sample_count_in_bit: 0,
            is_high: false,
            has_change_in_middle_bit: false,
        }
    }
    /// If anything unexpected is received from audio, invalidate will reset the bit detector to
    /// prevent reading wrong data if the audio timecode is not clear
    fn invalidate(&mut self) {
        self.sample_bounds.invalidate();
        self.zero_detector.invalidate();
        self.heartbeat_in_sync = false;
        self.sample_count_in_bit = 0;
        self.has_change_in_middle_bit = false;
    }
    /// Every audio sample-point that is received is pushed in this function. It will return if a bit
    /// is detected by returning true (1) or false (0)
    /// The function feeds and handles detection of audio-level for high and low as well as bit-heartbeat detection
    pub(crate) fn push_sample(&mut self, sample: T, index: usize, images: &mut [AudioImage]) -> Option<bool> {
        images.iter_mut().for_each(|image| {
            image.push_threashold(index, self.sample_bounds.get_threshold())
        });
        if let Some(is_high) = self.sample_bounds.is_high(sample) {
            // A sample-level (high/low) is detected by sample_bounds.
            let (bit, has_error) = self.handle_received_level(is_high);
            images.iter_mut().for_each(|image| {
                image.push_bit(index, bit);
                if has_error != 0 {
                    image.push_error(index, has_error);
                }
            });
            bit
        } else {
            // sample_bounds is currently not able to tell if a sample is high or low. Continue to push samples in the sample_bounds to detect
            None
        }
    }
    /// Handles an audio sample point that was detected as high or low
    fn handle_received_level(&mut self, is_high: bool) -> (Option<bool>, usize) {
        if !self.heartbeat_in_sync {
            if !self.zero_detector.is_end_of_zero(&is_high) {
                return (None, 0);
            }
            self.heartbeat_in_sync = true;
            self.reset_to_start_of_bit();
            self.is_high = is_high;
        }
        self.sample_count_in_bit += 1;
        let state_changed = self.is_state_change(&is_high);
        match self.next_expected_event() {
            ExpectedEvents::MustBeSteadyOne => {
                if state_changed {
                    self.invalidate();
                    return (None, 1);
                }
                (None, 0)
            }
            ExpectedEvents::MustBeSteadyTwo => {
                if state_changed {
                    self.invalidate();
                    return (None, 2);
                }
                (None, 0)
            }
            ExpectedEvents::CanChangeInMiddle => {
                if state_changed {
                    self.has_change_in_middle_bit = true;
                }
                (None, 0)
            }
            ExpectedEvents::CanChangeInEnd => {
                if state_changed {
                    let bit = self.has_change_in_middle_bit;
                    self.reset_to_start_of_bit();
                    (Some(bit), 0)
                } else { (None, 0) }
            }
            ExpectedEvents::Overdue => {
                self.invalidate();
                (None, 3)
            }
        }
    }

    /// Sets all values to receive the next sample as the first sample of a new bit
    fn reset_to_start_of_bit(&mut self) {
        self.sample_count_in_bit;
        self.has_change_in_middle_bit = false;
    }

    /// Tells if the state changed to previous one and saves current state as previous
    fn is_state_change(&mut self, is_high: &bool) -> bool {
        if &self.is_high != is_high {
            self.is_high = *is_high;
            true
        } else { false }
    }
    /// Tells what event is expected by the received sample-point
    fn next_expected_event(&self) -> ExpectedEvents {
        if self.sample_count_in_bit < self.min_number_of_samples_per_half_bit {
            return ExpectedEvents::MustBeSteadyOne;
        }
        if self.sample_count_in_bit <= self.max_number_of_samples_per_half_bit {
            if self.has_change_in_middle_bit {
                return ExpectedEvents::MustBeSteadyTwo;
            }
            return ExpectedEvents::CanChangeInMiddle;
        }
        if self.sample_count_in_bit < self.min_number_of_samples_per_bit {
            return ExpectedEvents::MustBeSteadyTwo;
        }
        if self.sample_count_in_bit <= self.max_number_of_samples_per_bit {
            return ExpectedEvents::CanChangeInEnd;
        }
        return ExpectedEvents::Overdue;
    }
}


/// When analyzing sample by sample, this tells what event the BitDecoder is waiting for
enum ExpectedEvents {
    /// The next received sample must have the same state as te original
    MustBeSteadyOne,
    // TODO delete
    MustBeSteadyTwo,
    /// The next received sample might change the state in the middle of the bit indicating a 1
    CanChangeInMiddle,
    /// The next received sample might change the state indicating the end of the bit
    CanChangeInEnd,
    /// The end of the bit should have been detected by now. Something went wrong -> Invalidate
    Overdue,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_testing() {}
}