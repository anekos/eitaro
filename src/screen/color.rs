
use std::io::{Error as IOError, Write};
use std::sync::mpsc::Receiver;

use deco::{dprintln, dwrite, dwriteln};

use crate::dictionary::{Entry, Text};
use crate::errors::AppResultU;
use crate::pager::with_pager;



pub fn main(rx: Receiver<Option<Vec<Entry>>>) -> AppResultU {
    for entries in rx {
        if let Some(entries) = entries {
            print(entries)?
        } else {
            print_not_found();
        }
    }
    Ok(())
}

pub fn print(entries: Vec<Entry>) -> AppResultU {
    fn color_key<W: Write>(out: &mut W, key: &str) -> Result<(), IOError> {
        dwriteln!(out, [black on_yellow bold "{}" !] key)
    }

    fn color<W: Write>(out: &mut W, text: &Text) -> Result<(), IOError> {
        use self::Text::*;

        match text {
            Annot(s) => dwrite!(out, [yellow "{}" !] s),
            Class(s) => dwrite!(out, [blue "{}" !] s),
            Countability(c) => dwrite!(out, [yellow bold "{}" !] c),
            Definition(s) => dwrite!(out, [white bold "{}" !] s),
            Error(s) => dwrite!(out, [red bold "{}" !] s),
            Etymology(s) => dwrite!(out, [magenta bold "語源" ! " {}"] s),
            Example(s) => dwrite!(out, [green "{}" !] s),
            Information(s) => dwrite!(out, [cyan "{}" !] s),
            Note(s) => write!(out, "{}", s),
            Tag(s) => dwrite!(out, [red bold "{}" !] s),
            Word(s) => color_key(out, &s),
        }
    }

    with_pager(|out| {
        for entry in entries {
            color_key(out, &entry.key)?;
            for definition in &entry.definitions {
                for (index, text) in definition.content.iter().enumerate() {
                    if 0 < index {
                        write!(out, " ")?;
                    }
                    color(out, text)?;
                }
                writeln!(out)?;
            }
        }
        Ok(())
    })
}

pub fn print_not_found() {
    dprintln!([black on_red "{}" !] "Not Found");
}
