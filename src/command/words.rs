
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::path::Path;

use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;
use crate::str_utils;


#[derive(Debug, Default, StructOpt)]
pub struct Opt {
}



pub fn extract<T: AsRef<Path>>(_opt: Opt, dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);

    let mut text = "".to_owned();
    stdin().read_to_string(&mut text)?;

    let mut lt = HashMap::new();

    let chars = str_utils::simple_words_pattern();
    for word in chars.find_iter(&text) {
        let word = word.as_str();
        if let Ok(lem) = lt.entry(word).or_insert_with(|| dic.lemmatize(&word.to_lowercase())) {
            println!("{}", lem);
        }
    }

    Ok(())
}
