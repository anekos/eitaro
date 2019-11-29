
use std::io::Read;

use crate::dictionary::DictionaryWriter;
use crate::errors::{AppError, AppResultU};
use crate::loader::Loader;
use crate::parser::eijiro::parse_line;
use crate::types::DictionaryFormat::Csv;



#[derive(Default)]
pub struct CsvLoader();


impl Loader for CsvLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU {
        println!("Reading CSV...");

        let mut source = csv::Reader::from_reader(source);
        for columns in source.records() {
            let columns = columns?;
            if 2 < columns.len() {
                return Err(AppError::DictionaryFormat(Csv, "Too many columns"))
            }
            if columns.len() < 2 {
                return Err(AppError::DictionaryFormat(Csv, "Too few columns"))
            }

            writer.insert(&columns[0], parse_line(&columns[1])?)?;
        }

        Ok(())
    }
}
