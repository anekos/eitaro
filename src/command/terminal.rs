
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use rustyline;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;
use crate::path::get_history_path;
use crate::screen;



pub fn lemmatize<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    if let Some(found) = dic.lemmatize(word)? {
        for it in found {
            println!("{}", it);
        }
    }
    Ok(())
}

pub fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print_lines(&mut dic, word)
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
                lookup_and_print_lines(&mut dic, input)?;
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

fn lookup_and_print_lines(dic: &mut Dictionary, s: &str) -> AppResultU {
    for line in s.lines() {
        let found = dic.get_smart(line.trim())?;
        screen::standard::print_opt(found)?;
    }
    Ok(())
}

fn append_history(line: &str) -> AppResultU {
    let path = get_history_path()?;
    let mut file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
