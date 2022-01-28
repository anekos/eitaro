
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use regex::Regex;
use separator::Separatable;
use structopt::StructOpt;
use shellexpand;

use crate::dictionary::Dictionary;
use crate::errors::{AppError, AppResult};
use crate::loader::{csv, eijiro, ejdic, gene, json_simple_key_value, Loader};
use crate::types::DictionaryFormat;


#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Dictionary files
    files: Vec<PathBuf>,
}


pub fn build_dictionary<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> Result<(), AppError> {
    use DictionaryFormat::*;

    let mut dictionary = Dictionary::new(dictionary_path);
    let named_pattern = Regex::new(r"^(\w+)@(.+)$")?;

    let stat = dictionary.write(move |writer| {
        for file in &opt.files {
            let (source, file) = extract_source_and_path(&file, &named_pattern)?;
            println!("[{} ({})]", file, source.unwrap_or("-"));
            let format = guess(&file.as_ref())?;
            let mut file = File::open(file.as_ref())?;
            let mut writer = writer.clone().with_source(source);
            match format {
                Csv => csv::CsvLoader::default().load(&mut file, &mut writer)?,
                Eijiro => eijiro::EijiroLoader::default().load(&mut file, &mut writer)?,
                Ejdic => ejdic::EjdicLoader::default().load(&mut file, &mut writer)?,
                Gene => gene::GeneLoader::default().load(&mut file, &mut writer)?,
                JsonSimpleKeyValue => json_simple_key_value::JsonSimpleKeyValueLoader::default().load(&mut file, &mut writer)?,
            };
        }
        println!("[Finalize]");
        Ok(())
    })?;

    println!("Finished: {} words, {} aliases", stat.words.separated_string(), stat.aliases.separated_string());

    Ok(())
}

fn extract_source_and_path<'a, T: AsRef<Path>>(file: &'a T, pattern: &Regex) -> AppResult<(Option<&'a str>, Cow<'a, str>)> {
    let file = file.as_ref().to_str().ok_or(AppError::Unexpect("Invalid string"))?;
    if let Some(caps) = pattern.captures(file) {
        if caps.len() == 3 {
            Ok((
                Some(caps.get(1).unwrap().as_str()),
                shellexpand::tilde(caps.get(2).unwrap().as_str())
            ))
        } else {
            Ok((None, Cow::Borrowed(file)))
        }
    } else {
        Ok((None, Cow::Borrowed(file)))
    }
}

fn guess<T: AsRef<Path>>(source_path: &T) -> Result<DictionaryFormat, AppError> {
    let mut file = File::open(source_path)?;
    let mut head = [0u8;100];
    let size = file.read(&mut head)?;
    let head = &head[0..size];

    if head.starts_with(b" / This book describes Jpan and its kaisha at the cutting edge.") {
        return Ok(DictionaryFormat::Gene)
    }

    // 81a1 == ■
    if head.starts_with(b"\x81\xa1") {
        return Ok(DictionaryFormat::Eijiro);
    }

    if head.starts_with(b"{\"") {
        return Ok(DictionaryFormat::JsonSimpleKeyValue)
    }

    if head.contains(&b'\t') {
        return Ok(DictionaryFormat::Ejdic);
    }

    if head.contains(&b',') {
        return Ok(DictionaryFormat::Csv)
    }

    Err(AppError::Eitaro("Unknown format"))
}
