
use std::io::{BufWriter, Error as IOError, stdout, Write};
use std::sync::mpsc::Receiver;

use deco::{dprintln, dwrite, dwriteln};

use crate::dictionary::{Entry, Text};
use crate::errors::AppResultU;



pub fn main(rx: Receiver<Option<Vec<Entry>>>) -> AppResultU {
    for entries in rx {
        print_opt(entries)?
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
            Example(s) => dwrite!(out, [green "{}" !] s),
            Information(s) => dwrite!(out, [cyan "{}" !] s),
            Note(s) => write!(out, "{}", s),
            Tag(s) => dwrite!(out, [red bold "{}" !] s),
            Word(s) => color_key(out, &s),
        }
    }

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    for entry in entries {
        color_key(&mut out, &entry.key)?;
        for definition in &entry.definitions {
            for (index, text) in definition.content.iter().enumerate() {
                if 0 < index {
                    write!(out, " ")?;
                }
                color(&mut out, text)?;
            }
            writeln!(out)?;
        }
    }

    Ok(())
}

pub fn print_opt(entries: Option<Vec<Entry>>) -> AppResultU {
    if let Some(entries) = entries {
        print(entries)?
    } else {
        print_not_found()
    }
    Ok(())
}

fn print_not_found() {
    dprintln!([black on_red "{}" !] "Not Found");
}
