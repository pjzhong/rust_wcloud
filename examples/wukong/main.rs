use std::time::Instant;

use rust_wcloud::{ChineseTokenizer, WordCloud, WordCloudSize};

/// 1.现在分词搞定了
/// 2.思考如何进行排版,词重叠在一起了。区域先排查下
pub fn main() {
    let wukong = include_str!("wukong.txt");
    let tokenlizer = ChineseTokenizer::default()
        .with_max_words(1000)
        .with_min_word_leng(2)
        .with_word("悟空传");

    let wordcloud = WordCloud::default().with_tokenizer(tokenlizer);

    let mask = WordCloudSize::FromDimensions {
        width: 1920,
        height: 1080,
    };

    let now = Instant::now();
    let wordcloud_image = wordcloud.generate_from_text(wukong, mask, 1.0);

    println!("Generated in {}ms", now.elapsed().as_millis());
    wordcloud_image
        .save("examples/wukong/cloud.png")
        .expect("Unable to save image");
}
