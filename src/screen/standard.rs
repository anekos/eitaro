
use std::fmt::{Error as FmtError, Write};
use std::sync::mpsc::Receiver;

use dictionary::{Entry, Text};
use errors::AppError;



pub fn main(rx: Receiver<Option<Vec<Entry>>>) {
    for entries in rx {
        print_opt(entries).unwrap();
    }
}

pub fn print_opt(entries: Option<Vec<Entry>>) -> Result<(), AppError> {
    use colored::*;

    fn color_key(out: &mut String, key: &str) -> Result<(), FmtError> {
        writeln!(out, "{}", key.black().on_yellow().bold())
    }

    fn color(out: &mut String, text: &Text) -> Result<(), FmtError> {
        use self::Text::*;

        match text {
            Annot(s) => write!(out, "{}", s.yellow()),
            Class(s) => write!(out, "{}", s.blue()),
            Countability(c) => write!(out, "{}", c.to_string().yellow().bold()),
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
            for definition in &entry.definitions {
                for (index, text) in definition.content.iter().enumerate() {
                    if 0 < index {
                        buffer.push(' ');
                    }
                    color(&mut buffer, text)?;
                }
                buffer.push('\n');
            }
            print!("{}", buffer);
        }
    } else {
        println!("{}", "Not Found".black().on_red());
    }

    Ok(())
}
