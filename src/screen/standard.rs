
use std::io::{stdout, Write as _};
use std::fmt::{Error as FmtError, Write};
use std::sync::mpsc::Receiver;

use deco::{dwrite, dwriteln};

use crate::dictionary::{Entry, Text};
use crate::errors::AppError;



pub fn main(rx: Receiver<Option<Vec<Entry>>>) -> Result<(), AppError> {
    for entries in rx {
        print_opt(entries)?
    }
    Ok(())
}

pub fn print_opt(entries: Option<Vec<Entry>>) -> Result<(), AppError> {
    fn color_key(out: &mut String, key: &str) -> Result<(), FmtError> {
        dwriteln!(out, [black on_yellow bold "{}" !] key)
    }

    fn color(out: &mut String, text: &Text) -> Result<(), FmtError> {
        use self::Text::*;

        match text {
            Annot(s) => dwrite!(out, [yellow "{}" !] s),
            Class(s) => dwrite!(out, [blue "{}" !] s),
            Countability(c) => dwrite!(out, [yellow bold "{}" !] c),
            Definition(s) => dwrite!(out, [white bold "{}" !] s),
            Example(s) => dwrite!(out, [green "{}"] s),
            Information(s) => dwrite!(out, [cyan "{}"] s),
            Note(s) => write!(out, "{}", s),
            Tag(s) => dwrite!(out, [red bold "{}"] s),
            Word(s) => color_key(out, &s),
        }
    }

    let out = stdout();
    let mut out = out.lock();

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
            write!(out, "{}", buffer)?;
        }
    } else {
        dwriteln!(out, [black on_red "{}" !] "Not Found")?;
    }

    Ok(())
}
