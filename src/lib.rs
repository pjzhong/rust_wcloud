use std::{fs, path::PathBuf};

use ab_glyph::{point, FontVec, Point, PxScale};
use image::{GrayImage, Luma, Rgba, RgbaImage};
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

        let (summed_area_table, mut gray_buffer) = match size {
            WordCloudSize::FromDimensions { width, height } => {
                let buf = GrayImage::from_pixel(width, height, Luma([0]));
                let summed_area_table = buf.as_raw().iter().map(|e| *e as u32).collect::<Vec<_>>();

                (summed_area_table, buf)
            }
        };

        //  let mut final_words = Vec::with_capacity(words.len());
        // let mut last_freq = 1.0;
        // let skip_list = create_mask_skip_list(&gray_buffer);

        let mut rng = match self.rng_seed {
            Some(seed) => WyRand::new_seed(seed),
            None => WyRand::new(),
        };

        let first_word = words.first().expect("There are no words!");
        //使用第一个词的长宽来作为参考
        let font_size = {
            let rect_at_image_height = self.text_dimensions_at_font_size(
                first_word.0,
                PxScale::from(gray_buffer.height() as f32 * 0.95),
            );

            let height_ration =
                rect_at_image_height.height as f32 / rect_at_image_height.width as f32;
            let start_height = gray_buffer.width() as f32 * height_ration;

            start_height
        };
        let glyphs = text::text_to_glyphs(first_word.0, &self.font, PxScale::from(font_size));

        let pos = point(0.0, 0.0);
        text::draw_glyphs_to_gray_buffer(&mut gray_buffer, glyphs.clone(), &self.font, pos, false);

        let final_words = vec![Word {
            text: first_word.0,
            font: &self.font,
            font_size: PxScale::from(font_size),
            glyphs,
            rotated: false,
            position: pos,
            frequency: first_word.1,
            index: 0,
        }];

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

    fn text_dimensions_at_font_size(&self, text: &str, font_size: PxScale) -> Rect {
        let glyphs = text::text_to_glyphs(text, &self.font, font_size);
        Rect {
            width: glyphs.width + self.word_margin,
            height: glyphs.height + self.word_margin,
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
            let furthest_left = row
                .rposition(|p| p == &Luma::from([0]))
                .unwrap_or(img.width() as usize);
            let furthest_right = row.position(|p| p == &Luma::from([0])).unwrap_or(0);

            (furthest_left, furthest_right)
        })
        .collect()
}
