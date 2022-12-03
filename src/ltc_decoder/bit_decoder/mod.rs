#![no_std]

use crate::ltc_decoder::bit_decoder::sample_bounds::SampleBounds;
use crate::ltc_decoder::bit_decoder::sample_rater::*;
use crate::ltc_decoder::bit_decoder::threshold_cross_detector::{ThresholdCross, ThresholdCrossDetector};
use crate::ltc_decoder::bit_decoder::zero_detector::ZeroDetector;
use crate::ltc_decoder::print_decoder::AudioImage;
use crate::ltc_decoder::Sample;

mod sample_bounds;
mod zero_detector;
mod sample_rater;
mod threshold_cross_detector;

/// Contains the state of received half-bits and bits by ThresholdCrossDetector
enum BitDecoderState {
    /// Waiting for a full-bit to receive to get in sync
    OutOfSync,
    /// Received a full-bit or two half-bits and waiting for a full-bit or half-bit
    BitCompleted,
    /// Half bit received. Waiting for second half-bit -> Otherwise invalidate
    HalfBitReceived,
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
    fn invalidate(&mut self) {
        self.state = BitDecoderState::OutOfSync;
        self.threshold_cross_detector.invalidate();
    }
    /// Every audio sample-point that is received is pushed in this function. It will return if a bit
    /// is detected by returning true (1) or false (0)
    /// The function feeds and handles detection of audio-level for high and low as well as bit-heartbeat detection
    pub(crate) fn push_sample(&mut self, sample: T) -> Option<bool> {
        match self.threshold_cross_detector.crosses(sample) {
            ThresholdCross::None => None,
            ThresholdCross::Invalid => {
                self.invalidate();
                None
            }
            ThresholdCross::Short => {
                // half bit received
                match self.state {
                    BitDecoderState::OutOfSync => None,
                    BitDecoderState::BitCompleted => {
                        self.state = BitDecoderState::HalfBitReceived;
                        None
                    }
                    BitDecoderState::HalfBitReceived => {
                        self.state = BitDecoderState::BitCompleted;
                        Some(true)
                    }
                }
            }
            ThresholdCross::Long => {
                // full bit received
                match self.state {
                    BitDecoderState::OutOfSync => {
                        self.state = BitDecoderState::BitCompleted;
                        Some(false)
                    }
                    BitDecoderState::BitCompleted => {
                        Some(false)
                    }
                    BitDecoderState::HalfBitReceived => {
                        /// Expected a half-bit in the state of sync
                        self.invalidate();
                        None
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_testing() {}
}