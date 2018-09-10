
use ::dictionary::Dictionary;

pub mod eijiro;


pub trait Loader {
    fn load(&self, source: &str) -> Dictionary;
}
