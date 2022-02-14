use super::errors::Result;
use super::dictionary::Dictionary;

use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CharStatus {
    NotInWord,
    WrongPosition,
    RightPosition,
    NotUsed
}

pub struct CharAndStatus(pub char, pub CharStatus);

pub struct GuessResult {
    _word: String,
    pub chars_result: Vec<CharAndStatus>,
}

impl GuessResult {
    fn is_won(&self) -> bool {
        self.chars_result
            .iter()
            .all(|cs| cs.1 == CharStatus::RightPosition)
    }
}

pub struct GameStatus {
    pub guesses: Vec<GuessResult>,
}

impl GameStatus {
    fn new_game() -> GameStatus {
        GameStatus {
            guesses: Vec::new(),
        }
    }
}

pub enum RoundResult<'a> {
    Error(String),
    Won(&'a GameStatus, String),
    Lost(&'a GameStatus, String),
    Continue(&'a GameStatus),
}

pub trait WordleGame {
    fn guess_word<'a>(&'a mut self, word: &str) -> RoundResult<'a>;
    fn max_guesses(&self) -> usize;
    fn chars_status(&self) -> Vec<CharAndStatus>;
}

pub struct WordleGameImpl {
    dictionary: Box<dyn Dictionary>,
    word: String,
    status: GameStatus,
    max_guesses: usize,
    chars_status: HashMap<char, CharStatus>,
}

impl WordleGameImpl {
    pub fn new(
        dictionary: Box<dyn Dictionary>,
        word: &str,
        max_guesses: usize,
    ) -> Result<WordleGameImpl> {
        let word = word.to_uppercase();
        let chars_status: HashMap<char, CharStatus> = dictionary
            .available_chars()
            .iter()
            .map(|&c| (c, CharStatus::NotUsed))
            .collect();

        Ok(WordleGameImpl {
            dictionary: dictionary,
            word: word,
            status: GameStatus::new_game(),
            max_guesses: max_guesses,
            chars_status: chars_status,
        })
    }

    fn guess_result(target_word: &str, guess_word: &str) -> GuessResult {
        assert!(target_word.len() == guess_word.len());

        let mut positions_map: HashMap<char, HashSet<usize>> = HashMap::new();
        for (pos, c) in target_word.chars().enumerate() {
            positions_map.entry(c).or_default().insert(pos);
        }

        let chars_result: Vec<CharAndStatus> = guess_word
            .chars()
            .enumerate()
            .map(|(position, src_char)| {
                //for (c, poss) in positions_map.iter() {
                //    println!("{}: {:?}", c, poss);
                //}

                let char_status: CharStatus = match positions_map.get_mut(&src_char) {
                    None => CharStatus::NotInWord,
                    Some(ref mut positions) => {
                        if positions.contains(&position) {
                            positions.remove(&position);
                            CharStatus::RightPosition
                        } else {
                            // Remove the first position from the set, if any
                            //if let Some(p) = positions.iter().next().map(|p| *p) {
                            //    positions.remove(&p);
                            //}
                            if positions.is_empty() {
                                CharStatus::NotInWord
                            } else {
                                CharStatus::WrongPosition
                            }
                        }
                    }
                };

                CharAndStatus(src_char, char_status)
            })
            .collect();

        GuessResult {
            _word: guess_word.into(),
            chars_result: chars_result,
        }
    }
}

impl WordleGame for WordleGameImpl {
    fn max_guesses(&self) -> usize {
        self.max_guesses
    }

    fn chars_status(&self) -> Vec<CharAndStatus> {
        self.dictionary
            .available_chars()
            .iter()
            .map(|&c| CharAndStatus(c, *self.chars_status.get(&c).unwrap()))
            .collect()
    }

    fn guess_word<'a>(&'a mut self, word: &str) -> RoundResult<'a> {
        let word = word.to_uppercase();

        let num_guesses = self.status.guesses.len();
        if num_guesses == self.max_guesses {
            return RoundResult::Lost(&self.status, self.word.clone());
        } else if word.len() != self.word.len() {
            return RoundResult::Error(format!("Word must be {} characters!", self.word.len()));
        } else if !self.dictionary.contains_word(&word) {
            return RoundResult::Error(format!("Word \"{}\" is not in the dictionary!", word));
        }

        let result = WordleGameImpl::guess_result(&self.word, &word);

        // Update internal cache
        for cs in result.chars_result.iter() {
            let CharAndStatus(guessed_char, guess_status) = *cs;

            self.chars_status.entry(guessed_char).and_modify(|entry| {
                let new_status: CharStatus = match (*entry, guess_status) {
                    (CharStatus::NotUsed, s) => s,
                    (CharStatus::RightPosition, _) => CharStatus::RightPosition,
                    (_, CharStatus::RightPosition) => CharStatus::RightPosition,
                    (CharStatus::WrongPosition, _) => CharStatus::WrongPosition,
                    (_, s) => s,
                };
                *entry = new_status;
            });
        }

        let won: bool = result.is_won();
        self.status.guesses.push(result);

        if won {
            RoundResult::Won(&self.status, self.word.clone())
        } else if self.status.guesses.len() == self.max_guesses {
            return RoundResult::Lost(&self.status, self.word.clone());
        } else {
            RoundResult::Continue(&self.status)
        }
    }
}
