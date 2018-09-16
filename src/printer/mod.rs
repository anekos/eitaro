
use colored::*;

pub mod parser;

use errors::AppError;
use self::parser::{Text, parse};



pub fn print_colored_opt(s: Option<&str>) -> Result<(), AppError> {
    if let Some(s) = s {
        print_colored(s)?;
    } else {
        println!("{}", "Not found".white().on_red().bold());
    }
    Ok(())
}

fn print_colored(s: &str) -> Result<(), AppError> {
    for line in s.lines() {
        let text = parse(line)?;
        for it in &text {
            print_text(it);
        }
        println!("");
    }
    Ok(())
}

fn print_text(text: &Text) {
    use self::Text::*;

    match text {
        Annot(s) => print!("{} ", s.yellow()),
        Class(s) => print!("{} ", s.blue()),
        Example(s) => print!(" {} ", s.green()),
        Note(s) => print!(" {}", s.cyan()),
        Plain(s) => print!("{}", s.white().bold()),
        Tag(s) => print!("{}", s.red().bold()),
        Word(s) => print!("{} ", s.black().on_yellow().bold()),
    }
}
