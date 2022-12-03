#![no_std]

use std::time::Duration;
use crate::ltc_decoder::bit_decoder::sample_bounds::SampleBounds;
use crate::ltc_decoder::bit_decoder::zero_detector::ZeroDetector;
use crate::ltc_decoder::Sample;

/// State of ThresholdCross of an audio signal if it is after
/// half a bit (Short)
/// a bit (Long)
/// No Threshold Cross (None)
/// An invalid Cross (Invalid) and the state of the whole deparsers should be invalidated
pub(crate) enum ThresholdCross {
    ///No threshold cross detected on sample point
    None,
    /// Invalid threshold cross detected on sample point -> Invalidate parents
    Invalid,
    /// Threshold cross detected for a short period (= half of a 1)
    Short,
    /// Threshold cross detected for a long period (=0)
    Long,
}

#[derive(Default)]
/// Calculates the lenght of a bit / a half-bit and keeps track of it
struct ThresholdCrossState {
    valid: bool,
    unknown_size: usize,
    half_size: usize,
    full_size: usize,
}

impl ThresholdCrossState {
    /// Constructor
    fn new() -> Self {
        let mut s = Self::default();
        s.invalidate();
        s
    }
    /// Returns the ThresholdCross-type after a threshold-cross was detected. The size tells how
    /// many samples were in between two states. If not valid it needs at least one half-bit and
    /// one bit to detected to have a valid state to tell what a half-bit and a bit is
    fn from_cross_size(&mut self, size: usize) -> ThresholdCross {
        if !self.valid {
            // Didn't find a short and a long item yet
            if self.unknown_size == 0 {
                self.unknown_size = size;
                return ThresholdCross::None;
            }
            if Self::is_approx_same(&self.unknown_size, &size) {
                return ThresholdCross::None;
            }
            if Self::is_approx_half(&size, &self.unknown_size) {
                self.half_size = size;
                self.full_size = self.unknown_size;
                self.valid = true;
                return ThresholdCross::Short;
            }
            if Self::is_approx_double(&size, &self.unknown_size) {
                self.half_size = self.unknown_size;
                self.full_size = size;
                self.valid = true;
                return ThresholdCross::Long;
            }
            return ThresholdCross::Invalid;
        }
        if Self::is_approx_same(&size, &self.full_size) {
            return ThresholdCross::Long;
        }
        if Self::is_approx_same(&size, &self.half_size) {
            return ThresholdCross::Short;
        }
        return ThresholdCross::Invalid;
    }
    /// Invalidates the state -> the duration of half-bits and bits will be recalculated until the
    /// structs starts returning cross-types again
    pub(crate) fn invalidate(&mut self) {
        self.valid = false;
        self.half_size = 0;
        self.full_size = 0;
    }
    /// Tells if a value is approximately half to a compared value. Used to determine how long a
    /// half-bit and a bit is
    fn is_approx_half(check: &usize, comp: &usize) -> bool {
        let low = comp / 3;
        let high = (comp * 2) / 3;
        return check > &low && check < &high;
    }
    /// Tells if a value is approximately double to a compared value. Used to determine how long a
    /// half-bit and a bit is
    fn is_approx_double(check: &usize, comp: &usize) -> bool {
        return Self::is_approx_half(comp, check);
    }
    /// Tells if a value is approximately the same to a compared value. Used to determine how long a
    /// half-bit and a bit is
    fn is_approx_same(check: &usize, comp: &usize) -> bool {
        let low = (comp * 4) / 5;
        let high = (comp * 5) / 4;
        return check >= &low && check <= &high;
    }
}

/// The detector takes audio smaples one after another and eventually will return if a half-bit
/// or a bit was detected on a threshold cross.
pub(crate) struct ThresholdCrossDetector<T: Sample> {
    /// Calculates and holds the threshold, when a signal is low or high
    sample_bounds: SampleBounds<T>,
    /// Tells if it's currently counting samples until the next threshold cross
    /// or if it's waiting for sample_bounds or the first threshold-cross to be happening
    counting: bool,
    /// Tells if the last sample was high or low
    is_high: Option<bool>,
    /// If counting, this holds the current count from the last threshold-cross
    count: usize,
    /// Calculates and holds information about how long a half-bit and bit is.
    state: ThresholdCrossState,
}


impl<T: Sample> ThresholdCrossDetector<T> {
    /// Constructor
    pub(crate) fn new() -> Self {
        Self {
            sample_bounds: SampleBounds::new(),
            counting: false,
            is_high: None,
            count: 0,
            state: ThresholdCrossState::new(),
        }
    }

    /// Used to find threshold-crosses. Returns if a bit or a half-bit duration cross has been detected
    pub(crate) fn crosses(&mut self, sample: T) -> ThresholdCross {
        if let Some(is_high) = self.sample_bounds.is_high(sample) {
            if self.is_high.is_none() {
                // Initial setting of current is-high
                self.is_high = Some(is_high);
                return ThresholdCross::None;
            }
            let changed = self.is_high.unwrap() != is_high;
            if changed {
                self.is_high = Some(is_high);
            }
            if !self.counting {
                if changed {
                    self.counting = true;
                    self.count = 0;
                }
                return ThresholdCross::None;
            }
            self.count += 1;
            if changed {
                let count = self.count;
                self.count = 0;
                return self.state.from_cross_size(count);
            }
            ThresholdCross::None
        } else {
            //Sample bounds does not know the treshold for low and high bits at the moment
            self.invalidate();
            ThresholdCross::None
        }
    }
    /// Used to invalidate the whole decoding system in case unexpected data is received.
    fn invalidate(&mut self) {
        self.counting = false;
        self.is_high = None;
        self.count = 0;
        self.sample_bounds.invalidate();
        self.state.invalidate();
    }

}

#[cfg(test)]
mod tests {
    use crate::ltc_decoder::bit_decoder::threshold_cross_detector::{ThresholdCross, ThresholdCrossDetector, ThresholdCrossState};
    use crate::ltc_decoder::get_test_samples_48_14;

    #[test]
    fn test_threshold_crossfade_detector() {
        let (sample_rate, samples) = get_test_samples_48_14();
        let mut tcd = ThresholdCrossDetector::new();

        for index in 0..3000 {
            match tcd.crosses(samples[index]) {
                ThresholdCross::None => {}
                ThresholdCross::Invalid => panic!("invalid"),
                ThresholdCross::Short => {}
                ThresholdCross::Long => {}
            }
        }
    }
}

#[test]
fn test_is_approx_half() {
    assert!(ThresholdCrossState::is_approx_half(&100, &200));
    assert!(ThresholdCrossState::is_approx_half(&80, &200));
    assert!(ThresholdCrossState::is_approx_half(&120, &200));
    assert!(!ThresholdCrossState::is_approx_half(&150, &200));
    assert!(!ThresholdCrossState::is_approx_half(&50, &200));

    assert!(ThresholdCrossState::is_approx_half(&11, &25));
    assert!(ThresholdCrossState::is_approx_half(&12, &23));
}

#[test]
fn test_is_approx_double() {
    assert!(ThresholdCrossState::is_approx_double(&200, &100));
    assert!(ThresholdCrossState::is_approx_double(&200, &80));
    assert!(ThresholdCrossState::is_approx_double(&200, &120));
    assert!(!ThresholdCrossState::is_approx_double(&200, &150));
    assert!(!ThresholdCrossState::is_approx_double(&200, &50));
}
