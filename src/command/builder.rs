
use std::fs::File;
use std::io::Read;
use std::path::Path;

use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;
use errors::AppError;

use loader::{eijiro, Loader};



pub fn build_dictionary<T: AsRef<Path>, U: AsRef<Path>>(source_path: &T, dictionary_path: &U) -> Result<(), AppError> {
    eprintln!("Building...");
    let mut buffer = vec![];
    let mut file = File::open(source_path)?;
    eprintln!("Reading...");
    let _ = file.read_to_end(&mut buffer)?;
    eprintln!("Encoding...");
    let decoded = WINDOWS_31J.decode(&buffer, Replace).map_err(|err| err.to_string())?;
    eprintln!("Loading...");
    let ldr = eijiro::EijiroLoader::default();
    ldr.load(&decoded, dictionary_path).unwrap();
    Ok(())
}
