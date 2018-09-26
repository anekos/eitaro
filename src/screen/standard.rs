
use std::fmt::{Error as FmtError, Write};
use std::sync::mpsc::Receiver;

use dictionary::Entry;
use errors::AppError;
use screen::parser::{parse, Text};



pub fn main(rx: Receiver<Option<Vec<Entry>>>) {
    for entries in rx {
        print_opt(entries).unwrap();
    }
}

pub fn print_opt(entries: Option<Vec<Entry>>) -> Result<(), AppError> {
    use colored::*;

    fn color_key(out: &mut String, key: &str) -> Result<(), FmtError> {
        write!(out, "{}\n", key.black().on_yellow().bold())
    }

    fn color(out: &mut String, text: &Text) -> Result<(), FmtError> {
        use self::Text::*;

        match text {
            Annot(s) => write!(out, "{} ", s.yellow()),
            Class(s) => write!(out, "{} ", s.blue()),
            Example(s) => write!(out, " {} ", s.green()),
            LineBreak => writeln!(out),
            Note(s) => write!(out, " {}", s.cyan()),
            Plain(s) => write!(out, "{}", s.white().bold()),
            Tag(s) => write!(out, "{}", s.red().bold()),
            Word(s) => color_key(out, &s),
        }
    }


    if let Some(entries) = entries {
        for entry in entries {
            let mut buffer = "".to_owned();
            let texts = parse(&entry.content)?;
            color_key(&mut buffer, &entry.key)?;
            for text in &texts {
                color(&mut buffer, text)?;
            }
            print!("{}", buffer);
        }
    } else {
        println!("{}", "Not Found".black().on_red());
    }

    Ok(())
}