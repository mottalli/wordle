use super::errors::Result;

use rand::Rng;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};

pub trait Dictionary {
    fn get_random_word(&self, size: u8) -> Result<String>;
    fn contains_word(&self, word: &str) -> bool;
    fn available_chars(&self) -> Vec<char>;
}

pub struct EnglishDictionary {
    words: HashSet<String>,
    word_size: u8,
}

impl EnglishDictionary {
    pub fn new(word_size: u8) -> Result<EnglishDictionary> {
        let mut words = HashSet::<String>::new();

        let file = File::open(format!("dictionaries/english/{}.txt", word_size))?;
        let lines = io::BufReader::new(file).lines();
        lines.into_iter().filter_map(|w| w.ok()).for_each(|w| {
            words.insert(w.to_uppercase());
        });

        if words.is_empty() {
            Err("Error loading dictionary, dictionary is empty".into())
        } else {
            Ok(EnglishDictionary {
                words: words,
                word_size: word_size,
            })
        }
    }
}

impl Dictionary for EnglishDictionary {
    fn get_random_word(&self, size: u8) -> Result<String> {
        if self.word_size != size {
            return Err(format!(
                "Tried to get a word of {} characters using a dictionary of {} characters",
                size, self.word_size
            )
            .into());
        }

        let num_words = self.words.len();

        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..num_words);

        Ok(self.words.iter().nth(r).unwrap().into())
    }

    fn contains_word(&self, word: &str) -> bool {
        self.words.contains(&word.to_uppercase())
    }

    fn available_chars(&self) -> Vec<char> {
        vec![
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ]
    }
}