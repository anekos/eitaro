
use std::collections::HashMap;
use std::fmt;
use std::io::{stdin, Read};
use std::path::Path;

use separator::Separatable;

use crate::dictionary::Dictionary;
use crate::errors::{AppResult, AppResultU};
use crate::str_utils;



#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Level {
    Leveled(u8),
    OutOf,
    NotInDictionary,
}

struct LevelIter(Level);

pub struct Target {
    pub not_in_dictionary: bool,
    pub svl: bool,
    pub usage: bool,
}

struct Common{
    words: Vec<Word>,
}

struct Word {
    word: String,
    count: usize,
    level: Level,
}


const INDENT: &str = "    ";


pub fn analyze<T: AsRef<Path>>(dictionary_path: &T, target: Target) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);

    let mut text = "".to_owned();
    stdin().read_to_string(&mut text)?;

    let common = analyze_common(&mut dic, &text)?;

    if target.svl {
        analyze_svl(&common)?;
    }
    if target.usage {
        analyze_usage(&common)?;
    }
    if target.not_in_dictionary {
        analyze_not_in_dictionary(&common)?;
    }

    Ok(())
}


fn analyze_common(dic: &mut Dictionary, text: &str) -> AppResult<Common> {
    let mut words = HashMap::<&str, usize>::new();

    let chars = str_utils::simple_words_pattern();
    for word in chars.find_iter(&text) {
        let word = word.as_str();
        if 2 <= word.len() {
            let count = words.entry(word).or_default();
            *count += 1;
        }
    }

    let mut result = Vec::<Word>::new();

    for (word, count) in words {
        let word = word.to_lowercase();
        let level = if let Some(level) = dic.get_level(&word)? {
            Level::Leveled(level)
        } else if dic.get(&word)?.is_some() {
            Level::OutOf
        } else {
            Level::NotInDictionary
        };
        result.push(Word {
            count,
            level,
            word,
        });
    }

    Ok(Common { words: result })
}

fn analyze_svl(common: &Common) -> AppResultU {
    fn pct(v: usize, total: usize) -> f64 {
        v as f64 / total as f64 * 100.0
    }

    let mut unique_counts = HashMap::<Level, usize>::new();
    let mut unique_total = 0;
    let mut cumulative_counts = HashMap::<Level, usize>::new();
    let mut cumulative_total = 0;

    for word in &common.words {
        let unique_count = unique_counts.entry(word.level).or_default();
        *unique_count += 1;
        unique_total += 1;

        let cumulative_count = cumulative_counts.entry(word.level).or_default();
        *cumulative_count += word.count;
        cumulative_total += word.count;
    }

    println!("Word level:");
    println!(
        "{}{:15} {:7}  {:>6}  {:>6}   {:7}  {:>6}  {:>6}",
        INDENT,
        "Level",
        "Unique",
        "%",
        "Σ",
        "Cumulu",
        "%",
        "Σ");

    let mut unique_acc = 0;
    let mut cumulative_acc = 0;

    for level in LevelIter::new() {
        let unique_count = unique_counts.entry(level).or_default();
        unique_acc += *unique_count;
        let cumulative_count = cumulative_counts.entry(level).or_default();
        cumulative_acc += *cumulative_count;
        println!(
            "{}{:15} {:>7}  {:>5.1}%  {:>5.1}%   {:>7}  {:>5.1}%  {:>5.1}%",
            INDENT,
            level,
            unique_count.separated_string(),
            pct(*unique_count, unique_total),
            pct(unique_acc, unique_total),
            cumulative_count.separated_string(),
            pct(*cumulative_count, cumulative_total),
            pct(cumulative_acc, cumulative_total));
    }

    println!(
        "{}{:15} {:>7}                   {:>7}",
        INDENT,
        "Total",
        unique_total.separated_string(),
        cumulative_total.separated_string());
    println!();

    Ok(())
}

fn analyze_not_in_dictionary(common: &Common) -> AppResultU {
    println!("Words not in dictionary:");
    for word in &common.words {
        if word.level == Level::NotInDictionary {
            println!("{}{}", INDENT, word.word);
        }
    }
    println!();
    Ok(())
}

fn analyze_usage(common: &Common) -> AppResultU {
    println!("Usage ranking:");
    let mut words: Vec<(&str, usize)> = common.words.iter().map(|it| (it.word.as_ref(), it.count)).collect();
    words.sort_by(|(_, a), (_, b)| b.cmp(a));
    for (word, count) in words.iter().take(10) {
        println!("{}{:10} {:>7}", INDENT, word, count.separated_string());
    }
    println!();
    Ok(())
}


impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Level::*;

        match self {
            Leveled(level) => f.pad(&format!("{:>02}", level)),
            OutOf => f.pad("Out of level"),
            NotInDictionary => f.pad("Not in dict"),
        }
    }
}

impl LevelIter {
    fn new() -> Self {
        LevelIter(Level::Leveled(0))
    }
}

impl Iterator for LevelIter {
    type Item = Level;

    fn next(&mut self) -> Option<Self::Item> {
        use Level::*;

        let result = match self.0 {
            Leveled(level) if level < 12 =>
                Leveled(level + 1),
            Leveled(_) =>
                OutOf,
            OutOf =>
                NotInDictionary,
            NotInDictionary =>
                return None,
        };

        self.0 = result;

        Some(result)
    }
}
