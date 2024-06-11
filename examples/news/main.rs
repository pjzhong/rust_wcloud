use std::time::Instant;

use rust_wcloud::{ChineseTokenizer, WordCloud, WordCloudSize};

/// 1.现在分词搞定了
/// 2.思考如何进行排版,词重叠在一起了。区域先排查下
pub fn main() {
    let wukong = include_str!("news.txt");

    let tokenlizer = ChineseTokenizer::default()
        .with_max_words(10000)
        .with_min_word_leng(2);

    let wordcloud = WordCloud::default()
        .with_tokenizer(tokenlizer)
        .with_font_from_path("examples/news/MSYH.TTC");

    let mask = WordCloudSize::FromDimensions {
        width: 1920,
        height: 1080,
    };

    // let color_func = |_word: &Word, rng: &mut WyRand| {
    //     let lightness = rng.generate_range(40..100);

    //     let col = Hsl::new(0.0, 0.0, lightness as f32 / 100.0);
    //     let rgb: Srgb = col.into_color();

    //     let raw: [u8; 3] = rgb.into_format()
    //         .into_raw();

    //     Rgba([raw[0], raw[1], raw[2], 1])
    // };

    let now = Instant::now();
    let wordcloud_image = wordcloud.generate_from_text(wukong, mask, 1.0);

    println!("Generated in {}ms", now.elapsed().as_millis());
    wordcloud_image
        .save("examples/news/cloud.png")
        .expect("Unable to save image");
}
