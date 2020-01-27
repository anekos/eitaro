
use std::collections::HashMap;
use std::fmt;
use std::io::{stdin, Read};
use std::path::Path;

use regex::Regex;
use separator::Separatable;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;



#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Level {
    Leveled(u8),
    OutOf,
    NotInDictionary,
}

struct LevelIter(Level);


const CHARS: &str = r"[a-zA-Z]+";


pub fn analyze<T: AsRef<Path>>(dictionary_path: &T) -> AppResultU {
    fn pct(v: usize, total: usize) -> f64 {
        v as f64 / total as f64 * 100.0
    }

    let mut words = HashMap::<&str, usize>::new();

    let mut unique_counts = HashMap::<Level, usize>::new();
    let mut unique_total = 0;
    let mut cumulative_counts = HashMap::<Level, usize>::new();
    let mut cumulative_total = 0;

    let mut dic = Dictionary::new(dictionary_path);
    let mut text = "".to_owned();

    stdin().read_to_string(&mut text)?;

    let chars = Regex::new(CHARS)?;
    for word in chars.find_iter(&text) {
        let word = word.as_str();
        if 2 <= word.len() {
            let count = words.entry(word).or_default();
            *count += 1;
        }
    }

    for (word, word_count) in words {
        let word = word.to_lowercase();
        let level = if let Some(level) = dic.get_level(&word)? {
            Level::Leveled(level)
        } else if dic.get(&word)?.is_some() {
            Level::OutOf
        } else {
            Level::NotInDictionary
        };

        let unique_count = unique_counts.entry(level).or_default();
        *unique_count += 1;
        unique_total += 1;

        let cumulative_count = cumulative_counts.entry(level).or_default();
        *cumulative_count += word_count;
        cumulative_total += word_count;
    }

    println!(
        "{:15} {:7}  {:>6}  {:>6}   {:7}  {:>6}  {:>6}",
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
            "{:15} {:>7}  {:>5.1}%  {:>5.1}%   {:>7}  {:>5.1}%  {:>5.1}%",
            level,
            unique_count.separated_string(),
            pct(*unique_count, unique_total),
            pct(unique_acc, unique_total),
            cumulative_count.separated_string(),
            pct(*cumulative_count, cumulative_total),
            pct(cumulative_acc, cumulative_total));
    }

    println!(
        "{:15} {:>7}                   {:>7}", "Total",
        unique_total.separated_string(),
        cumulative_total.separated_string());
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
