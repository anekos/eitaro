
use std::io::Read;

use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

use crate::dictionary::DictionaryWriter;
use crate::errors::{AppError, AppResultU};
use crate::loader::Loader;
use crate::parser::gene::parse_line;



#[derive(Default)]
pub struct GeneLoader();


impl Loader for GeneLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU {
        println!("Reading GENE...");
        let mut buffer = vec![];
        let _ = source.read_to_end(&mut buffer)?;

        println!("Encoding...");
        let source = WINDOWS_31J.decode(&buffer, Replace).map_err(|_| AppError::Encoding("MS932"))?;

        let mut key_buffer = None;

        for line in source.lines().skip(2) {
            if let Some(key) = key_buffer.take() {
                let definition = parse_line(line)?;
                writer.define(key, definition)?;
            } else {
                key_buffer = Some(line)
            }
        }

        Ok(())
    }
}
