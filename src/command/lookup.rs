
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use rustyline;
use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::{AppResult, AppResultU};
use crate::path::get_history_path;
use crate::screen;



const DEFAULT_PROMPT: &str = "Eitaro> ";


#[derive(Debug, StructOpt)]
pub struct LookupOpt {
    /// Word
    word: String,
    /// No Color
    #[structopt(long="no-color", parse(from_flag = std::ops::Not::not))]
    color: bool,
    /// Take only n related entries
    #[structopt(short, long)]
    n: Option<usize>
}


#[derive(Debug, Default, StructOpt)]
pub struct ShellOpt {
    /// Prompt text
    #[structopt(short, long, env="EITARO_PROMPT")]
    prompt: Option<String>,
}


pub fn lookup<T: AsRef<Path>>(opt: LookupOpt, dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print(&mut dic, &opt.word, opt.color, opt.n, true, true)
}

pub fn shell<T: AsRef<Path>>(opt: ShellOpt, dictionary_path: &T) -> AppResultU {
    let config = rustyline::config::Builder::new()
        .auto_add_history(true)
        .build();
    let mut editor = rustyline::Editor::<()>::with_config(config);
    let history_path = get_history_path()?;
    if history_path.exists() {
        editor.load_history(&history_path)?;
    }

    let mut dic = Dictionary::new(dictionary_path);
    let prompt = opt.prompt.unwrap_or_else(|| DEFAULT_PROMPT.to_owned());
    loop {
        match editor.readline(&prompt) {
            Ok(ref input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                lookup_and_print(&mut dic, input, true, None, true, true)?;
                let _ = append_history(input);
            },
            Err(rustyline::error::ReadlineError::Eof) => {
                println!();
                break;
            },
            Err(_) => continue,
        }
    }

    editor.save_history(&get_history_path()?)?;
    Ok(())
}

fn lookup_and_print(dic: &mut Dictionary, word: &str, color: bool, limit: Option<usize>, correction: bool, pager: bool) -> AppResultU {
    let mut found = if word.starts_with('/') {
        dic.search(word[1..].trim())
    } else {
        dic.get_smart(word.trim())
    }?;

    if let Some(limit) = limit {
        found = found.map(|it| it.into_iter().take(limit + 1).collect());
    }

    if let Some(found) = found {
        if color {
            screen::color::print(found)?;
        } else {
            screen::plain::print(found)?;
        }
        return Ok(())
    }

    if correction {
        if let Some(found) = untypo(dic, word)? {
            return lookup_and_print(dic, &found, color, limit, false, pager);
        }
    }

    if color {
        screen::color::print_not_found();
    } else {
        screen::plain::print_not_found();
    }

    Ok(())
}

fn untypo(dic: &mut Dictionary, word: &str) -> AppResult<Option<String>> {
    let candidates = dic.correct(word);

    if candidates.is_empty() {
        return Ok(None)
    }

    for (index, candidate) in candidates.iter().enumerate() {
        if 0 < index {
            print!(" ");
        }
        print!("[{}] {} ", index, candidate);
    }
    println!("[x] Cancel");

    loop {
        print!("Choose a word [0]: ");
        stdout().flush()?;
        let mut choosen = "".to_owned();
        stdin().read_line(&mut choosen).unwrap();
        let choosen = choosen.trim();
        if choosen == "x" {
            return Ok(None)
        }
        let choosen = if choosen.is_empty() {
            Some(0)
        } else {
            choosen.parse::<usize>().ok()
        };
        let choosen = choosen.and_then(|it| candidates.get(it));
        if let Some(choosen) = choosen {
            return Ok(Some(choosen.to_owned()));
        }

        println!("Invalid input!");
    }
}


fn append_history(line: &str) -> AppResultU {
    let path = get_history_path()?;
    let mut file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
