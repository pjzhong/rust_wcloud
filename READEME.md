# rust_wcloud
Generate beautiful word clouds with support for masks, custom fonts, custom coloring functions, And Chinese Word Segmentation(support by [jieba-rs](https://github.com/messense/jieba-rs)).


## Usage

`rust_wcloud` can be used as both a command-line application and a library.

### Command-line

The binary runs under the `rust_wcloud` name. The only required input is the text used to generate the word cloud, which can be provided via the `--text` flag or through `stdin`.

`$ rust_wcloud --text file.txt -o cloud.png`

`$ echo 'Clouds are awesome!' | rust_wcloud --output cloud.png`

For a list of all options, use `rust_wcloud --help`.

Here's a basic example:

```rust
use rust_wcloud::{WordCloud, WordCloudSize};


fn main() {
    let text = r#"
        An arcus cloud is a low, horizontal cloud formation,
        usually appearing as an accessory cloud to a cumulonimbus.
        Roll clouds and shelf clouds are the two main types of arcus
        clouds. They most frequently form along the leading edge or
        gust fronts of thunderstorms; some of the most dramatic arcus
        formations mark the gust fronts of derecho-producing convective
        systems. Roll clouds may also arise in the absence of
        thunderstorms, forming along the shallow cold air currents of
        some sea breeze boundaries and cold fronts.
    "#;

    let wordcloud = WordCloud::default()
        .with_rng_seed(0);

    let size = WordCloudSize::FromDimensions { width: 1000, height: 500 };
    let wordcloud_image = wordcloud.generate_from_text(text, size, 1.0);

    wordcloud_image.save("cloud.png")
        .expect("Unable to save image");
}
```

![](examples/cloud.png)

## Gallery

<p>
<img src="examples/mask/1_mask.png" width="49%" />
<img src="examples/mask/2_mask.png" width="49%" />
    
<img src="examples/mask/3_mask.png" width="49%" />
</p>

## Credit

This project is largely based on the [wcloud](https://github.com/isaackd/wcloud) project by [@isaackd](https://github.com/isaackd). 

## License

`rust_wcloud` is released under the [MIT License](https://github.com/isaackd/wcloud-dev/blob/main/LICENSE). 