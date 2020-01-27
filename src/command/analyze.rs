
use std::collections::{HashMap, HashSet};
use std::io::{stdin, Read};
use std::path::Path;

use regex::Regex;
use separator::Separatable;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;



#[derive(Eq, Hash, PartialEq)]
enum Level {
    Leveled(u8),
    OutOf,
    NotInDictionary,
}

const CHARS: &str = r"[a-zA-Z]+";


pub fn analyze<T: AsRef<Path>>(dictionary_path: &T) -> AppResultU {
    fn pct(v: usize, total: usize) -> f64 {
        v as f64 / total as f64 * 100.0
    }

    let mut counts = HashMap::<Level, usize>::new();
    let mut words = HashSet::<&str>::new();
    let mut total = 0;

    let mut dic = Dictionary::new(dictionary_path);
    let mut text = "".to_owned();

    stdin().read_to_string(&mut text)?;

    let chars = Regex::new(CHARS)?;
    for word in chars.find_iter(&text) {
        let word = word.as_str();
        if 2 <= word.len() {
            words.insert(word);
        }
    }

    for word in words {
        let word = word.to_lowercase();
        let count = if let Some(level) = dic.get_level(&word)? {
            counts.entry(Level::Leveled(level)).or_default()
        } else if dic.get(&word)?.is_some() {
            counts.entry(Level::OutOf).or_default()
        } else {
            counts.entry(Level::NotInDictionary).or_default()
        };
        *count += 1;
        total += 1;
    }

    println!("Level               Count       %    Accu");

    let mut acc = 0;
    for level in 1 ..= 12 {
        let count = counts.entry(Level::Leveled(level)).or_default();
        acc += *count;
        println!(
            "Level {:>2}           {:>6}  {:>5.1}%  {:>5.1}%",
            level,
            count.separated_string(),
            pct(*count, total),
            pct(acc, total));
    }

    let ool = counts.entry(Level::OutOf).or_default().to_owned();
    let nid = counts.entry(Level::NotInDictionary).or_default().to_owned();

    acc += ool;
    println!("Out of level       {:>6}  {:>5.1}%  {:>5.1}%", ool.separated_string(), pct(ool, total), pct(acc, total));
    acc += nid;
    println!("Not in dictionary  {:>6}  {:>5.1}%  {:>5.1}%", nid.separated_string(), pct(nid, total), pct(acc, total));

    println!();

    println!("Total:             {:>6}", total.separated_string());
    Ok(())
}
