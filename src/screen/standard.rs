
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
            Annot(s) => write!(out, "{}", s.yellow()),
            Class(s) => write!(out, "{}", s.blue()),
            Definition(s) => write!(out, "{}", s.white().bold()),
            Example(s) => write!(out, "{}", s.green()),
            Information(s) => write!(out, "{}", s.cyan()),
            Note(s) => write!(out, "{}", s),
            Tag(s) => write!(out, "{}", s.red().bold()),
            Word(s) => color_key(out, &s),
        }
    }


    if let Some(entries) = entries {
        for entry in entries {
            let mut buffer = "".to_owned();
            color_key(&mut buffer, &entry.key)?;
            for (index, definition) in parse(&entry.content)?.iter().enumerate() {
                if 0 < index {
                    buffer.push('\n');
                }
                for (index, text) in definition.iter().enumerate() {
                    if 0 < index {
                        buffer.push(' ');
                    }
                    color(&mut buffer, text)?;
                }
            }
            print!("{}", buffer);
        }
    } else {
        println!("{}", "Not Found".black().on_red());
    }

    Ok(())
}
