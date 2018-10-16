
use std::io::Read;

use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

use dictionary::DictionaryWriter;
use errors::AppError;
use loader::Loader;
use parser::gene::parse_line;



#[derive(Default)]
pub struct GeneLoader();


impl Loader for GeneLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> Result<(), AppError> {
        println!("Reading...");
        let mut buffer = vec![];
        let _ = source.read_to_end(&mut buffer)?;

        println!("Encoding...");
        let source = WINDOWS_31J.decode(&buffer, Replace).map_err(|err| err.to_string())?;

        let mut key_buffer = None;

        for line in source.lines().skip(2) {
            if let Some(key) = key_buffer.take() {
                let definition = parse_line(line)?;
                writer.insert(key, definition)?;
            } else {
                key_buffer = Some(line)
            }
        }

        Ok(())
    }
}