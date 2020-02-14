
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
    println!("{}", dic.lemmatize(&opt.word)?);
    Ok(())
}
