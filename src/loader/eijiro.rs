
use std::path::Path;

use errors::AppError;
use loader::Loader;
use store::{Dictionary, DictionaryWriter};
use str_utils::scan_words;



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, AppError> {
        let mut result = Dictionary::new(dictionary_path);

        result.writes(move |writer| {
            for line in source.lines() {
                if line.starts_with("■") {
                    load_line(writer, &line[3..])?;
                }
            }
            Ok(())
        })?;

        Ok(result)
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) -> Result<(), AppError> {
    fn extract_aliases(writer: &mut DictionaryWriter, key: &str, mut right: &str) -> Result<(), AppError> {
        if let Some(changes) = right.find("【変化】") {
            right = &right[changes..];
            if let Some(paren) = right.find('【') {
                right = &right[0..paren];
            }
            for it in scan_words(right) {
                writer.alias(it, key)?;
            }
        }
        Ok(())
    }

    fn extract_link(writer: &mut DictionaryWriter, key: &str, right: &str) -> Result<(), AppError> {
        if let (Some(0), Some(r)) = (right.find("＝<→"), right.find('>')) {
            writer.alias(key, &right[7..r])?;
        }
        return Ok(());
    }

    if_let_some!(sep = line.find(" : "), Ok(()));
    let (left, right) = line.split_at(sep);
    let right = &right[3..];

    if let (Some(l), Some(r)) = (left.find('{'), left.rfind('}')) {
        let (left, tag) = left.split_at(l);
        let left = left.trim();
        let mut tag = &tag[1..(r - l)];

        if let Some(hyphen) = tag.find('-') {
            tag = &tag[0.. hyphen];
        }

        let right = if tag.chars().next().map(|it| it.is_digit(10)) == Some(false) {
            format!("{{{}}} {}", tag, right)
        } else {
            format!("{}", right)
        };

        extract_link(writer, left, &right)?;
        extract_aliases(writer, left, &right)?;
        writer.insert(left, &right)?;
        return Ok(());
    }

    extract_link(writer, left, &right)?;
    extract_aliases(writer, left, right)?;
    writer.insert(left, right)?;

    Ok(())
}
