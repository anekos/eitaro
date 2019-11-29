
use std::path::Path;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;




pub fn untypo<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    for candidate in dic.correct(word) {
        println!("{}", candidate);
    }
    Ok(())
}
