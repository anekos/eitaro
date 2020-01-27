use std::collections::HashSet;
use std::io::{BufRead, BufReader, BufWriter, Read, stdin, stdout, Write};
use std::path::Path;

use regex::Regex;

pub mod csv;

use crate::dictionary::Dictionary;
use crate::errors::{AppResult, AppResultU};



trait Exporter {
    fn export<T: Write>(&self, dictionary: &mut Dictionary, words: &[&str], out: &mut T) -> AppResultU;
}


pub fn export<T: AsRef<Path>>(dictionary_path: &T, as_text: bool) -> AppResultU {
    let mut dictionary = Dictionary::new(dictionary_path);
    let exporter = csv::CsvExporter();

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    let input = stdin();
    let input = input.lock();
    let mut reader = BufReader::new(input);
    if as_text {
        let mut buffer = "".to_owned();
        reader.read_to_string(&mut buffer)?;
        let words = extract_text(&mut dictionary, &buffer)?;
        let words = words.iter().map(String::as_ref).collect::<Vec<&str>>();
        exporter.export(&mut dictionary, &words, &mut out)?;
    } else {
        let words = reader.lines().collect::<Result<Vec<String>, _>>()?;
        let words: Vec<&str> = words.iter().map(String::as_ref).map(str::trim).collect();
        exporter.export(&mut dictionary, &words, &mut out)?;
    }

    out.flush()?;

    Ok(())
}

fn extract_text(dictionary: &mut Dictionary, s: &str) -> AppResult<Vec<String>> {
    let splitter = Regex::new(r"[^a-zA-Z]+").unwrap();
    let words = splitter.split(s).filter(|it| 2 < it.len()).collect::<HashSet<&str>>();

    let valid = Regex::new(r"\A[a-zA-Z]{2,}+\z").unwrap();

    let mut result = HashSet::new();
    for word in words {
        if let Some(found) = dictionary.lemmatize(word)? {
            if valid.is_match(&found) {
                result.insert(found);
            }
        }
    }

    let mut result = result.into_iter().filter(|it| 2 < it.len()).collect::<Vec<String>>();
    result.sort();

    Ok(result)
}
