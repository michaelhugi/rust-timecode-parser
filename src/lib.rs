#![cfg_attr(not(test), no_std)]
extern crate core;

use core::fmt::{Debug, Display, Formatter};

pub mod ltc_frame;
#[cfg(feature = "decode_ltc")]
pub mod ltc_decoder;

#[derive(PartialEq, Eq, Clone)]
pub struct TimecodeFrame {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub frames_per_second: FramesPerSecond,
}

impl TimecodeFrame {
    pub fn add_frame(&mut self) {
        self.frames += 1;
        match self.frames_per_second {
            FramesPerSecond::Unknown => {}
            FramesPerSecond::TwentyFour => {
                if self.frames >= 24 {
                    self.frames = 0;
                    self.seconds += 1;
                }
            }
            FramesPerSecond::TwentyFive => {
                if self.frames >= 25 {
                    self.frames = 0;
                    self.seconds += 1;
                }
            }
            FramesPerSecond::Thirty => {
                if self.frames >= 30 {
                    self.frames = 0;
                    self.seconds += 1;
                }
            }
        }
        if self.seconds > 59 {
            self.seconds = 0;
            self.minutes += 1;
        }
        if self.minutes > 59 {
            self.minutes = 0;
            self.hours += 1;
        }
    }
}

#[cfg(feature = "debug")]
impl Display for TimecodeFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}:{:02} fps:{:#?}", self.hours, self.minutes, self.seconds, self.frames, self.frames_per_second)
    }
}

#[cfg(feature = "debug")]
impl Debug for TimecodeFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl TimecodeFrame {
    pub fn new_from_duration(hours: u8, minutes: u8, seconds: u8, frames: u8, duration_for_frame_without_syncword_in_s: f32) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            frames,
            frames_per_second: FramesPerSecond::from_frame_duration_without_syncword_in_s(duration_for_frame_without_syncword_in_s),
        }
    }
    pub fn new(hours: u8, minutes: u8, seconds: u8, frames: u8, frames_per_second: FramesPerSecond) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            frames,
            frames_per_second,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum FramesPerSecond {
    Unknown,
    TwentyFour,
    TwentyFive,
    Thirty,
}

impl FramesPerSecond {
    const DURATION_THIRTY_FULL_FRAME_IN_S: f32 = 0.033_333_33;
    const DURATION_TWENTY_FIVE_FULL_FRAME_IN_S: f32 = 0.04;
    const DURATION_TWENTY_FOUR_FULL_FRAME_IN_S: f32 = 0.041_666_66;

    const DURATION_TWENTY_FOUR_WITHOUT_SYNC_WORD_IN_S: f32 = Self::DURATION_TWENTY_FOUR_FULL_FRAME_IN_S * 64.0 / 80.0;
    const DURATION_TWENTY_FIVE_WITHOUT_SYNC_WORD_IN_S: f32 = Self::DURATION_TWENTY_FIVE_FULL_FRAME_IN_S * 64.0 / 80.0;
    const DURATION_THIRTY_WITHOUT_SYNC_WORD_IN_S: f32 = Self::DURATION_THIRTY_FULL_FRAME_IN_S * 64.0 / 80.0;

    const DURATION_BOUND_TWENTY_FOUR_WITHOUT_SYNC_WORD_IN_S: (f32, f32) = (Self::DURATION_TWENTY_FOUR_WITHOUT_SYNC_WORD_IN_S * 0.98, Self::DURATION_TWENTY_FOUR_WITHOUT_SYNC_WORD_IN_S * 1.02);
    const DURATION_BOUND_THWENTY_FIVE_WITHOUT_SYNC_WORD_IN_S: (f32, f32) = (Self::DURATION_TWENTY_FIVE_WITHOUT_SYNC_WORD_IN_S * 0.98, Self::DURATION_TWENTY_FIVE_WITHOUT_SYNC_WORD_IN_S * 1.02);
    const DURATION_BOUND_THIRTY_WITHOUT_SYNC_WORD_IN_S: (f32, f32) = (Self::DURATION_THIRTY_WITHOUT_SYNC_WORD_IN_S * 0.98, Self::DURATION_THIRTY_WITHOUT_SYNC_WORD_IN_S * 1.02);

    fn from_frame_duration_without_syncword_in_s(frames_duration_s: f32) -> FramesPerSecond {
        if Self::is_in_duration_bounds(frames_duration_s, Self::DURATION_BOUND_TWENTY_FOUR_WITHOUT_SYNC_WORD_IN_S) {
            return FramesPerSecond::TwentyFour;
        }
        if Self::is_in_duration_bounds(frames_duration_s, Self::DURATION_BOUND_THWENTY_FIVE_WITHOUT_SYNC_WORD_IN_S) {
            return FramesPerSecond::TwentyFive;
        }
        if Self::is_in_duration_bounds(frames_duration_s, Self::DURATION_BOUND_THIRTY_WITHOUT_SYNC_WORD_IN_S) {
            return FramesPerSecond::Thirty;
        }
        FramesPerSecond::Unknown
    }

    fn is_in_duration_bounds(frames_duration_s: f32, bounds: (f32, f32)) -> bool {
        frames_duration_s > bounds.0 && frames_duration_s < bounds.1
    }
}
