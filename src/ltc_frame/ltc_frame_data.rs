use core::fmt::{Debug, Display, Formatter};

use intbits::Bits;

use crate::TimecodeFrame;

///Contains all the data of a LtcFrame without the SyncWord
#[derive(Clone)]
pub(crate) struct LtcFrameData {
    data: u64,
}

/// Holds the index and it's weight in LTC specification for one bit
struct BitIndex {
    index: u8,
    weight: u8,
}

impl BitIndex {
    const fn new(index: u8, weight: u8) -> Self {
        Self {
            // Bits arrive reversed
            index: 63 - index,
            weight,
        }
    }
}

/// Implementation used for decidubg abd encoding
impl LtcFrameData {
    /// All BitIndex to define a frame
    const BIT_INDEX_FRAMES: [BitIndex; 6] =
        [BitIndex::new(0, 1),
            BitIndex::new(1, 2),
            BitIndex::new(2, 4),
            BitIndex::new(3, 8),
            BitIndex::new(8, 10),
            BitIndex::new(9, 20)];
    const BIT_INDEX_SECONDS: [BitIndex; 7] =
        [BitIndex::new(16, 1),
            BitIndex::new(17, 2),
            BitIndex::new(18, 4),
            BitIndex::new(19, 8),
            BitIndex::new(24, 10),
            BitIndex::new(25, 20),
            BitIndex::new(26, 40)];
    const BIT_INDEX_MINUTES: [BitIndex; 7] =
        [BitIndex::new(32, 1),
            BitIndex::new(33, 2),
            BitIndex::new(34, 4),
            BitIndex::new(35, 8),
            BitIndex::new(40, 10),
            BitIndex::new(41, 20),
            BitIndex::new(42, 40)];
    const BIT_INDEX_HOURS: [BitIndex; 6] =
        [BitIndex::new(48, 1),
            BitIndex::new(49, 2),
            BitIndex::new(50, 4),
            BitIndex::new(51, 8),
            BitIndex::new(56, 10),
            BitIndex::new(57, 20)];
    /// If syncword is completely received, the data will start now
    /// Syncword bits is divided by two to avoid having to work with 16bit values for all bits
    const BIT_INDEX_SYNCWORD_START_FIRST_HALF: [BitIndex; 8] =
        [BitIndex::new(63, 1),
            BitIndex::new(62, 2),
            BitIndex::new(61, 4),
            BitIndex::new(60, 8),
            BitIndex::new(59, 16),
            BitIndex::new(58, 32),
            BitIndex::new(57, 64),
            BitIndex::new(56, 128)];
    /// If syncword is completely received, the data will start now
    /// Syncword bits is divided by two to avoid having to work with 16bit values for all bits
    const BIT_INDEX_SYNCWORD_START_SECOND_HALF: [BitIndex; 8] =
        [BitIndex::new(55, 1),
            BitIndex::new(54, 2),
            BitIndex::new(53, 4),
            BitIndex::new(52, 8),
            BitIndex::new(51, 16),
            BitIndex::new(50, 32),
            BitIndex::new(49, 64),
            BitIndex::new(48, 128)];
    const SYNC_WORD_SECOND_HALF: u8 = 0b0011_1111;
    const SYNC_WORD_FIRST_HALF: u8 = 0b1111_1101;
    /// Invalidates the data in case of unexpected data is received
    pub(crate) fn invalidate(&mut self) {
        self.data = 0;
    }
}


///Read data implementation
#[cfg(feature = "decode_ltc")]
impl LtcFrameData {
    ///Constructor for new empty ltc-frame-date for reading data from audio stream
    pub(crate) fn new_empty() -> Self {
        Self {
            data: 0
        }
    }
    /// Helper function (with type convertion)
    fn get_bits(&self, index: &[BitIndex]) -> u8 {
        let mut val = 0;
        for i in index {
            if self.data.bit(i.index) {
                val += i.weight
            }
        }

        val
    }
    ///Tells if sync-word has been received. This will help to track, how lon it takes to receive the
    /// data to determine the Timecode FrameRate
    pub(crate) fn next_bit_is_start_of_frame(&self) -> bool {
        Self::SYNC_WORD_FIRST_HALF == self.get_bits(&Self::BIT_INDEX_SYNCWORD_START_FIRST_HALF) &&
            Self::SYNC_WORD_SECOND_HALF == self.get_bits(&Self::BIT_INDEX_SYNCWORD_START_SECOND_HALF)
    }
    /// Returns the number of frames in the LtcFrameData
    pub(crate) fn get_frames(&self) -> u8 {
        self.get_bits(&Self::BIT_INDEX_FRAMES)
    }
    /// Returns the number of seconds in the LtcFrameData
    pub(crate) fn get_seconds(&self) -> u8 {
        self.get_bits(&Self::BIT_INDEX_SECONDS)
    }
    /// Returns the number of minutes in the LtcFrameData
    pub(crate) fn get_minutes(&self) -> u8 {
        self.get_bits(&Self::BIT_INDEX_MINUTES)
    }
    /// Returns the number of hours in the LtcFrameData
    pub(crate) fn get_hours(&self) -> u8 {
        self.get_bits(&Self::BIT_INDEX_HOURS)
    }
    ///Adds a bit at the end of the stream and returns the one on the beginning
    /// When reading from an ltc-audio-stream bit by bit can be passed in until the SyncKeyword matches the position whenn all data is received
    /// The overflow is needed to add it to the current SyncWord in LtcFrame to detect if the frame is complete
    pub(crate) fn shift_bit_with_overflow(&mut self, bit: bool) -> bool {
        let highest_bit = self.data.bit(63);
        self.data <<= 1;
        self.data.set_bit(0, bit);
        highest_bit
    }
}

#[cfg(feature = "decode_ltc")]
impl LtcFrameData {
    pub(crate) fn make_ltc_frame(&self, duration_for_frame_without_syncword_in_s: f32) -> TimecodeFrame {
        TimecodeFrame::new_from_duration(self.get_hours(), self.get_minutes(), self.get_seconds(), self.get_frames(), duration_for_frame_without_syncword_in_s)
    }
}

#[cfg(test)]
impl PartialEq<Self> for LtcFrameData {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

#[cfg(feature = "debug")]
impl Debug for LtcFrameData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0>2}:{:0>2}:{:0>2}:{:0>2} 0b_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}",
               self.get_hours(),
               self.get_minutes(),
               self.get_seconds(),
               self.get_frames(),
               self.data.bits(60..64),
               self.data.bits(56..60),
               self.data.bits(52..56),
               self.data.bits(48..52),
               self.data.bits(44..48),
               self.data.bits(40..44),
               self.data.bits(36..40),
               self.data.bits(32..36),
               self.data.bits(28..32),
               self.data.bits(24..28),
               self.data.bits(20..24),
               self.data.bits(16..20),
               self.data.bits(12..16),
               self.data.bits(8..12),
               self.data.bits(4..8),
               self.data.bits(0..4))
    }
}

#[cfg(feature = "debug")]
impl Display for LtcFrameData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:0>2}:{:0>2}:{:0>2}:{:0>2}",
               self.get_hours(),
               self.get_minutes(),
               self.get_seconds(),
               self.get_frames())
    }
}
