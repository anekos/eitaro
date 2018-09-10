
use std::error::Error;

use dictionary::Dictionary;

pub mod eijiro;



pub trait Loader {
    fn load(&self, source: &str) -> Result<Dictionary, Box<Error>>;
}
