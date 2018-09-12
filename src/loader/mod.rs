
use std::error::Error;
use std::path::Path;

use store::Dictionary;

pub mod eijiro;



pub trait Loader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, Box<Error>>;
}
