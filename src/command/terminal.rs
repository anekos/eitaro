
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, stdin};
use std::io::Write;
use std::path::Path;

use readline;

use dictionary::Dictionary;
use errors::AppError;
use printer::{print_colored_content, print_colored_opt};

use path::get_history_path;



pub fn color() -> Result<(), AppError> {
    let mut buffer = String::new();
    let stdin = stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    print_colored_content(&buffer)
}

pub fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<(), AppError> {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print_lines(&mut dic, word)
}

pub fn shell<T: AsRef<Path>>(dictionary_path: &T) -> Result<(), AppError> {
    use readline::Error::EndOfFile;

    restore_history()?;

    let mut dic = Dictionary::new(dictionary_path);
    loop {
        match readline::readline("Eitaro> ") {
            Ok(ref input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                lookup_and_print_lines(&mut dic, input)?;
                let _ = append_history(input);
            },
            Err(EndOfFile) => {
                println!();
                break;
            },
            Err(_) => continue,
        }
    }

    Ok(())
}

fn lookup_and_print_lines(dic: &mut Dictionary, s: &str) -> Result<(), AppError> {
    for line in s.lines() {
        let found = dic.get(line.trim())?;
        print_colored_opt(&found)?;
    }
    Ok(())
}

fn append_history(line: &str) -> Result<(), AppError> {
    readline::add_history(line)?;

    let path = get_history_path()?;
    let mut file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

fn restore_history() -> Result<(), AppError> {
    let path = get_history_path()?;
    if path.exists() {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(&mut file);
        for line in reader.lines() {
            readline::add_history(&line?)?;
        }
    }
    Ok(())
}
