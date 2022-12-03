use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;

use intbits::Bits;
use crate::ltc_frame::LtcFrame;
use crate::TimecodeFrame;

///Contains all the data of a LtcFrame without the SyncWord
#[derive(Clone)]
pub(crate) struct LtcFrameData {
    data: u64,
}

/// Implementation used for decidubg abd encoding
impl LtcFrameData {
    ///bit range ltc-definition for frame-units
    const BIT_RANGE_FRAME_UNITS: Range<u64> = 0..4;
    ///bit range ltc-definition for frame-units-user-bits
    const BIT_RANGE_FRAME_UNITS_USER_BITS: Range<u64> = 4..8;
    ///bit range ltc-definition for frame-tens
    const BIT_RANGE_FRAME_TENS: Range<u64> = 8..10;
    ///bit range ltc-definition for frame-tens-user-bits
    const BIT_RANGE_FRAME_TENS_USER_BITS: Range<u64> = 12..16;
    ///bit range ltc-definition for second-units
    const BIT_RANGE_SECOND_UNITS: Range<u64> = 16..20;
    ///bit range ltc-definition for second-units-user-bits
    const BIT_RANGE_SECOND_UNITS_USER_BITS: Range<u64> = 20..24;
    ///bit range ltc-definition for second-tens
    const BIT_RANGE_SECOND_TENS: Range<u64> = 24..27;
    ///bit range ltc-definition for second-tens-user-bits
    const BIT_RANGE_SECOND_TENS_USER_BITS: Range<u64> = 28..32;
    ///bit range ltc-definition for minute-units
    const BIT_RANGE_MINUTE_UNITS: Range<u64> = 32..36;
    ///bit range ltc-definition for minute-units-user-bits
    const BIT_RANGE_MINUTE_UNITS_USER_BITS: Range<u64> = 36..40;
    ///bit range ltc-definition for minute-tens
    const BIT_RANGE_MINUTE_TENS: Range<u64> = 40..43;
    ///bit range ltc-definition for minute-tens-user-bits
    const BIT_RANGE_MINUTE_TENS_USER_BITS: Range<u64> = 44..48;
    ///bit range ltc-definition for hour-units
    const BIT_RANGE_HOUR_UNITS: Range<u64> = 48..52;
    ///bit range ltc-definition for hour-units-user-bits
    const BIT_RANGE_HOUR_UNITS_USER_BITS: Range<u64> = 52..56;
    ///bit range ltc-definition for hour-tens
    const BIT_RANGE_HOUR_TENS: Range<u64> = 56..58;
    ///bit range ltc-definition for hour-tens-user-bits
    const BIT_RANGE_HOUR_TENS_USER_BITS: Range<u64> = 60..64;

    #[cfg(test)]
    pub(crate) fn new_raw(data: u64) -> Self {
        Self {
            data
        }
    }
}

///Write data implementations
#[cfg(feature = "encode_ltc")]
impl LtcFrameData {
    ///Constructor to write data to an audio stream
    fn new_with_time(hours: u8, minutes: u8, seconds: u8, frames: u8) -> Self {
        let mut s = Self {
            data: 0
        };
        s.set_hours(hours);
        s.set_minutes(minutes);
        s.set_seconds(seconds);
        s.set_frames(frames);
        s
    }

