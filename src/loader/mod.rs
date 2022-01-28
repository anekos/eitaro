
use std::io::Read;

use crate::dictionary::{DictionaryWriter};
use crate::errors::AppResultU;

pub mod csv;
pub mod eijiro;
pub mod ejdic;
pub mod gene;
pub mod json_simple_key_value;



pub trait Loader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU;
}
