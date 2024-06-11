use std::collections::HashMap;

use jieba_rs::Jieba;
use regex::{Matches, Regex};

pub struct ChineseTokenizer {
    //分词正则
    regex: Regex,
    pub jieba: Jieba,
    pub min_word_length: usize,
    pub exclude_numbers: bool,
    pub max_words: usize,
    pub repeat: bool,
}

impl Default for ChineseTokenizer {
    fn default() -> Self {
        let regex = Regex::new("\\w[\\w']*").expect("Unable to compile tokenization regex");

        ChineseTokenizer {
            regex,
            jieba: Jieba::new(),
            min_word_length: 0,
            exclude_numbers: true,
            max_words: 200,
            repeat: false,
        }
    }
}

impl<'a> ChineseTokenizer {
    pub fn with_word(mut self, word: &str) -> Self {
        self.jieba.add_word(word, None, None);
        self
    }

    pub fn with_min_word_leng(mut self, size: usize) -> Self {
        self.min_word_length = size;
        self
    }

    pub fn with_max_words(mut self, size: usize) -> Self {
        self.max_words = size;
        self
    }

    fn tokenize(&'a self, text: &'a str) -> Matches {
        self.regex.find_iter(text)
    }

    pub fn get_word_frequencies(&'a self, text: &'a str) -> HashMap<&'a str, usize> {
        let mut frequencies = HashMap::new();

        for word in self.tokenize(text) {
            let words = self.jieba.cut(word.as_str(), false);
            for word in words {
                let size = word.chars().count();
                if size < self.min_word_length {
                    continue;
                }

                let entry = frequencies.entry(word).or_insert(0);
                *entry += 1;
            }
        }

        frequencies
    }

    pub fn get_normalized_word_frequencies(&'a self, text: &'a str) -> Vec<(&'a str, f32)> {
        let frequencies = self.get_word_frequencies(text);

        if frequencies.is_empty() {
            return vec![];
        }

        let max_freq = *frequencies
            .values()
            .max()
            .expect("Can't not find max frequency") as f32;

        let mut normalized_freqs: Vec<(&str, f32)> = frequencies
            .into_iter()
            .map(|(key, val)| (key, val as f32 / max_freq))
            .collect();

        normalized_freqs.sort_by(|a, b| {
            if a.1 != b.1 {
                (b.1).partial_cmp(&a.1).unwrap()
            } else {
                (a.0).partial_cmp(b.0).unwrap()
            }
        });

        if self.max_words > 0 {
            normalized_freqs.truncate(self.max_words);
        }

        normalized_freqs
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::Write,
    };

    use super::ChineseTokenizer;

    #[test]
    fn wukong() {
        let str = fs::read_to_string("text/wukong.txt")
            .unwrap()
            .lines()
            .map(|line| line.trim())
            .collect::<String>();
        let tokenlizer = ChineseTokenizer::default()
            .with_min_word_leng(2)
            .with_word("悟空传");
        let mut frequencies = tokenlizer
            .get_word_frequencies(&str)
            .into_iter()
            .collect::<Vec<_>>();
        frequencies.sort_by_key(|word| word.1);
        let mut path = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("text/wukong_count.txt")
            .unwrap();
        for (k, v) in &frequencies {
            path.write_all(format!("{k:?} {v:?}\n").as_bytes()).unwrap();
        }

        path.write_all(format!("all:{:?}\n", frequencies.len()).as_bytes())
            .unwrap();
    }
}
