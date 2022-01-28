
use std::io::Read;

use json::{parse, JsonValue};
use regex::Regex;

use crate::dictionary::{DictionaryWriter, Text};
use crate::errors::{AppError, AppResultU};
use crate::loader::Loader;
use crate::types::DictionaryFormat::JsonSimpleKeyValue;



#[derive(Default)]
pub struct JsonSimpleKeyValueLoader();


impl Loader for JsonSimpleKeyValueLoader {
    fn load<S: Read>(&self, source: &mut S, writer: &mut DictionaryWriter) -> AppResultU {
        println!("Reading JSON as simple key value object...");

        let number = Regex::new(r"\d+\. ")?;

        let mut buffer: String = "".to_owned();
        source.read_to_string(&mut buffer)?;
        if let Ok(JsonValue::Object(obj)) = parse(&buffer) {
            for (term, def) in obj.iter() {
                let def = match def {
                    JsonValue::String(def) => def,
                    JsonValue::Short(def) => def.as_str(),
                    _ => return Err(AppError::DictionaryFormat(JsonSimpleKeyValue, "Invalid type")),
                };

                let mut left = 0;
                for m in number.find_iter(def) {
                    let start = m.start();
                    if 0 < (start - left) {
                        writer.define(&term, text(&def[left .. start]))?;
                    }
                    left = start;
                }
                if 0 < left {
                    writer.define(&term, text(&def[left ..]))?;
                }
            }
        }
        Ok(())
    }
}


fn text(s: &str) -> Vec<Text> {
    vec![Text::Definition(s.to_owned())]
}
