#[macro_use]
extern crate error_chain;

use clap::Parser;
use colored::*;
use std::io::{self, Write};

mod wordle;
use wordle::{CharAndStatus, CharStatus, Dictionary, GuessResult, RoundResult, WordleGame};

fn colored_char_by_status(cs: &CharAndStatus) -> ColoredString {
    let CharAndStatus(c, status) = *cs;
    let c = c.to_string();
    match status {
        CharStatus::NotInWord => c.black().on_red(),
        CharStatus::WrongPosition => c.black().on_yellow(),
        CharStatus::RightPosition => c.black().on_green(),
        CharStatus::NotUsed => c.white(),
    }
}

fn print_chars_with_status(chars_status: &[CharAndStatus]) {
    let colored_string = chars_status
        .iter()
        .map(colored_char_by_status)
        .map(|cs| cs.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", colored_string);
}

fn print_guess_result(result: &GuessResult) {
    print_chars_with_status(&result.chars_result);
}

fn game_loop(game: &mut dyn WordleGame) -> wordle::Result<()> {
    loop {
        print!("Available letters: ");
        print_chars_with_status(&game.chars_status());

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

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// Numbers of letters in the word to guess
    #[clap(long, default_value_t=5)]
    word_size: usize,
    /// Number of tries
    #[clap(long, default_value_t=6)]
    num_guesses: usize,
    /// Disable using the dictionary for matching words
    #[clap(long)]
    dont_use_dictionary: bool,
}

fn do_main() -> wordle::Result<()> {
    let args = Cli::parse();

    let dict = wordle::EnglishDictionary::new(args.word_size)?;
    let word = dict.get_random_word(args.word_size)?;
    // let word = "silos";
    // println!("Word is: {}", word);
    let mut game = wordle::WordleGameImpl::new(Box::new(dict), &word, args.num_guesses)?;
    game_loop(&mut game)
}

fn main() {
    match do_main() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }
}
