
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use rustyline;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;
use crate::path::get_history_path;
use crate::screen;




pub fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str, color: bool, n: Option<usize>) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print_lines(&mut dic, word, color, n)
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
                lookup_and_print_lines(&mut dic, input, true, None)?;
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

fn lookup_and_print_lines(dic: &mut Dictionary, s: &str, color: bool, limit: Option<usize>) -> AppResultU {
    for line in s.lines() {
        let mut found = dic.get_smart(line.trim())?;
        if let Some(limit) = limit {
            found = found.map(|it| it.into_iter().take(limit).collect());
        }
        if color {
            screen::color::print_opt(found)?;
        } else {
            screen::plain::print_opt(found)?;
        }
    }
    Ok(())
}

fn append_history(line: &str) -> AppResultU {
    let path = get_history_path()?;
    let mut file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