    ///Helper function (with type convertion)
    fn set_bits(&mut self, range: Range<u64>, bits: u8) {
        self.data.set_bits(range, bits as u64)
    }
    /// Helper function. Because tens are written in the stram separately
    /// assert_eq!(LtcFrameData::split_to_tens_and_units(16),(1,6));
    /// assert_eq!(LtcFrameData::split_to_tens_and_units(6),(0,6));
    /// assert_eq!(LtcFrameData::split_to_tens_and_units(10),(1,0));
    fn split_to_tens_and_units(val: u8) -> (u8, u8) {
        let units = val % 10;
        let tens = (val - units) / 10;
        (tens, units)
    }
    ///Sets the bits in LtcFrameData for specified frames
    pub(crate) fn set_frames(&mut self, frames: u8) {
        let (tens, units) = Self::split_to_tens_and_units(frames);
        self.set_bits(Self::BIT_RANGE_FRAME_TENS, tens);
        self.set_bits(Self::BIT_RANGE_FRAME_UNITS, units);
    }
    ///Sets the bits in LtcFrameData for specified seconds
    pub(crate) fn set_seconds(&mut self, seconds: u8) {
        let (tens, units) = Self::split_to_tens_and_units(seconds);
        self.set_bits(Self::BIT_RANGE_SECOND_TENS, tens);
        self.set_bits(Self::BIT_RANGE_SECOND_UNITS, units);
    }
    ///Sets the bits in LtcFrameData for specified minutes
    pub(crate) fn set_minutes(&mut self, minutes: u8) {
        let (tens, units) = Self::split_to_tens_and_units(minutes);
        self.set_bits(Self::BIT_RANGE_MINUTE_TENS, tens);
        self.set_bits(Self::BIT_RANGE_MINUTE_UNITS, units);
    }
    ///Sets the bits in LtcFrameData for specified hours
    pub(crate) fn set_hours(&mut self, hours: u8) {
        let (tens, units) = Self::split_to_tens_and_units(hours);
        self.set_bits(Self::BIT_RANGE_HOUR_TENS, tens);
        self.set_bits(Self::BIT_RANGE_HOUR_UNITS, units);
    }
    ///Sets the bits in LtcFrameData for frame-units-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_frame_units_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_FRAME_UNITS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for frame-tens-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_frame_tens_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_FRAME_TENS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for second-units-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_second_units_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_SECOND_UNITS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for second-tens-unser-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_second_tens_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_SECOND_TENS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for minute-units-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_minute_units_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_MINUTE_UNITS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for minute-tens-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_minute_tens_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_MINUTE_TENS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for hour-units-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_hour_units_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_HOUR_UNITS_USER_BITS, bits);
    }
    ///Sets the bits in LtcFrameData for hour-tens-user-bits
    /// Used for adding additional user data to the timecode
    pub(crate) fn set_hour_tens_user_bits(&mut self, bits: u8) {
        self.set_bits(Self::BIT_RANGE_HOUR_TENS_USER_BITS, bits);
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
    ///Helper function (with type convertion)
    fn get_bits(&self, range: Range<u64>) -> u8 {
        self.data.bits(range) as u8
    }
    /// Returns the number of frames in the LtcFrameData
    pub(crate) fn get_frames(&self) -> u8 {
        self.get_bits(Self::BIT_RANGE_FRAME_UNITS) + 10 * self.get_bits(Self::BIT_RANGE_FRAME_TENS)
    }
    /// Returns the number of seconds in the LtcFrameData
    pub(crate) fn get_seconds(&self) -> u8 {
        self.get_bits(Self::BIT_RANGE_SECOND_UNITS) + 10 * self.get_bits(Self::BIT_RANGE_SECOND_TENS)
    }
    /// Returns the number of minutes in the LtcFrameData
    pub(crate) fn get_minutes(&self) -> u8 {
        self.get_bits(Self::BIT_RANGE_MINUTE_UNITS) + 10 * self.get_bits(Self::BIT_RANGE_MINUTE_TENS)
    }
    /// Returns the number of hours in the LtcFrameData
    pub(crate) fn get_hours(&self) -> u8 {
        self.get_bits(Self::BIT_RANGE_HOUR_UNITS) + 10 * self.get_bits(Self::BIT_RANGE_HOUR_TENS)
    }
    /// Returns the bits for frame-units-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_frame_units_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_FRAME_UNITS_USER_BITS) }
    /// Returns the bits for frame-tens-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_frame_tens_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_FRAME_TENS_USER_BITS) }
    /// Returns the bits for second-units-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_second_units_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_SECOND_UNITS_USER_BITS) }
    /// Returns the bits for second-tens-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_second_tens_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_SECOND_TENS_USER_BITS) }
    /// Returns the bits for minute-units-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_minute_units_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_MINUTE_UNITS_USER_BITS) }
    /// Returns the bits for minute-tens-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_minute_tens_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_MINUTE_TENS_USER_BITS) }
    /// Returns the bits for hour-units-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_hour_units_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_HOUR_UNITS_USER_BITS) }
    /// Returns the bits for hour-tens-user-bits the LtcFrameData
    /// Used for adding additional user data to the timecode
    pub(crate) fn get_hour_tens_user_bits(&self) -> u8 { self.get_bits(Self::BIT_RANGE_HOUR_TENS_USER_BITS) }
    ///Adds a bit at the end of the stream and returns the one on the beginning
    /// When reading from an ltc-audio-stream bit by bit can be passed in until the SyncKeyword matches the position whenn all data is received
    /// The overflow is needed to add it to the current SyncWord in LtcFrame to detect if the frame is complete
    pub(crate) fn shift_bit_with_overflow(&mut self, bit: bool) -> bool {
        let highest_bit = self.data.bit(63);
        self.data = self.data << 1;
        self.data.set_bit(0, bit);
        highest_bit
    }
}

