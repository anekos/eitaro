use std::path::Path;

use deco::{dprint, dprintln};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rustyline;
use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;


#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Minimum level (1 to 10)
    #[structopt(long = "min")]
    min: Option<u8>,
    /// Maximum level
    #[structopt(long = "max")]
    max: Option<u8>,
}

pub fn play<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> AppResultU {
    let dic = Dictionary::new(dictionary_path);
    let mut rng = thread_rng();
    let words = dic.wordle_words(opt.min.unwrap_or(0), opt.max.unwrap_or(100))?;

    let mut correct = words.choose(&mut rng).unwrap();
    let mut round = 1;

    help();

    let config = rustyline::config::Builder::new().build();
    let mut editor = rustyline::Editor::<()>::with_config(config);

    loop {
        match editor.readline(&format!("{}/{}❔ ", round, 6)) {
            Ok(ref input) => {
                match show_hints(&correct, input) {
                    Ok(ok) => {
                        if ok {
                            correct = words.choose(&mut rng).unwrap();
                            round = 1
                        } else {
                            round += 1;
                            if 6 < round {
                                println!("😿 {}", correct);
                                correct = words.choose(&mut rng).unwrap();
                                round = 1
                            }
                        }
                    }
                    Err(msg) => println!("😼 {}", msg)
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!();
                break;
            },
            Err(_) => continue,
        }
    }

    Ok(())
}


fn help() {
    dprintln!([on_yellow bold "w" on_black "eary" ! "\n  The letter W is in the word and in the correct spot."]);
    dprintln!([bold on_black "p" on_red "i" on_black "lls" ! "\n  The letter I is in the word but in the wrong spot."]);
    dprintln!([bold "vag" on_black "u" ! bold "e" ! "\n  The letter U is not in the word in any spot."]);
}


fn show_hints(correct: &str, input: &str) -> Result<bool, &'static str> {
    if input.len() < 5 {
        return Err("Too short");
    }
    if 5 < input.len() {
        return Err("Too long");
    }

    let mut cs = correct.chars();
    let mut is = input.chars();

    if correct == input {
        print!("😸 ");
    } else {
        print!("😹 ");
    }

    for _ in 0..5 {
        let c = cs.next().unwrap();
        let i = is.next().unwrap();

        if c == i {
            dprint!([on_yellow bold "{}" !] i);
        } else if correct.find(i).is_some() {
            dprint!([on_red bold "{}" !] i);
        } else {
            dprint!([on_black bold "{}" !] i);
        }
    }
    println!();

    Ok(correct == input)
}
