
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
        let tag = extract_tag_name(&tag[1..(r - l)]);

        extract_link(writer, left, &right)?;
        extract_aliases(writer, left, &right)?;

        let right = if let Some(tag) = tag {
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


fn extract_tag_name(s: &str) -> Option<&str> {
    let mut left = 0;
    let mut in_tag = false;
    let mut index = 0;

    for c in s.chars() {
        if in_tag {
            if !c.is_alphabetic() {
                return Some(&s[left..index]);
            }
        } else {
            if c.is_alphabetic() {
                left = index;
                in_tag = true;
            }
        }

        index += c.len_utf8();
    }

    if in_tag {
        Some(&s[left..index])
    } else {
        None
    }
}

#[cfg(test)]#[test]
fn test_extract_tag_name() {
    assert_eq!(extract_tag_name("   自動  "), Some("自動"));
    assert_eq!(extract_tag_name("-自動  "), Some("自動"));
    assert_eq!(extract_tag_name("1-自動  "), Some("自動"));
    assert_eq!(extract_tag_name(" 1 "), None);
}
