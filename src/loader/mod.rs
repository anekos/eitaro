
use std::io::Read;

use crate::dictionary::{DictionaryWriter};
use crate::errors::AppError;

pub mod eijiro;
pub mod ejdic;
pub mod gene;



pub trait Loader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> Result<(), AppError>;
}
