
#[macro_use] extern crate if_let_return;
extern crate encoding;
extern crate rusty_leveldb;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process::exit;

use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

mod loader;
mod dictionary;

use loader::Loader;



fn build_dictionary(source_path: &str) -> Result<(), Box<Error>> {
    eprintln!("Building...");
    let mut buffer = vec![];
    let mut file = File::open(source_path)?;
    eprintln!("Reading...");
    let _ = file.read_to_end(&mut buffer)?;
    eprintln!("Encoding...");
    let decoded = WINDOWS_31J.decode(&buffer, Replace)?;
    eprintln!("Loading...");
    let ldr = loader::eijiro::EijiroLoader::default();
    let _ = ldr.load(&decoded);
    Ok(())
}


fn load_dictionary(_dictionary_path: &str) -> Result<(), Box<Error>> {
    Ok(())
}


fn _main() -> Result<(), Box<Error>> {
    let mut args = env::args();
    args.next().unwrap();
    if let Some(ref source_path) = args.next() {
        build_dictionary(source_path)
    } else {
        load_dictionary("eitaro.dic")
    }

}

fn main() {
    if let Err(err) =  _main() {
        eprintln!("Error: {:?}", err);
        eprintln!("Usage: eitaro <DICTIONARY>");
        exit(1);
    }
}
