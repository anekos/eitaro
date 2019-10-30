use std::io::{BufRead, BufReader, BufWriter, stdout, Write, stdin};
use std::path::Path;

pub mod csv;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;



trait Exporter {
    fn export<T: Write>(&self, dictionary: &mut Dictionary, words: &[&str], out: &mut T) -> AppResultU;
}


pub fn export<T: AsRef<Path>>(dictionary_path: &T) -> AppResultU {
    let mut dictionary = Dictionary::new(dictionary_path);
    let exporter = csv::CsvExporter();

    let input = stdin();
    let input = input.lock();
    let reader = BufReader::new(input);
    let words = reader.lines().collect::<Result<Vec<String>, _>>()?;
    let words: Vec<&str> = words.iter().map(String::as_ref).map(str::trim).collect();

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    exporter.export(&mut dictionary, &words, &mut out)?;

    out.flush()?;

    Ok(())
}
