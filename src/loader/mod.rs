
use std::io::Read;
use std::path::Path;

use dictionary::{Dictionary, Stat};
use errors::AppError;

pub mod eijiro;
pub mod ejdic;



pub trait Loader {
    fn load<S: Read, D: AsRef<Path>>(&self, source: &mut S, dictionary_path: &D) -> Result<(Dictionary, Stat), AppError>;
}
