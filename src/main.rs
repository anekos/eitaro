
extern crate encoding;

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



fn _main() -> Result<(), Box<Error>> {
    let mut args = env::args();
    args.next().unwrap();
    let source_path = args.next().ok_or("Not enough arguments")?;

    let mut buffer = vec![];

    let mut file = File::open(source_path)?;
    let _ = file.read_to_end(&mut buffer)?;

    eprintln!("Decoding...");
    let decoded = WINDOWS_31J.decode(&buffer, Replace)?;

    eprintln!("Loading...");
    let ldr = loader::eijiro::EijiroLoader::default();
    ldr.load(&decoded);

    Ok(())
}


fn main() {
    if let Err(err) =  _main() {
        eprintln!("Error: {:?}", err);
        eprintln!("Usage: eitaro <DICTIONARY>");
        exit(1);
    }
}