#[cfg(feature = "decode_ltc")]
impl LtcFrameData {
    pub(crate) fn into_ltc_frame(&self) -> TimecodeFrame {
        TimecodeFrame::new_without_user_bits(self.get_hours(), self.get_minutes(), self.get_seconds(), self.get_frames())
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>2}:{:0>2}:{:0>2}:{:0>2}\n0b_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}_{:04b}",
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>2}:{:0>2}:{:0>2}:{:0>2}",
               self.get_hours(),
               self.get_minutes(),
               self.get_seconds(),
               self.get_frames())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bits() {
        let d = LtcFrameData::new_raw(0b_1111_1110_1101_1100_1011_1010_1001_1000_0111_0110_0101_0100_0011_0010_0001_0000);
        assert_eq!(d.get_bits(0..4), 0b_0000);
        assert_eq!(d.get_bits(4..8), 0b_0001);
        assert_eq!(d.get_bits(8..12), 0b_0010);
        assert_eq!(d.get_bits(60..64), 0b_1111);
        assert_eq!(d.get_frame_units_user_bits(), 0b0001);
        assert_eq!(d.get_frame_tens_user_bits(), 0b0011);
        assert_eq!(d.get_second_units_user_bits(), 0b0101);
        assert_eq!(d.get_second_tens_user_bits(), 0b0111);
        assert_eq!(d.get_minute_units_user_bits(), 0b1001);
        assert_eq!(d.get_minute_tens_user_bits(), 0b1011);
        assert_eq!(d.get_hour_units_user_bits(), 0b1101);
        assert_eq!(d.get_hour_tens_user_bits(), 0b1111);

        let d = LtcFrameData::new_raw(0b_0000_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_1110_0000_1001);
        assert_eq!(d.get_frames(), 29);
        assert_eq!(d.get_seconds(), 14);
        assert_eq!(d.get_minutes(), 38);
        assert_eq!(d.get_hours(), 5);
    }

    #[test]
    fn test_split_to_units_and_tens() {
        assert_eq!(LtcFrameData::split_to_tens_and_units(16), (1, 6));
        assert_eq!(LtcFrameData::split_to_tens_and_units(6), (0, 6));
        assert_eq!(LtcFrameData::split_to_tens_and_units(10), (1, 0));
        assert_eq!(LtcFrameData::split_to_tens_and_units(0), (0, 0));
    }

    #[test]
    fn test_setters() {
        let d = LtcFrameData::new_with_time(5, 38, 14, 29);
        assert_eq!(d, LtcFrameData::new_raw(0b_0000_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001))
    }

    #[test]
    fn test_shift_bit_with_overflow() {
        let mut d = LtcFrameData::new_raw(0b_0100_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001);
        assert_eq!(d.shift_bit_with_overflow(true), false);
        assert_eq!(d, LtcFrameData::new_raw(0b_100_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001_1));
        assert_eq!(d.shift_bit_with_overflow(true), true);
        assert_eq!(d, LtcFrameData::new_raw(0b_00_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001_11));
        assert_eq!(d.shift_bit_with_overflow(false), false);
        assert_eq!(d, LtcFrameData::new_raw(0b_0_0000_0000_0101_0000_0011_0000_1000_0000_0001_0000_0100_0000_0010_0000_1001_110));
    }
}