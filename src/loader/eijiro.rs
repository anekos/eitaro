
use std::io::Read;
use std::path::Path;

use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

use dictionary::{Dictionary, DictionaryWriter, Stat};
use errors::AppError;
use loader::Loader;
use parser::eijiro::parse_line;
use str_utils::{scan_words, WordType};



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load<S: Read, D: AsRef<Path>>(&self, source: &mut S, dictionary_path: &D) -> Result<(Dictionary, Stat), AppError> {
        let mut dictionary = Dictionary::new(dictionary_path);

        println!("Reading...");
        let mut buffer = vec![];
        let _ = source.read_to_end(&mut buffer)?;

        println!("Encoding...");
        let source = WINDOWS_31J.decode(&buffer, Replace).map_err(|err| err.to_string())?;

        let stat = dictionary.write(move |writer| {
            for line in source.lines() {
                if line.starts_with("■") {
                    load_line(writer, &line[3..])?;
                }
            }
            Ok(())
        })?;

        Ok((dictionary, stat))
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) -> Result<(), AppError> {
    fn extract_aliases(writer: &mut DictionaryWriter, key: &str, right: &str) -> Result<(), AppError> {
        fn extract(writer: &mut DictionaryWriter, key: &str, right: &str, word_type: WordType, pattern: &str) -> Result<(), AppError> {
            if let Some(found) = right.find(pattern) {
                let right = &right[found + pattern.len()..];
                let right = read_until_symbols(&right);
                if !right.is_empty() {
                    for it in scan_words(word_type, right) {
                        writer.alias(&it, key)?;
                    }
                }
            }
            Ok(())
        }

        let right = right.replace('（', "").replace('）', "");
        extract(writer, key, &right, WordType::English, "【変化】")?;
        extract(writer, key, &right, WordType::Katakana, "【＠】")?;
        extract(writer, key, &right, WordType::English, "【略】")
    }

    fn extract_link(writer: &mut DictionaryWriter, key: &str, mut right: &str) -> Result<(), AppError> {
        if let Some(l) = right.find("＝<→") {
            right = &right[l + 7..];
        } else if let Some(l) = right.find("<→") {
            right = &right[l + 4..];
        } else {
            return Ok(())
        }
        if let Some(r) = right.find('>') {
            writer.alias(&right[0..r], key)?;
            writer.alias(key, &right[0..r])?;
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

        writer.insert(left, parse_line(&right)?)?;
        return Ok(());
    }

    extract_link(writer, left, &right)?;
    extract_aliases(writer, left, right)?;
    writer.insert(left, parse_line(&right)?)?;

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
        } else if c.is_alphabetic() {
            left = index;
            in_tag = true;
        }

        index += c.len_utf8();
    }

    if in_tag {
        Some(&s[left..index])
    } else {
        None
    }
}

fn read_until_symbols(s: &str) -> &str {
    const SYMBOLS: &str = "【{〈《◆■〔";

    let mut right = 0;

    for c in s.chars() {
        if SYMBOLS.find(c).is_some() {
            break;
        }
        right += c.len_utf8();
    }

    &s[0..right]
}

#[cfg(test)]#[test]
fn test_extract_tag_name() {
    assert_eq!(extract_tag_name("   自動  "), Some("自動"));
    assert_eq!(extract_tag_name("-自動  "), Some("自動"));
    assert_eq!(extract_tag_name("1-自動  "), Some("自動"));
    assert_eq!(extract_tag_name(" 1 "), None);
}

#[cfg(test)]#[test]
fn test_read_until_symbols() {
    assert_eq!(read_until_symbols("cat【neko"), "cat");
    assert_eq!(read_until_symbols("cat◆neko"), "cat");
}
