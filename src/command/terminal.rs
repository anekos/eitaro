
use std::path::Path;

use readline;

use dictionary::Dictionary;
use errors::AppError;
use printer::print_colored_opt;



pub fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<(), AppError> {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print_lines(&mut dic, word)
}

pub fn shell<T: AsRef<Path>>(dictionary_path: &T) -> Result<(), AppError> {
    use readline::Error::EndOfFile;

    let mut dic = Dictionary::new(dictionary_path);
    loop {
        match readline::readline("Eitaro> ") {
            Ok(ref input) => {
                readline::add_history(input)?;
                lookup_and_print_lines(&mut dic, input)?;
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
        print_colored_opt(dic.get(line.trim())?.as_ref().map(String::as_str))?;
    }
    Ok(())
}
