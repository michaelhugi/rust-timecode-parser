extern crate core;

use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

pub mod decoder;
mod ltc_frame;
#[cfg(feature = "decode_ltc")]
mod ltc_decoder;


pub struct TimecodeFrame {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub frames_per_second: FramesPerSecond,
    pub frame_units_user_bits: u8,
    pub frame_tens_user_bits: u8,
    pub second_units_user_bits: u8,
    pub second_tens_user_bits: u8,
    pub minute_units_user_bits: u8,
    pub minute_tens_user_bits: u8,
    pub hour_units_user_bits: u8,
    pub hour_tens_user_bits: u8,
}

#[cfg(feature = "debug")]
impl Display for TimecodeFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //TODO leading zeros
        write!(f, "{}:{}:{}:{}", self.hours, self.minutes, self.seconds, self.frames)
    }
}

#[cfg(feature = "debug")]
impl Debug for TimecodeFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl TimecodeFrame {
    pub fn new_without_user_bits(hours: u8, minutes: u8, seconds: u8, frames: u8) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            frames,
            frames_per_second: FramesPerSecond::Unknown,
            frame_units_user_bits: 0,
            frame_tens_user_bits: 0,
            second_units_user_bits: 0,
            second_tens_user_bits: 0,
            minute_units_user_bits: 0,
            minute_tens_user_bits: 0,
            hour_units_user_bits: 0,
            hour_tens_user_bits: 0,
        }
    }
}

pub enum FramesPerSecond {
    Unknown,
    TwentyFour,
    TwentyFive,
    Thirty,
}


