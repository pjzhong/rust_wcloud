use std::{fs, path::PathBuf};

use ab_glyph::{point, FontVec, Point, PxScale};
use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
use nanorand::{Rng, WyRand};
use palette::{Hsl, IntoColor, Pixel, Srgb};
use sat::Rect;
use text::GlyphData;
pub use tokenizer::ChineseTokenizer;

mod sat;
mod text;
mod tokenizer;

pub struct Word<'a> {
    pub text: &'a str,
    pub font: &'a FontVec,
    pub font_size: PxScale,
    pub glyphs: GlyphData,
    pub rotated: bool,
    pub position: Point,
    pub frequency: f32,
    pub index: usize,
}

// TODO: Figure out a better way to structure this
pub enum WordCloudSize {
    FromDimensions { width: u32, height: u32 },
    FromMask(GrayImage),
}

pub struct WordCloud {
    tokenizer: ChineseTokenizer,
    background_color: Rgba<u8>,
    pub font: FontVec,
    min_font_size: f32,
    max_font_size: Option<f32>,
    font_step: f32,
    word_margin: u32,
    word_rotate_chance: f64,
    relative_font_scaling: f32,
    rng_seed: Option<u64>,
}

impl Default for WordCloud {
    fn default() -> Self {
        let font = FontVec::try_from_vec(include_bytes!("../fonts/Dengb.ttf").to_vec()).unwrap();

        WordCloud {
            tokenizer: ChineseTokenizer::default(),
            background_color: Rgba([0, 0, 0, 255]),
            font,
            min_font_size: 4.0,
            max_font_size: None,
            font_step: 1.0,
            word_margin: 2,
            word_rotate_chance: 0.10,
            relative_font_scaling: 0.5,
            rng_seed: None,
        }
    }
}

impl WordCloud {
    pub fn with_tokenizer(mut self, value: ChineseTokenizer) -> Self {
        self.tokenizer = value;
        self
    }

    pub fn with_min_font_size(mut self, value: f32) -> Self {
        self.min_font_size = value;
        self
    }

    pub fn with_font_from_path(mut self, path: impl Into<PathBuf>) -> Self {
        let font_file = fs::read(path.into()).expect("Unable to read font file");

        self.font = FontVec::try_from_vec(font_file).expect("Font file may be invalid");

        self
    }

    fn generate_from_word_positions(
        rng: &mut WyRand,
        width: u32,
        height: u32,
        word_positions: Vec<Word>,
        scale: f32,
        background_color: Rgba<u8>,
        color_func: fn(&Word, &mut WyRand) -> Rgba<u8>,
    ) -> RgbaImage {
        let mut final_image_buffer = RgbaImage::from_pixel(
            (width as f32 * scale) as u32,
            (height as f32 * scale) as u32,
            background_color,
        );

        for word in word_positions {
            let col = color_func(&word, rng);

            text::draw_glyphs_to_rgba_buffer(
                &mut final_image_buffer,
                word.glyphs,
                word.font,
                word.position,
                word.rotated,
                col,
            )
        }

        final_image_buffer
    }

    pub fn generate_from_text(&self, text: &str, size: WordCloudSize, scale: f32) -> RgbaImage {
        self.generate_from_text_with_color_func(text, size, scale, random_color_rgba)
    }

    pub fn generate_from_text_with_color_func(
        &self,
        text: &str,
        size: WordCloudSize,
        scale: f32,
        color_func: fn(&Word, &mut WyRand) -> Rgba<u8>,
    ) -> RgbaImage {
        let words = self.tokenizer.get_normalized_word_frequencies(text);

        let (mut summed_area_table, mut gray_buffer) = match size {
            WordCloudSize::FromDimensions { width, height } => {
                let buf = GrayImage::from_pixel(width, height, Luma([0]));
                let mut summed_area_table = vec![0; buf.len()];
                u8_to_u32_vec(&buf, &mut summed_area_table);
                (summed_area_table, buf)
            }
            WordCloudSize::FromMask(image) => {
                let mut table = image.as_ref().iter().map(|e| *e as u32).collect::<Vec<_>>();
                sat::to_summed_area_table(&mut table, image.width() as usize, 0);
                (table, image)
            }
        };

        let mut final_words = Vec::with_capacity(words.len());
        let mut last_freq = 1.0;
        let has_mask = matches!(WordCloudSize::FromMask, _size);
        let skip_list = if has_mask {
            Some(create_mask_skip_list(&gray_buffer))
        } else {
            None
        };

        let mut rng = match self.rng_seed {
            Some(seed) => WyRand::new_seed(seed),
            None => WyRand::new(),
        };

        let first_word = words.first().expect("There are no words!");
        // First, we determine an appropriate font size to start with based on the height of the canvas.
        // Rasterizing the first word in the sorted list at a font size of 95% the canvas height produces a
        // bounding rectangle we can use as a heuristic
        let mut font_size = {
            let rect_at_image_height = self.text_dimensions_at_font_size(
                first_word.0,
                PxScale::from(gray_buffer.height() as f32 * 0.55),
            );

            let height_ration =
                rect_at_image_height.height as f32 / rect_at_image_height.width as f32;

            let mut start_height = gray_buffer.width() as f32 * height_ration;

            if matches!(WordCloudSize::FromMask, _size) {
                let black_pixels = gray_buffer.as_raw().iter().filter(|p| **p == 0).count();
                let available_space = black_pixels as f32 / gray_buffer.len() as f32;
                start_height *= available_space;
            }

            start_height
        };

        for (word, freq) in &words {
            if !self.tokenizer.repeat && self.relative_font_scaling != 0.0 {
                font_size *= self.relative_font_scaling * (freq / last_freq)
                    + (1.0 - self.relative_font_scaling);
            }

            if font_size < self.min_font_size {
                break;
            }

            let (pos, glyphs, rotated) = match self.place_word(
                word,
                font_size,
                &gray_buffer,
                &skip_list,
                &summed_area_table,
                &mut rng,
            ) {
                Ok((pos, glyphs, rotate, new_font_size)) => {
                    font_size = new_font_size;
                    (pos, glyphs, rotate)
                }
                Err(new_font_size) => {
                    font_size = new_font_size;
                    continue;
                }
            };

            text::draw_glyphs_to_gray_buffer(
                &mut gray_buffer,
                glyphs.clone(),
                &self.font,
                pos,
                rotated,
            );

            final_words.push(Word {
                text,
                font: &self.font,
                font_size: PxScale::from(font_size),
                glyphs: glyphs.clone(),
                rotated,
                position: pos,
                frequency: *freq,
                index: final_words.len(),
            });

            u8_to_u32_vec(&gray_buffer, &mut summed_area_table);
            let start_row = (pos.y - 1.0).min(0.0) as usize;
            sat::to_summed_area_table(
                &mut summed_area_table,
                gray_buffer.width() as usize,
                start_row,
            );

            last_freq = *freq;
        }

        WordCloud::generate_from_word_positions(
            &mut rng,
            gray_buffer.width(),
            gray_buffer.height(),
            final_words,
            scale,
            self.background_color,
            color_func,
        )
    }

