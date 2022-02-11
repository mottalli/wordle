use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, Write};

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
        }
    }
}

use errors::*;

trait Dictionary {
    fn get_random_word(&self, size: u8) -> Result<String>;
    fn contains_word(&self, word: &str) -> bool;
    fn available_letters(&self) -> Vec<char>;
}

#[derive(PartialEq)]
enum CharStatus {
    NotInWord,
    WrongPosition,
    RightPosition,
}

struct CharResult {
    char: char,
    status: CharStatus,
}

struct GuessResult {
    word: String,
    chars_result: Vec<CharResult>,
}

impl GuessResult {
    fn is_won(&self) -> bool {
        self.chars_result
            .iter()
            .all(|r| r.status == CharStatus::RightPosition)
    }
}

struct GameStatus {
    guesses: Vec<GuessResult>,
}

impl GameStatus {
    fn new_game() -> GameStatus {
        GameStatus {
            guesses: Vec::new(),
        }
    }
}

enum RoundResult<'a> {
    Error(String),
    Won(&'a GameStatus, String),
    Lost(&'a GameStatus, String),
    Continue(&'a GameStatus),
}

trait WordleGame {
    fn guess_word<'a>(&'a mut self, word: &str) -> RoundResult<'a>;
    fn max_guesses(&self) -> usize;
}

struct WordleGameImpl {
    dictionary: Box<dyn Dictionary>,
    word: String,
    status: GameStatus,
    max_guesses: usize,
}

impl WordleGameImpl {
    fn new(
        dictionary: Box<dyn Dictionary>,
        word: &str,
        max_guesses: usize,
    ) -> Result<WordleGameImpl> {
        let word = word.to_uppercase();

        Ok(WordleGameImpl {
            dictionary: dictionary,
            word: word,
            status: GameStatus::new_game(),
            max_guesses: max_guesses,
        })
    }

    fn guess_result(target_word: &str, guess_word: &str) -> GuessResult {
        assert!(target_word.len() == guess_word.len());

        let mut positions_map: HashMap<char, HashSet<usize>> = HashMap::new();
        for (pos, c) in target_word.chars().enumerate() {
            positions_map.entry(c).or_default().insert(pos);
        }

        let chars_result: Vec<CharResult> = guess_word
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

                CharResult {
                    char: src_char,
                    status: char_status,
                }
            })
            .collect();

        GuessResult {
            word: guess_word.into(),
            chars_result: chars_result,
        }
    }
}

impl WordleGame for WordleGameImpl {
    fn max_guesses(&self) -> usize {
        self.max_guesses
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

struct EnglishDictionary {
    words: HashSet<String>,
    word_size: u8,
}

impl EnglishDictionary {
    fn new(word_size: u8) -> Result<EnglishDictionary> {
        let mut words = HashSet::<String>::new();

        let file = File::open(format!("{}.txt", word_size))?;
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

    fn available_letters(&self) -> Vec<char> {
        vec![
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ]
    }
}

fn print_guess_result(result: &GuessResult) {
    let mut s = String::new();
    for r in result.chars_result.iter() {
        let char_output: String = match r.status {
            CharStatus::NotInWord => format!("/{}/", r.char),
            CharStatus::WrongPosition => format!("?{}?", r.char),
            CharStatus::RightPosition => format!("*{}*", r.char),
        };
        s.push_str(&char_output);
        s.push(' ');
    }
    println!("{}", s);
}

fn game_loop(game: &mut dyn WordleGame) -> Result<()> {
    loop {
        print!("Enter a word!: ");
        let _ = io::stdout().flush();
        let word: String = {
            let mut word = String::new();
            io::stdin().read_line(&mut word)?;
            word.trim().into()
        };

        let round_result = game.guess_word(&word);
        match round_result {
            RoundResult::Error(s) => eprintln!("Error: {}", s),
            RoundResult::Won(ref status, word) => {
                print_guess_result(status.guesses.last().unwrap());
                println!("Won! The word was {}", word);
                break;
            }
            RoundResult::Lost(ref status, word) => {
                print_guess_result(status.guesses.last().unwrap());
                println!("Lost :( The word was {}", word);
                break;
            }
            RoundResult::Continue(ref status) => {
                print_guess_result(status.guesses.last().unwrap());
                println!("Moving on...");
            }
        }
    }

    Ok(())
}

fn do_main() -> Result<()> {
    let word_size = 5;
    let dict = EnglishDictionary::new(word_size)?;
    let word = dict.get_random_word(word_size)?;
    //let word = "silos";
    let mut game = WordleGameImpl::new(Box::new(dict), &word, 6)?;
    game_loop(&mut game)
}

fn main() {
    match do_main() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }
}
