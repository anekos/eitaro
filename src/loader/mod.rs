
use std::io::Read;

use dictionary::{DictionaryWriter};
use errors::AppError;

pub mod eijiro;
pub mod ejdic;
pub mod gene;



pub trait Loader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> Result<(), AppError>;
}
