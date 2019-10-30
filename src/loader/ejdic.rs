
use std::io::Read;

use if_let_return::if_let_some;

use crate::dictionary::DictionaryWriter;
use crate::errors::AppResultU;
use crate::loader::Loader;
use crate::parser::ejdic::parse_line;



#[derive(Default)]
pub struct EjdicLoader();


impl Loader for EjdicLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU {
        println!("Reading...");
        let mut buffer = "".to_owned();
        let _ = source.read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            load_line(writer, line)?;
        }

        Ok(())
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) -> AppResultU {
    if_let_some!(tab = line.find('\t'), Ok(()));
    let keys = &line[0..tab];
    let definitions = &line[tab+1..];

    let mut keys = keys.split(',');
    let key = keys.next().unwrap();
    for definition in definitions.split(" / ") {
        writer.insert(key, parse_line(definition)?)?;
    }
    for alias in keys {
        writer.alias(&alias.trim(), key)?;
    }

    Ok(())
}
