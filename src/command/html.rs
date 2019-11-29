
use std::io::{BufWriter, Error as IOError, stdout, Write};
use std::path::Path;

use askama_escape::{escape, Html};

use crate::dictionary::{Dictionary, Entry, Text};
use crate::errors::AppError;



pub fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<(), AppError> {
    let mut dic = Dictionary::new(dictionary_path);
    let found = dic.get_smart(word.trim())?.ok_or(AppError::NotFound)?;
    print(found)?;
    Ok(())
}

fn print(entries: Vec<Entry>) -> Result<(), AppError> {
    fn span<W: Write>(out: &mut W, name: &'static str, text: &str) -> Result<(), IOError> {
        write!(out, "<span class=\"eitaro-definition eitaro-def-{}\">{}</span>", name, escape(text, Html))
    }

    fn color_key<W: Write>(out: &mut W, key: &str) -> Result<(), IOError> {
        span(out, "key", key)
    }

    fn color<W: Write>(out: &mut W, text: &Text) -> Result<(), IOError> {
        use self::Text::*;

        match text {
            Annot(s) => span(out, "annotation", s),
            Class(s) => span(out, "class", s),
            Countability(c) => span(out, "countability", &format!("{}", c)),
            Definition(s) => span(out, "definition", s),
            Etymology(s) => span(out, "etymology", s),
            Example(s) => span(out, "example", s),
            Information(s) => span(out, "information", s),
            Note(s) => span(out, "note", s),
            Tag(s) => span(out, "tag", s),
            Word(s) => color_key(out, &s),
        }
    }

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    for entry in entries {
        writeln!(out, "<h1 class=\"eitaro-term\">{}</h1>", escape(&entry.key, Html))?;

        writeln!(out, "<ol>")?;

        for definition in &entry.definitions {
            write!(out, "  <li>")?;
            for (index, text) in definition.content.iter().enumerate() {
                if 0 < index {
                    write!(out, " ")?;
                }
                color(&mut out, text)?;
            }
            writeln!(out, "  </li>")?;
        }

        writeln!(out, "</ol>")?;
    }

    Ok(())
}
