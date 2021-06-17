
use std::collections::HashSet;
use std::path::Path;

use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppResultU;


#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Only words
    #[structopt(short = "w", long)]
    only_words: bool,
}


pub fn lemmas<T: AsRef<Path>>(opt: Opt, dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);
    let keys = dic.keys()?;
    let keys: HashSet<&String> = keys.iter().collect();
    let keys: Vec<&String> = keys.into_iter().collect();
    let mut keys: Vec<&String> = keys.into_iter().filter(|it| !opt.only_words || it.find(' ') == None).collect();
    keys.sort();
    for key in keys {
        println!("{}", key);
    }
    Ok(())
}
