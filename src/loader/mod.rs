
use std::io::Read;

use crate::dictionary::{DictionaryWriter};
use crate::errors::AppResultU;

pub mod eijiro;
pub mod ejdic;
pub mod gene;



pub trait Loader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU;
}
