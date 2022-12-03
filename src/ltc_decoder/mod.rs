use std::fmt::Display;
use std::fs::File;
use std::ops::{Add, Div, Shl, Sub};
use std::path::Path;

use num_traits::{FromPrimitive, One, ToPrimitive, Zero};
#[cfg(test)]
use wav::BitDepth;

use crate::ltc_decoder::bit_decoder::BitDecoder;
use crate::ltc_decoder::print_decoder::AudioImage;
use crate::ltc_frame::LtcFrame;
use crate::TimecodeFrame;

mod bit_decoder;
mod print_decoder;

//pub trait Sample: Copy + Zero + std::ops::Div<f64>+ FromPrimitive + Ord + Sync + Send + 'static {}
//pub trait Sample: Zero + Ord + Clone + Copy + 'static {}

pub trait Sample: Zero + Ord + Clone + Copy + FromPrimitive + ToPrimitive + Display + 'static {}

impl<T> Sample for T where T: Zero + Ord + Clone + Copy + FromPrimitive + ToPrimitive + Display + 'static {}

pub struct LtcDecoder<T: Sample> {
    ltc_frame: LtcFrame,
    bit_decoder: BitDecoder<T>,
}

impl<T: Sample> LtcDecoder<T> {
    pub fn new<F: Into<f64>>(sample_rate: F) -> Self {
        let sample_rate: f64 = sample_rate.into();
        Self {
            ltc_frame: LtcFrame::new_empty(),
            bit_decoder: BitDecoder::new(sample_rate),
        }
    }
    /// Push received audio-sample-point one after another in this function. From time to time
    /// a Timecode-Frame will be returned to tell the current received timecode
    pub fn push_sample(&mut self, sample: T, index: usize, images: &mut [AudioImage]) -> Option<TimecodeFrame> {
        if let Some(bit) = self.bit_decoder.push_sample(sample, index, images) {
            self.ltc_frame.shift_bit(bit);
            if let Some(data) = self.ltc_frame.get_data() {
                return Some(data.into_ltc_frame());
            }
        }
        None
    }
}

#[cfg(test)]
pub(crate) fn get_test_samples_48_14() -> (u32, Vec<i32>) {
    let mut inp_file = File::open(Path::new("testfiles/TC25_48000_14.00.00.00.wav")).unwrap();
    let (header, data) = wav::read(&mut inp_file).unwrap();
    //
    assert_eq!(header.channel_count, 1);
    let mut index = 0;
    match data {
        BitDepth::TwentyFour(samples) => {
            (header.sampling_rate, samples)
        }
        _ => panic!("Unexpected bitdepth"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::ops::Shl;
    use std::path::Path;

    use num_traits::Zero;
    use wav::BitDepth;

    use crate::ltc_decoder::{get_test_samples_48_14, LtcDecoder, Sample};
    use crate::ltc_decoder::print_decoder::AudioImage;

    #[test]
    fn test_and_print_timecode() {
        let (sampling_rate, samples) = get_test_samples_48_14();
        let mut decoder = LtcDecoder::<i32>::new(sampling_rate);
        let mut index = 0;

        let mut images = [AudioImage::new(&samples, 0)];

        for sample in samples {
            if let Some(tc) = decoder.push_sample(sample, index, &mut images) {
                println!("TC: {:?}", tc);
            } else {
                if index < 200 {}
            }
            index += 1;
        }
        images[0].save("TC4814(0)");
    }

    #[test]
    fn test_sample_trait() {
        test_zero(0_i64);
        test_zero(0_i32);
        test_zero(0_i16);
        test_zero(0_i8);
        test_zero(0_u64);
        test_zero(0_u32);
        test_zero(0_u16);
        test_zero(0_u8);

        test_ord(0_i64);
        test_ord(0_i32);
        test_ord(0_i16);
        test_ord(0_i8);
        test_ord(0_u64);
        test_ord(0_u32);
        test_ord(0_u16);
        test_ord(0_u8);

        test_clone(0_i64);
        test_clone(0_i32);
        test_clone(0_i16);
        test_clone(0_i8);
        test_clone(0_u64);
        test_clone(0_u32);
        test_clone(0_u16);
        test_clone(0_u8);

        test_copy(0_i64);
        test_copy(0_i32);
        test_copy(0_i16);
        test_copy(0_i8);
        test_copy(0_u64);
        test_copy(0_u32);
        test_copy(0_u16);
        test_copy(0_u8);

        test_shl(0_i64);
        test_shl(0_i32);
        test_shl(0_i16);
        test_shl(0_i8);
        test_shl(0_u64);
        test_shl(0_u32);
        test_shl(0_u16);
        test_shl(0_u8);

        test_sample(0_i64);
        test_sample(0_i32);
        test_sample(0_i16);
        test_sample(0_i8);
        test_sample(0_u64);
        test_sample(0_u32);
        test_sample(0_u16);
        test_sample(0_u8);
    }

    fn test_zero<T: Zero>(_s: T) {
        assert!(true);
    }

    fn test_ord<T: Ord>(_s: T) {
        assert!(true);
    }

    fn test_clone<T: Clone>(_s: T) {
        assert!(true);
    }

    fn test_copy<T: Copy>(_s: T) {
        assert!(true);
    }

    fn test_sample<T: Sample>(_s: T) {
        assert!(true);
    }

    fn test_shl<T: Shl>(_s: T) {
        assert!(true);
    }
}