    fn place_word(
        &self,
        word: &str,
        mut font_size: f32,
        gray_buffer: &ImageBuffer<Luma<u8>, Vec<u8>>,
        skip_list: &Option<Vec<(usize, usize)>>,
        summed_area_table: &[u32],
        rng: &mut WyRand,
    ) -> Result<(Point, GlyphData, bool, f32), f32> {
        let initial_font_size = font_size;
        let mut shold_rotate = rng.generate::<u8>() <= (255.0 * self.word_rotate_chance) as u8;
        let mut tried_rotate = false;
        loop {
            let glyphs = text::text_to_glyphs(word, &self.font, PxScale::from(font_size));
            let rect = if shold_rotate {
                Rect {
                    width: glyphs.height + self.word_margin,
                    height: glyphs.width + self.word_margin,
                }
            } else {
                Rect {
                    width: glyphs.width + self.word_margin,
                    height: glyphs.height + self.word_margin,
                }
            };

            if rect.width > gray_buffer.width() || rect.height > gray_buffer.height() {
                if let Some(next_font_size) =
                    Self::check_font_size(font_size, self.font_step, self.min_font_size)
                {
                    font_size = next_font_size;
                    continue;
                } else {
                    return Err(font_size);
                }
            }
            let place_res = if let Some(skip_list) = &skip_list {
                sat::find_space_for_rect_masked(
                    summed_area_table,
                    gray_buffer.width(),
                    gray_buffer.height(),
                    skip_list,
                    &rect,
                    rng,
                )
            } else {
                sat::find_space_for_rect(
                    summed_area_table,
                    gray_buffer.width(),
                    gray_buffer.height(),
                    &rect,
                    rng,
                )
            };

            match place_res {
                Some(pos) => {
                    let half_margin = self.word_margin as f32 / 2.0;
                    let x = pos.x as f32 + half_margin;
                    let y = pos.y as f32 + half_margin;

                    return Ok((point(x, y), glyphs, shold_rotate, font_size));
                }
                None => {
                    if let Some(next_font_size) =
                        Self::check_font_size(font_size, self.font_step, self.min_font_size)
                    {
                        font_size = next_font_size;
                    } else if !tried_rotate {
                        //TODO 横着放不行，试下竖着放
                        shold_rotate = true;
                        tried_rotate = true;
                        font_size = initial_font_size;
                    } else {
                        return Err(font_size);
                    }
                }
            }
        }
    }

    fn text_dimensions_at_font_size(&self, text: &str, font_size: PxScale) -> Rect {
        let glyphs = text::text_to_glyphs(text, &self.font, font_size);
        Rect {
            width: glyphs.width + self.word_margin,
            height: glyphs.height + self.word_margin,
        }
    }

    fn check_font_size(font_size: f32, font_step: f32, min_font_size: f32) -> Option<f32> {
        let next_font_size = font_size - font_step;

        if next_font_size >= min_font_size && next_font_size > 0.0 {
            Some(next_font_size)
        } else {
            None
        }
    }
}

fn random_color_rgba(_: &Word, rng: &mut WyRand) -> Rgba<u8> {
    let hue: u8 = rng.generate_range(0..255);

    let col = Hsl::new(hue as f32, 1.0, 0.5);
    let rgb: Srgb = col.into_color();

    let raw: [u8; 3] = rgb.into_format().into_raw();

    Rgba([raw[0], raw[1], raw[2], 1])
}

fn create_mask_skip_list(img: &GrayImage) -> Vec<(usize, usize)> {
    img.rows()
        .map(|mut row| {
            let furthest_right = row
                .rposition(|p| p == &Luma::from([0]))
                .unwrap_or(img.width() as usize);
            let furthest_left = row.position(|p| p == &Luma::from([0])).unwrap_or(0);

            (furthest_left, furthest_right)
        })
        .collect()
}

fn u8_to_u32_vec(buffer: &GrayImage, dst: &mut [u32]) {
    for (i, el) in buffer.as_ref().iter().enumerate() {
        dst[i] = *el as u32;
    }
}
