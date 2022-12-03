use std::cmp::{max, min};

use draw::{Canvas, Color, Drawing, LineBuilder, Point, render, RGB, Shape, Style, SvgRenderer};
use draw::render::bitmap::PNGRenderer;
use draw::Shape::Line;
use draw::shape::LinePoint;

use crate::ltc_decoder::Sample;

fn print_timecode_decoder() {}

struct AudioImageXPos {
    pub width_u32: u32,
    pub width_f32: f32,
    pub start_index: usize,
    pub start_index_f32: f32,
    pub end_index: usize,
}

impl AudioImageXPos {
    const MaxWidth: usize = 4096 * 4;
    fn new(samples_len: usize, start_index: usize) -> Self {
        let start_index = max(0, min(start_index, samples_len - 1));
        let end_index = min(samples_len, start_index + Self::MaxWidth);
        let width = end_index - start_index;
        Self {
            width_u32: width as u32,
            width_f32: width as f32,
            start_index,
            start_index_f32: start_index as f32,
            end_index,
        }
    }

    fn index_to_x(&self, index: usize) -> f32 {
        index as f32 - self.start_index_f32
    }
}

struct AudioImageYPos {
    height_u32: u32,
    height_f32: f32,
    sample_height_factor: f32,
    sample_offset: f32,
}

impl AudioImageYPos {
    const AudioHeight: f32 = 200.0;
    const AudioMiddle: f32 = Self::AudioHeight / 2.0;
    const BitDrawHeight: f32 = 100.0;

    fn new<T: Sample>(samples: &[T]) -> Self {
        let min = samples.iter().min().unwrap().to_f32().unwrap();
        let max = samples.iter().max().unwrap().to_f32().unwrap();
        let size = max - min;
        let sample_height_factor = (Self::AudioHeight as f32) / size;
        let min = min * sample_height_factor;
        let height = Self::AudioHeight + Self::BitDrawHeight;
        Self {
            height_u32: height as u32,
            height_f32: height,
            sample_height_factor,
            sample_offset: -min,
        }
    }

    pub fn audio_sample_to_y<T: Sample>(&self, sample: &T) -> f32 {
        self.height_f32 - ((sample.to_f32().unwrap() * self.sample_height_factor) + self.sample_offset)
    }
}

struct BitPos {
    x_pos: f32,
    bit: bool,
}

struct ErrorPos {
    x_pos: f32,
    code: usize,
}

pub struct AudioImage {
    canvas: Canvas,
    x_pos: AudioImageXPos,
    y_pos: AudioImageYPos,
    threshold_lines: Vec<Point>,
    //Points where no treshold is available
    threshold_break: Vec<Point>,

    bit_pos: Vec<BitPos>,

    errors: Vec<ErrorPos>,
}

impl AudioImage {
    pub(crate) fn new<T: Sample>(samples: &[T], start_index: usize) -> Self {
        let x_pos = AudioImageXPos::new(samples.len(), start_index);
        let y_pos = AudioImageYPos::new(&samples);
        let mut s = Self {
            canvas: Canvas::new(x_pos.width_u32, y_pos.height_u32),
            x_pos,
            y_pos,
            threshold_lines: vec![],
            threshold_break: vec![],
            bit_pos: vec![],
            errors: vec![],
        };
        s.draw_background();
        s.draw_samples(samples);
        s
    }

    pub fn push_threashold<T: Sample>(&mut self, index: usize, threshold: Option<T>) {
        if index < self.x_pos.start_index || index > self.x_pos.end_index {
            return;
        }
        let x = self.x_pos.index_to_x(index);
        if threshold.is_none() {
            self.threshold_break.push(Point::new(x, 1.0));
            return;
        }

        let y = self.y_pos.audio_sample_to_y(&threshold.unwrap());

        self.threshold_lines.push(Point::new(x, y));
    }

    pub fn push_error(&mut self, index: usize, code: usize) {
        self.errors.push(ErrorPos {
            x_pos: self.x_pos.index_to_x(index),
            code,
        });
    }

