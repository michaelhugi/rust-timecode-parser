use std::fmt::{Debug, Display, Formatter};
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
}

#[cfg(test)]
impl PartialEq<Self> for LtcFrame {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.sync_word == other.sync_word
    }
}

///Implementations that are used to decode and encode timecode
impl LtcFrame {
    const LTC_SYNC_WORD: u16 = 0b_0011_1111_1111_1101;
    #[cfg(test)]
    pub(crate) fn new_raw(sync_word: u16, data: u64) -> Self {
        Self {
            sync_word,
            data: LtcFrameData::new_raw(data),
        }
    }
}

#[cfg(feature = "debug")]
impl Debug for LtcFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
        }
    }
    ///When a new audio bit is received, this function will shift all received data and add it to the end. Once the sync_word matches, the data is a valid frame
    pub(crate) fn shift_bit(&mut self, bit: bool) {
        let overflow_bit = self.data.shift_bit_with_overflow(bit);
        self.sync_word = self.sync_word << 1;
        self.sync_word.set_bit(0, overflow_bit);
    }
    ///Tells if all data is received by the audio stream after the sync-word
    pub(crate) fn data_valid(&self) -> bool {
        self.sync_word == Self::LTC_SYNC_WORD
    }

    ///Returns the data read from audio decoding only if all data has been received after the sync-word
    /// It may be more efficient to first check if data_valid() returns true due to less memory allocation in ram
    pub(crate) fn get_data(&self) -> Option<LtcFrameData> {
        if self.data_valid() {
            Some(self.data.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_bit() {
        let mut f = LtcFrame::new_raw(0b_1000_0000_0010_0010, 0b_1000_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001);
        f.shift_bit(true);
        assert_eq!(f, LtcFrame::new_raw(0b_000_0000_0010_0010_1, 0b_000_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001_1));
        f.shift_bit(true);
        assert_eq!(f, LtcFrame::new_raw(0b_00_0000_0010_0010_10, 0b_00_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001_11));
        f.shift_bit(false);
        assert_eq!(f, LtcFrame::new_raw(0b_0_0000_0010_0010_100, 0b_0_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001_110));
    }
}