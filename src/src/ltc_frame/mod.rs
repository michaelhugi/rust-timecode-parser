use core::fmt::{Debug, Display, Formatter};
use intbits::Bits;
use crate::ltc_frame::ltc_frame_data::LtcFrameData;

pub(crate) mod ltc_frame_data;

/// Represents 80 bits that represent a ltc-tc-frame
/// Contains functions to push bits received by an audio signal and read it's value as well as functions to write bits to the audio
pub(crate) struct LtcFrame {
    ///Are on higher index of all bits received
    sync_word: u16,
    ///Contains the data of the old-frame, if the frame is complete
    data: LtcFrameData,
    /// Tells how many samples it took to get a whole tc-frame without sync-word
    frame_data_sample_count: usize,
}

impl LtcFrame {}

#[cfg(test)]
impl PartialEq<Self> for LtcFrame {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.sync_word == other.sync_word
    }
}

///Implementations that are used to decode and encode timecode
impl LtcFrame {
    const LTC_SYNC_WORD: u16 = 0b_0011_1111_1111_1101;

    /// Invalidates the current status of the ltc-frame
    pub(crate) fn invalidate(&mut self) {
        self.data.invalidate();
        self.sync_word = 0;
    }
}

#[cfg(feature = "debug")]
impl Debug for LtcFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "sync_word: 0b_{:04b}_{:04b}_{:04b}_{:04b}\ndata: {:?}",
               self.sync_word.bits(12..16),
               self.sync_word.bits(8..12),
               self.sync_word.bits(4..8),
               self.sync_word.bits(0..4),
               self.data
        )
    }
}

#[cfg(feature = "debug")]
impl Display for LtcFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "sync_word:{}\ndata: {}", self.sync_word == Self::LTC_SYNC_WORD.bits(12..16), self.data)
    }
}

#[cfg(feature = "decode_ltc")]
impl LtcFrame {
    ///Constructor that is used when reading ltc stream from audio
    pub(crate) fn new_empty() -> Self {
        Self {
            sync_word: 0,
            data: LtcFrameData::new_empty(),
            frame_data_sample_count: 0,
        }
    }
    ///When a new audio bit is received, this function will shift all received data and add it to the end. Once the sync_word matches, the data is a valid frame
    pub(crate) fn shift_bit(&mut self, bit: bool) {
        let overflow_bit = self.data.shift_bit_with_overflow(bit);
        self.sync_word <<= 1;
        self.sync_word.set_bit(0, overflow_bit);
    }
    ///Tells if all data is received by the audio stream after the sync-word
    pub(crate) fn data_valid(&self) -> bool {
        self.sync_word == Self::LTC_SYNC_WORD
    }
    ///Used to count how many samples a timecode-frame has needed to complete do determine FramesPerSecond of LTC
    pub(crate) fn sample_received(&mut self) {
        if self.data.next_bit_is_start_of_frame() {
            self.frame_data_sample_count = 0;
        } else {
            self.frame_data_sample_count += 1;
        }
    }

    ///Returns the data read from audio decoding only if all data has been received after the sync-word
    /// It may be more efficient to first check if data_valid() returns true due to less memory allocation in ram
    pub(crate) fn get_data(&mut self) -> Option<(LtcFrameData, usize)> {
        if self.data_valid() {
            Some((self.data.clone(), self.frame_data_sample_count))
        } else {
            None
        }
    }
}
