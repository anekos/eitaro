
// ref: http://norvig.com/spell-correct.html

use std::path::Path;
use std::io::{BufReader, Read};
use std::fs::File;
use std::collections::HashSet;

use bincode::deserialize;
use strsim::jaro_winkler;

use crate::errors::AppResult;



const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";


pub struct Corrector {
    keys: HashSet<String>,
}


impl Corrector {
    pub fn load<T: AsRef<Path>>(path: &T) -> AppResult<Self> {
        let file = File::open(&path)?;
        let mut file = BufReader::new(file);
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        let keys = deserialize(&buffer)?;
        Ok(Self { keys })
    }

    pub fn correct(&self, word: &str) -> Vec<String> {
        let word = word.to_lowercase();
        let candidates = build_complex_candidates(&word);

        let mut min_d = std::f64::MAX;

        let mut result = vec![];
        for candidate in candidates {
            if self.keys.contains(&candidate) {
                let d = jaro_winkler(&word, &candidate);
                // println!("{:?}→{:?}", d, candidate);
                result.push((d, candidate));
                if d < min_d {
                    min_d = d;
                }
            }
        }

        result.sort_by(|b, a| a.0.partial_cmp(&b.0).unwrap());

        result.into_iter().take(10).map(|(_, word)| word).collect()
    }
}


fn build_complex_candidates(word: &str) -> HashSet<String> {
    let mut result = HashSet::new();
    for candidate in build_simple_candidates(word) {
        let simple = build_simple_candidates(&candidate);
        result.insert(candidate);
        result.extend(simple);
    }
    result
}

fn build_simple_candidates(word: &str) -> HashSet<String> {
    let mut set = HashSet::new();

    let splits = (0..word.len()).map(|it| word.split_at(it)).collect::<Vec<_>>();

    for (l, r) in &splits {
        // deletion
        if !r.is_empty() {
            set.insert(format!("{}{}", l, &r[1..]));
            // replacing
            for c in LETTERS.chars() {
                set.insert(format!("{}{}{}", l, c, &r[1..]));
            }
            // transposition
            if 1 < r.len() {
                set.insert(format!("{}{}{}{}", l, &r[1..2], &r[0..1], &r[2..]));
            }
        }
        // insertion
        for c in LETTERS.chars() {
            set.insert(format!("{}{}{}", l, c, &r[..]));
        }
    }

    set
}
