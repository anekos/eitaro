
use std::path::Path;

use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;



#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Word
    word: String,
}


pub fn lemmatize<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    if let Some(found) = dic.lemmatize(&opt.word)? {
        println!("{}", found);
    }
    Ok(())
}
