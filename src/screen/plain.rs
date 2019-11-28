
use std::io::{BufWriter, Error as IOError, stdout, Write};

use crate::dictionary::{Entry, Text};
use crate::errors::AppResultU;



pub fn print(entries: Vec<Entry>) -> AppResultU {
    fn color<W: Write>(out: &mut W, text: &Text) -> Result<(), IOError> {
        use self::Text::*;

        match text {
            Annot(s) | Class(s) | Definition(s) | Example(s) | Information(s) | Note(s) | Tag(s) | Word(s)=>
                write!(out, "{}", s),
            Countability(c) =>
                write!(out, "{}", c),
        }
    }

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    for entry in entries {
        writeln!(out, "*{}*", &entry.key)?;
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

pub fn print_not_found() {
    println!("Not Found");
}
