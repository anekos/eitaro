
use std::io::Read;

use dictionary::DictionaryWriter;
use errors::AppError;
use loader::Loader;
use parser::ejdic::parse_line;



#[derive(Default)]
pub struct EjdicLoader();


impl Loader for EjdicLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> Result<(), AppError> {
        println!("Reading...");
        let mut buffer = "".to_owned();
        let _ = source.read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            load_line(writer, line)?;
        }

        Ok(())
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) -> Result<(), AppError> {
    if_let_some!(tab = line.find('\t'), Ok(()));
    let keys = &line[0..tab];
    let def = &line[tab+1..];

    let mut keys = keys.split(',');
    let key = keys.next().unwrap();
    writer.insert(key, parse_line(def)?)?;
    for alias in keys {
        writer.alias(&alias.trim(), key)?;
    }

    Ok(())
}
