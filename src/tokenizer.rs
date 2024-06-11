use std::collections::{HashMap, HashSet};

use jieba_rs::Jieba;
use regex::Regex;

pub struct ChineseTokenizer {
    //分词正则
    regex: Regex,
    pub jieba: Jieba,
    pub filter: HashSet<String>,
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
            filter: Default::default(),
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

    pub fn with_filter(mut self, value: &[&str]) -> Self {
        self.filter = value.iter().map(|el| el.to_lowercase()).collect();

        self
    }

    pub fn with_exclude_numbers(mut self, value: bool) -> Self {
        self.exclude_numbers = value;
        self
    }

    fn tokenize(&'a self, text: &'a str) -> impl IntoIterator<Item = &str> {
        let mut iter: Box<dyn Iterator<Item = &str>> = Box::new(
            self.regex
                .find_iter(text)
                .map(|mat| mat.as_str())
                .filter(|str| !str.is_empty())
                .flat_map(|str| self.jieba.cut(str, false)),
        );

        if self.min_word_length > 0 {
            iter = Box::new(iter.filter(|str| {
                let chars = str.chars().count();
                chars >= self.min_word_length
            }));
        }

        if self.exclude_numbers {
            iter = Box::new(iter.filter(move |word| !word.chars().all(char::is_numeric)));
        }

        if !self.filter.is_empty() {
            iter = Box::new(iter.filter(|str| {
                let lower_case = str.to_lowercase();
                !self.filter.contains(&lower_case)
            }));
        }

        iter
    }

    pub fn get_word_frequencies(&'a self, text: &'a str) -> HashMap<&'a str, usize> {
        let mut frequencies = HashMap::new();

        for word in self.tokenize(text) {
            let entry = frequencies.entry(word).or_insert(0);
            *entry += 1;
        }

        let common_cased_map = Self::keep_common_case(&frequencies);

        common_cased_map
    }

    fn keep_common_case(map: &HashMap<&'a str, usize>) -> HashMap<&'a str, usize> {
        type CaseCounts<'a> = HashMap<&'a str, usize>;

        let mut common_cases = HashMap::<String, CaseCounts>::new();
        for (key, val) in map {
            common_cases
                .entry(key.to_lowercase())
                .or_default()
                .insert(key, *val);
        }

        common_cases
            .values()
            .map(|val| {
                let mut most_common_case: Vec<(&str, usize)> = val
                    .iter()
                    .map(|(case_key, case_val)| (*case_key, *case_val))
                    .collect();

                most_common_case.sort_by(|a, b| {
                    if a.1 != b.1 {
                        (b.1).partial_cmp(&a.1).unwrap()
                    } else {
                        (b.0).partial_cmp(a.0).unwrap()
                    }
                });

                let occurrence_sum = val.values().sum();

                (most_common_case.first().unwrap().0, occurrence_sum)
            })
            .collect()
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
        let str = fs::read_to_string("text/news.txt")
            .unwrap()
            .lines()
            .map(|line| line.trim())
            .collect::<String>();
        let tokenlizer = ChineseTokenizer::default().with_min_word_leng(2);
        let mut frequencies = tokenlizer
            .get_word_frequencies(&str)
            .into_iter()
            .collect::<Vec<_>>();
        frequencies.sort_by_key(|word| word.1);
        frequencies.reverse();
        let mut path = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("text/news_count.txt")
            .unwrap();
        for (k, v) in &frequencies {
            path.write_all(format!("{k}, {v}\n").as_bytes()).unwrap();
        }

        path.write_all(format!("all:{:?}\n", frequencies.len()).as_bytes())
            .unwrap();
    }
}
