use crate::ltc_decoder::Sample;

/// Contains the state of received half-bits and bits by ThresholdCrossDetector
enum BitDecoderState {
    /// Waiting for a full-bit to receive to get in sync
    OutOfSync,
    /// Received a full-bit or two half-bits and waiting for a full-bit or half-bit
    BitCompleted,
    /// Half bit received. Waiting for second half-bit -> Otherwise invalidate
    HalfBitReceived,
}

/// Return value possibilites for returning bits of the bit decoder
pub(crate) enum BitVal {
    /// No bit detected after pushing last audio sample
    None,
    /// Invalid state detected-> Invalidate decoder
    Invalid,
    /// True (1)
    True,
    /// False (0)
    False,
}

/// Reads sample by sample, detects the heartbeat of bits in ltc stream and returns 0s and 1s
pub(crate) struct BitDecoder<T: Sample> {
    /// ThresholdCrossDetector returns bits and half-bits.
    threshold_cross_detector: ThresholdCrossDetector<T>,
    /// State holds the current state of received bits and half-bits
    state: BitDecoderState,
}


impl<T: Sample> BitDecoder<T> {
    /// Constructor
    pub(crate) fn new() -> Self {
        Self {
            threshold_cross_detector: ThresholdCrossDetector::new(),
            state: BitDecoderState::OutOfSync,
        }
    }
    /// If anything unexpected is received from audio, invalidate will reset the bit detector to
    /// prevent reading wrong data if the audio timecode is not clear
    pub(crate) fn invalidate(&mut self) {
        self.state = BitDecoderState::OutOfSync;
        self.threshold_cross_detector.invalidate();
    }
    /// Every audio sample-point that is received is pushed in this function. It will return if a bit
    /// is detected by returning true (1) or false (0)
    /// The function feeds and handles detection of audio-level for high and low as well as bit-heartbeat detection
    pub(crate) fn get_bit(&mut self, sample: T) -> BitVal {
        match self.threshold_cross_detector.crosses(sample) {
            ThresholdCross::None => BitVal::None,
            ThresholdCross::Invalid => BitVal::Invalid,
            ThresholdCross::Short => {
                // half bit received
                match self.state {
                    BitDecoderState::OutOfSync => BitVal::None,
                    BitDecoderState::BitCompleted => {
                        self.state = BitDecoderState::HalfBitReceived;
                        BitVal::None
                    }
                    BitDecoderState::HalfBitReceived => {
                        self.state = BitDecoderState::BitCompleted;
                        BitVal::True
                    }
                }
            }
            ThresholdCross::Long => {
                // full bit received
                match self.state {
                    BitDecoderState::OutOfSync => {
                        self.state = BitDecoderState::BitCompleted;
                        BitVal::False
                    }
                    BitDecoderState::BitCompleted => {
                        BitVal::False
                    }
                    BitDecoderState::HalfBitReceived => {
                        // Expected a half-bit in the state of sync
                        BitVal::Invalid
                    }
                }
            }
        }
    }
}


/// When reading audio samples, the SampleBounds calculate what high and low means in the audio signal for detecting LTC
struct SampleBounds<T: Sample> {
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
    fn new() -> SampleBounds<T> {
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
        let min_val = self.sample_history.iter().min();
        let max_val = self.sample_history.iter().max();
        if min_val.is_none() || max_val.is_none() {
            self.invalidate();
            return;
        }
        let min_val = *min_val.unwrap();
        let max_val = *max_val.unwrap();

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
    fn is_high(&mut self, sample: T) -> Option<bool> {
        self.push_sample(sample);
        if !self.valid {
            None
        } else {
            Some(self.threshold < sample)
        }
    }
    /// In case of any unexpected event in the audio stream, invalidate helps to reset the system
    /// and start from the beginning again
    fn invalidate(&mut self) {
        self.threshold = T::zero();
        self.max_value = T::zero();
        self.min_value = T::zero();
        self.valid = false;
        self.received_count = 0;
    }
}


/// State of ThresholdCross of an audio signal if it is after
/// half a bit (Short)
/// a bit (Long)
/// No Threshold Cross (None)
/// An invalid Cross (Invalid) and the state of the whole deparsers should be invalidated
enum ThresholdCross {
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
    fn cross_from_cross_size(&mut self, size: usize) -> ThresholdCross {
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
        ThresholdCross::Invalid
    }
    /// Invalidates the state -> the duration of half-bits and bits will be recalculated until the
    /// structs starts returning cross-types again
    fn invalidate(&mut self) {
        self.unknown_size = 0;
        self.valid = false;
        self.half_size = 0;
        self.full_size = 0;
    }
    /// Tells if a value is approximately half to a compared value. Used to determine how long a
    /// half-bit and a bit is
    fn is_approx_half(check: &usize, comp: &usize) -> bool {
        let low = comp / 3;
        let high = (comp * 2) / 3;
        check > &low && check < &high
    }
    /// Tells if a value is approximately double to a compared value. Used to determine how long a
    /// half-bit and a bit is
    fn is_approx_double(check: &usize, comp: &usize) -> bool {
        Self::is_approx_half(comp, check)
    }
    /// Tells if a value is approximately the same to a compared value. Used to determine how long a
    /// half-bit and a bit is
    fn is_approx_same(check: &usize, comp: &usize) -> bool {
        let low = (comp * 4) / 5;
        let high = (comp * 5) / 4;
        check >= &low && check <= &high
    }
}

/// The detector takes audio smaples one after another and eventually will return if a half-bit
/// or a bit was detected on a threshold cross.
struct ThresholdCrossDetector<T: Sample> {
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
    fn new() -> Self {
        Self {
            sample_bounds: SampleBounds::new(),
            counting: false,
            is_high: None,
            count: 0,
            state: ThresholdCrossState::new(),
        }
    }

    /// Used to find threshold-crosses. Returns if a bit or a half-bit duration cross has been detected
    fn crosses(&mut self, sample: T) -> ThresholdCross {
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
                return self.state.cross_from_cross_size(count);
            }
            ThresholdCross::None
        } else {
            //Sample bounds does not know the treshold for low and high bits at the moment
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
    use crate::ltc_decoder::bit_decoder::{SampleBounds, ThresholdCrossState};

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
}