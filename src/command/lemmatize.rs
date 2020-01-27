
use std::path::Path;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;



pub fn lemmatize<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    if let Some(found) = dic.lemmatize(word)? {
        println!("{}", found);
    }
    Ok(())
}
