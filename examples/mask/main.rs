use std::{fs, time::Instant};

use image::Rgba;
use nanorand::{Rng, WyRand};
use palette::{Hsl, IntoColor, Pixel, Srgb};
use rust_wcloud::{ChineseTokenizer, Word, WordCloud, WordCloudSize};

pub fn main() {
    let wukong = include_str!("news.txt");

    let tokenlizer = ChineseTokenizer::default()
        .with_max_words(10000)
        .with_filter(&["一个"])
        .with_min_word_leng(2);

    let wordcloud = WordCloud::default().with_tokenizer(tokenlizer);

    for (i, path) in ["spin.jpg", "stormtrooper_mask.png", "avatar.jfif"]
        .iter()
        .enumerate()
    {
        let mask_buf = fs::read(format!("examples/mask/{path}")).expect("Unable to read font file");
        let mask_img = image::load_from_memory(&mask_buf)
            .expect("Unable to load mask from memory")
            .to_luma8();

        let mask = WordCloudSize::FromMask(mask_img);

        let color_func = |_word: &Word, rng: &mut WyRand| {
            let lightness = rng.generate_range(40..100);

            let col = Hsl::new(1.0, 0.0, lightness as f32 / 100.0);
            let rgb: Srgb = col.into_color();

            let raw: [u8; 3] = rgb.into_format().into_raw();

            Rgba([raw[0], raw[1], raw[2], 1])
        };

        let now = Instant::now();

        let wordcloud_image =
            wordcloud.generate_from_text_with_color_func(wukong, mask, 1.0, color_func);
        println!("Generated in {}ms", now.elapsed().as_millis());
        let i = i + 1;
        wordcloud_image
            .save(format!("examples/mask/{i}_mask.png"))
            .expect("Unable to save image");
    }
}
