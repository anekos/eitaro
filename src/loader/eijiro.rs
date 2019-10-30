
use std::io::Read;

use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;
use if_let_return::if_let_some;

use crate::dictionary::DictionaryWriter;
use crate::errors::{AppError, AppResultU};
use crate::loader::Loader;
use crate::parser::eijiro::parse_line;
use crate::str_utils::{scan_words, WordType};



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU {
        println!("Reading...");
        let mut buffer = vec![];
        let _ = source.read_to_end(&mut buffer)?;

        println!("Encoding...");
        let source = WINDOWS_31J.decode(&buffer, Replace).map_err(|_| AppError::Encoding("MS932"))?;

        for line in source.lines() {
            if line.starts_with("■") {
                load_line(writer, &line[3..])?;
            }
        }


        Ok(())
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) -> AppResultU {
    fn extract_aliases(writer: &mut DictionaryWriter, key: &str, right: &str) -> AppResultU {
        fn extract(writer: &mut DictionaryWriter, key: &str, right: &str, word_type: WordType, pattern: &str) -> AppResultU {
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
        extract(writer, key, &right, WordType::English, "【同】")?;
        extract(writer, key, &right, WordType::Katakana, "【＠】")?;
        extract(writer, key, &right, WordType::English, "【略】")
    }

    fn extract_link(writer: &mut DictionaryWriter, key: &str, mut right: &str) -> AppResultU {
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

    fn extract_level(writer: &mut DictionaryWriter, key: &str, mut right: &str) -> AppResultU {
        if let Some(l) = right.find("【レベル】") {
            right = &right[l + 15..];
            let mut n = "".to_owned();
            for c in right.chars() {
                if c.is_digit(10) {
                    n.push(c);
                } else {
                    break;
                }
            }
            let level: u8 = n.parse()?;
            writer.levelize(level, key)?;
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
        extract_level(writer, left, &right)?;

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
    extract_level(writer, left, &right)?;

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
    const SYMBOLS: &str = "【{◆■〔";

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
    // for 【変化】《複》affairs、【分節】
    assert_eq!(read_until_symbols("《複》affairs【"), "《複》affairs");
    //  for 【同】〈米〉cookie
    assert_eq!(read_until_symbols("〈米〉cookie【"), "〈米〉cookie");
}
