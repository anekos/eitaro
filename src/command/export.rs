use std::collections::HashSet;
use std::io::{BufRead, BufReader, BufWriter, Read, stdin, stdout, Write};
use std::path::Path;

use regex::Regex;
use structopt::StructOpt;

pub mod csv;

use crate::dictionary::Dictionary;
use crate::errors::{AppResult, AppResultU};
use crate::str_utils;



#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short = "t", long = "as-text")]
    as_text: bool,
}

trait Exporter {
    fn export<T: Write>(&self, dictionary: &mut Dictionary, words: &[&str], out: &mut T) -> AppResultU;
}


pub fn export<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> AppResultU {
    let mut dictionary = Dictionary::new(dictionary_path);
    let exporter = csv::CsvExporter();

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    let input = stdin();
    let input = input.lock();
    let mut reader = BufReader::new(input);
    if opt.as_text {
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
    let valid = Regex::new(r"\A[a-zA-Z]{2,}\z").unwrap();

    let mut words = HashSet::new();

    let chars = str_utils::simple_words_pattern();
    for word in chars.find_iter(s) {
        words.insert(word.as_str());
    }

    let mut result = Vec::<String>::new();

    for word in words {
        if let Some(found) = dictionary.lemmatize(word)? {
            if 2 < found.len() && valid.is_match(&found) {
                result.push(found);
            }
        }
    }

    result.sort();

    Ok(result)
}
