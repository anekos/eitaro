
use std::path::Path;

use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;



#[derive(Debug, StructOpt)]
pub struct Opt {
    word: String,
}


pub fn level<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    if let Some(found) = dic.get_level(&opt.word)? {
        println!("{}", found);
    } else {
        eprintln!("Not available");
    }
    Ok(())
}
