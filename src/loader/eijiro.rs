
use std::path::Path;

use dictionary::{Dictionary, DictionaryWriter};
use errors::AppError;
use loader::Loader;
use str_utils::{scan_words, WordType};



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
    fn extract_aliases(writer: &mut DictionaryWriter, key: &str, right: &str) -> Result<(), AppError> {
        fn extract(writer: &mut DictionaryWriter, key: &str, mut right: &str, word_type: WordType, pattern: &str) -> Result<(), AppError> {
            const LEFT_PAREN: char = '【';
            if let Some(found) = right.find(pattern) {
                right = &right[found + pattern.len()..];
                if let Some(paren) = right.find(LEFT_PAREN) {
                    right = &right[0..paren];
                }
                for it in scan_words(word_type, right) {
                    writer.alias(it, key)?;
                }
            }
            Ok(())
        }

        let right = right.replace('（', "").replace('）', "");
        extract(writer, key, &right, WordType::English, "【変化】")?;
        extract(writer, key, &right, WordType::Katakana, "【＠】")
    }

    fn extract_link(writer: &mut DictionaryWriter, key: &str, mut right: &str) -> Result<(), AppError> {
        if right.starts_with('＝') {
            right = &right[3..];
        }
        if let (true, Some(r)) = (right.starts_with("<→"), right.find('>')) {
            writer.alias(&right[4..r], key)?;
            writer.alias(key, &right[4..r])?;
        }
        Ok(())
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

        extract_link(writer, left, &right)?;
        extract_aliases(writer, left, &right)?;

        let right = if tag.chars().next().map(|it| it.is_digit(10)) == Some(false) {
            format!("{{{}}} {}", tag, right)
        } else {
            right.to_string()
        };

        writer.insert(left, &right)?;
        return Ok(());
    }

    extract_link(writer, left, &right)?;
    extract_aliases(writer, left, right)?;
    writer.insert(left, right)?;

    Ok(())
}
