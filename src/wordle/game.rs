use super::dictionary::Dictionary;
use super::errors::Result;

use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CharStatus {
    NotInWord,
    WrongPosition,
    RightPosition,
    NotUsed,
}

#[derive(Debug, PartialEq)]
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
            dictionary,
            word,
            status: GameStatus::new_game(),
            max_guesses,
            chars_status,
        })
    }

    fn guess_result(target_word: &str, guess_word: &str) -> GuessResult {
        assert!(target_word.len() == guess_word.len());

        let mut positions_map: HashMap<char, HashSet<usize>> = HashMap::new();
        for (pos, c) in target_word.chars().enumerate() {
            positions_map.entry(c).or_default().insert(pos);
        }

        let mut processed_positions: HashSet<usize> = HashSet::new();

        let mut chars_result: Vec<CharAndStatus> = guess_word
            .chars()
            .map(|c| CharAndStatus(c, CharStatus::NotInWord))
            .collect();

        // First, remove all positions that match
        target_word
            .chars()
            .zip(guess_word.chars())
            .enumerate()
            .filter_map(
                |(pos, (src, guess))| {
                    if src == guess {
                        Some((pos, src))
                    } else {
                        None
                    }
                },
            )
            .for_each(|(pos, c)| {
                assert!(chars_result[pos].0 == c);
                chars_result[pos].1 = CharStatus::RightPosition;
                // Now remove the right position from the list of positions map
                positions_map.get_mut(&c).unwrap().remove(&pos);
                processed_positions.insert(pos);
            });

        guess_word
            .chars()
            .enumerate()
            .filter(|(pos, _)| !processed_positions.contains(&pos))
            .for_each(|(pos, src_char)| {
                let char_status: CharStatus = match positions_map.get_mut(&src_char) {
                    None => CharStatus::NotInWord,
                    Some(ref positions) if positions.is_empty() => CharStatus::NotInWord,
                    Some(ref mut positions) => {
                        // We already processed all the "correct" positions
                        assert!(!positions.contains(&pos));
                        // Remove some element from the list of positions. It doesn't really matter
                        // which one.
                        let first_element: usize = *positions.iter().next().unwrap();
                        positions.remove(&first_element);
                        CharStatus::WrongPosition
                    }
                };

                chars_result[pos].1 = char_status;
            });

        GuessResult {
            _word: guess_word.into(),
            chars_result,
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

#[cfg(test)]
mod tests {
    use super::super::dictionary::EnglishDictionary;
    use super::*;

    fn set_up_game(word: &str) -> WordleGameImpl {
        let dict = EnglishDictionary::new(word.len()).unwrap();
        WordleGameImpl::new(Box::new(dict), word, 3).unwrap()
    }

    #[test]
    fn fails_after_specified_tries() {
        let word: String = "sound".into();
        let mut game = set_up_game(&word);
        let r1 = game.guess_word("wrong");
        assert!(matches!(r1, RoundResult::Continue(_)));
        let r2 = game.guess_word("wrong");
        assert!(matches!(r2, RoundResult::Continue(_)));
        let r3 = game.guess_word("wrong");
        assert!(matches!(r3, RoundResult::Lost(_, _)));
        if let RoundResult::Lost(_, result_word) = r3 {
            assert_eq!(result_word.to_lowercase(), word.to_lowercase());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn suceeds_correctly() {
        let word: String = "sound".into();
        let mut game = set_up_game(&word);
        let r1 = game.guess_word("wrong");
        assert!(matches!(r1, RoundResult::Continue(_)));
        let r2 = game.guess_word(&word);
        assert!(matches!(r2, RoundResult::Won(_, _)));
    }

    #[test]
    fn fails_with_wrong_number_of_letters() {
        let mut game = set_up_game("sound");
        let r1 = game.guess_word("toomanyletters");
        assert!(matches!(r1, RoundResult::Error(_)));
    }

    #[test]
    fn reports_letters_1() {
        let word: String = "sound".into();
        let mut game = set_up_game(&word);
        let guess = game.guess_word("wrong");
        assert!(matches!(guess, RoundResult::Continue(_)));

        if let RoundResult::Continue(status) = guess {
            assert_eq!(1, status.guesses.len());
            let guess_result = status.guesses.first().unwrap();
            let chars_result = &guess_result.chars_result;
            assert_eq!(CharAndStatus('W', CharStatus::NotInWord), chars_result[0]);
            assert_eq!(CharAndStatus('R', CharStatus::NotInWord), chars_result[1]);
            assert_eq!(
                CharAndStatus('O', CharStatus::WrongPosition),
                chars_result[2]
            );
            assert_eq!(
                CharAndStatus('N', CharStatus::RightPosition),
                chars_result[3]
            );
            assert_eq!(CharAndStatus('G', CharStatus::NotInWord), chars_result[4]);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn does_not_report_letters_several_times_1() {
        let word: String = "sound".into();
        let mut game = set_up_game(&word);
        let guess = game.guess_word("groot");
        assert!(matches!(guess, RoundResult::Continue(_)));

        if let RoundResult::Continue(status) = guess {
            assert_eq!(1, status.guesses.len());
            let guess_result = status.guesses.first().unwrap();
            let chars_result = &guess_result.chars_result;
            assert_eq!(CharAndStatus('G', CharStatus::NotInWord), chars_result[0]);
            assert_eq!(CharAndStatus('R', CharStatus::NotInWord), chars_result[1]);
            assert_eq!(
                CharAndStatus('O', CharStatus::WrongPosition),
                chars_result[2]
            );
            assert_eq!(CharAndStatus('O', CharStatus::NotInWord), chars_result[3]);
            assert_eq!(CharAndStatus('T', CharStatus::NotInWord), chars_result[4]);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn does_not_report_letters_several_times_2() {
        let word: String = "sound".into();
        let mut game = set_up_game(&word);
        let guess = game.guess_word("boost");
        assert!(matches!(guess, RoundResult::Continue(_)));

        if let RoundResult::Continue(status) = guess {
            assert_eq!(1, status.guesses.len());
            let guess_result = status.guesses.first().unwrap();
            let chars_result = &guess_result.chars_result;
            assert_eq!(CharAndStatus('B', CharStatus::NotInWord), chars_result[0]);
            assert_eq!(
                CharAndStatus('O', CharStatus::RightPosition),
                chars_result[1]
            );
            assert_eq!(CharAndStatus('O', CharStatus::NotInWord), chars_result[2]);
            assert_eq!(
                CharAndStatus('S', CharStatus::WrongPosition),
                chars_result[3]
            );
            assert_eq!(CharAndStatus('T', CharStatus::NotInWord), chars_result[4]);
        } else {
            unreachable!();
        }
    }
}
