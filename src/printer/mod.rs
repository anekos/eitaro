
use colored::*;

pub mod parser;

use dictionary::Entry;
use errors::AppError;
use self::parser::{Text, parse};



pub fn print_colored_opt(entries: &Option<Vec<Entry>>, ignore_none: bool) -> Result<(), AppError> {
    if let Some(entries) = entries.as_ref() {
        for entry in entries {
            print_colored(entry)?;
        }
    } else {
        if !ignore_none {
            println!("{}", "Not found".white().on_red().bold());
        }
    }
    Ok(())
}

fn print_colored(entry: &Entry) -> Result<(), AppError> {
    print_key(&entry.key);
    print_colored_content(&entry.content)
}

pub fn print_colored_content(content: &str) -> Result<(), AppError> {
    let text = parse(content)?;
    for it in &text {
        print_text(it);
    }
    Ok(())
}

fn print_key(key: &str) {
    println!("{}", key.black().on_yellow().bold());
}

fn print_text(text: &Text) {
    use self::Text::*;

    match text {
        Annot(s) => print!("{} ", s.yellow()),
        Class(s) => print!("{} ", s.blue()),
        Example(s) => print!(" {} ", s.green()),
        LineBreak => println!(),
        Note(s) => print!(" {}", s.cyan()),
        Plain(s) => print!("{}", s.white().bold()),
        Tag(s) => print!("{}", s.red().bold()),
        Word(s) => print_key(&s),
    }
}