    pub fn push_bit(&mut self, index: usize, bit: Option<bool>) {
        if bit.is_none() {
            return;
        }
        if index < self.x_pos.start_index || index > self.x_pos.end_index {
            return;
        }
        self.bit_pos.push(BitPos {
            x_pos: self.x_pos.index_to_x(index),
            bit: bit.unwrap(),
        })
    }


    fn draw_background(&mut self) {
        let mut rect = Drawing::new()
            // give it a shape
            .with_shape(Shape::Rectangle {
                width: self.x_pos.width_u32,
                height: self.y_pos.height_u32,
            })
            // give it a cool style
            .with_style(Style::filled(Color::black()));
        self.canvas.display_list.add(rect);
    }

    fn draw_samples<T: Sample>(&mut self, samples: &[T]) {
        if samples.len() == 0 { return; }
        let mut builder = LineBuilder::new(self.x_pos.index_to_x(self.x_pos.start_index), self.y_pos.audio_sample_to_y(&samples[self.x_pos.start_index]));
        for index in self.x_pos.start_index + 1..self.x_pos.end_index {
            builder = builder.line_to(self.x_pos.index_to_x(index), self.y_pos.audio_sample_to_y(&samples[index]));
        }
        self.canvas.display_list.add(Drawing::new().with_shape(builder.build()).with_style(Style::stroked(1, RGB { r: 255, g: 255, b: 255 })));
    }

    pub fn save(&mut self, name: &str) {
        self.draw_line(&self.threshold_lines.clone(), RGB { r: 0, g: 30, b: 255 });
        self.draw_line(&self.threshold_break.clone(), RGB { r: 255, g: 0, b: 30 });
        self.draw_bits();
        self.draw_errors();
        render::save(
            &self.canvas,
            format!("test_outputs/svg/{}.svg", name.to_string()).as_str(),
            SvgRenderer::new(),
        ).expect("Failed to save")
    }

    fn draw_line(&mut self, points: &[Point], color: RGB) {
        if points.len() == 0 {
            return;
        }
        let point = points[0];
        let mut builder = LineBuilder::new(point.x, point.y);
        for index in 1..points.len() {
            let point = points[index];
            builder = builder.line_to(point.x, point.y);
        }
        self.canvas.display_list.add(Drawing::new().with_shape(builder.build()).with_style(Style::stroked(1, color)));
    }

    fn draw_bits(&mut self) {
        self.bit_pos.iter().for_each(|point| {
            let color = if point.bit {
                RGB { r: 0, g: 255, b: 0 }
            } else {
                RGB { r: 255, g: 0, b: 0 }
            };

            let shape = LineBuilder::new(point.x_pos, 0.0).line_to(point.x_pos, AudioImageYPos::BitDrawHeight).build();
            let drawing = Drawing::new().with_shape(shape).with_style(Style::stroked(1, color));
            self.canvas.display_list.add(drawing);
        });
    }

    fn draw_errors(&mut self) {
        self.errors.iter().for_each(|error| {
            let color = match error.code {
                1 => RGB { r: 255, g: 0, b: 0 },
                2 => RGB { r: 0, g: 255, b: 0 },
                _ => RGB { r: 0, g: 0, b: 255 }
            };

            let shape = LineBuilder::new(error.x_pos, 0.0).line_to(error.x_pos, AudioImageYPos::BitDrawHeight / 2.0).build();
            let drawing = Drawing::new().with_shape(shape).with_style(Style::stroked(3, color));
            self.canvas.display_list.add(drawing);
        });
    }
}

#[cfg(test)]
pub mod tests {
    use rand::Rng;

    use crate::ltc_decoder::get_test_samples_48_14;
    use crate::ltc_decoder::print_decoder::AudioImage;

    #[test]
    fn test_image_drawing() {
        let mut samples = [0; 500];
        let mut rng = rand::thread_rng();
        for x in 0..500 {
            samples[x] = rng.gen_range(-200..200);
        }
        let mut ai = AudioImage::new(&samples, 0);
        ai.save("Random");
    }

    #[test]
    fn test_wave_drawing() {
        let (sampling_rate, samples) = get_test_samples_48_14();
        let samples = &samples[0..samples.len()];
        let mut ai = AudioImage::new(samples, 30);
        ai.push_bit(200, Some(true));
        ai.push_bit(400, Some(false));

        ai.save("Timecode4814");
    }
}