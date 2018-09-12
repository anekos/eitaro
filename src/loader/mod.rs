
use std::path::Path;

use errors::AppError;
use store::Dictionary;

pub mod eijiro;



pub trait Loader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, AppError>;
}
