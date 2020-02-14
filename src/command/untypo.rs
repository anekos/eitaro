
use std::path::Path;

use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;




#[derive(Debug, StructOpt)]
pub struct Opt {
    word: String,
}


pub fn untypo<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    for candidate in dic.correct(&opt.word) {
        println!("{}", candidate);
    }
    Ok(())
}
