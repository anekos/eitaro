
use std::path::Path;

use crate::errors::AppResultU;
use crate::path::get_history_path;


pub fn path<T: AsRef<Path>>(dictionary_path: &T) -> AppResultU {
    let history = get_history_path()?;
    println!("dictionary: {}", dictionary_path.as_ref().to_str().unwrap());
    println!("history: {}", history.to_str().unwrap());
    Ok(())
}
