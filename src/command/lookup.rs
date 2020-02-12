
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use rustyline;

use crate::dictionary::Dictionary;
use crate::errors::{AppResult, AppResultU};
use crate::path::get_history_path;
use crate::screen;




pub fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str, color: bool, n: Option<usize>) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print_lines(&mut dic, word, color, n, true)
}

pub fn shell<T: AsRef<Path>>(dictionary_path: &T, prompt: &str) -> AppResultU {
    let config = rustyline::config::Builder::new()
        .auto_add_history(true)
        .build();
    let mut editor = rustyline::Editor::<()>::with_config(config);
    editor.load_history(&get_history_path()?)?;

    let mut dic = Dictionary::new(dictionary_path);
    loop {
        match editor.readline(prompt) {
            Ok(ref input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                lookup_and_print_lines(&mut dic, input, true, None, false)?;
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

fn lookup_and_print_lines(dic: &mut Dictionary, s: &str, color: bool, limit: Option<usize>, pager: bool) -> AppResultU {
    for line in s.lines() {
        lookup_and_print(dic, line, color, limit, true, pager)?;
    }
    Ok(())
}

fn lookup_and_print(dic: &mut Dictionary, word: &str, color: bool, limit: Option<usize>, correction: bool, pager: bool) -> AppResultU {
    let mut found = dic.get_smart(word.trim())?;
    if let Some(limit) = limit {
        found = found.map(|it| it.into_iter().take(limit).collect());
    }

    if let Some(found) = found {
        if pager {
            setup_pager();
        }
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

fn setup_pager() {
    pager::Pager::with_default_pager("less --quit-if-one-screen --RAW-CONTROL-CHARS").setup();
}
