
use std::path::Path;

use dictionary::Dictionary;
use errors::AppError;

pub mod eijiro;



pub trait Loader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, AppError>;
}
