
use std::error::Error;
use std::path::Path;

use dictionary::{Dictionary, DictionaryWriter};
use loader::Loader;



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, Box<Error>> {
        let mut result = Dictionary::new(dictionary_path)?;

        let lines: Vec<&str> = source.lines().collect();
        for chunk in lines.chunks(10000) {
            result.writes(move |writer| {
                for line in chunk {
                    if line.starts_with("■") {
                        load_line(writer, &line[3..]);
                    }
                }
            })?;
        }

        Ok(result)
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) {
    if_let_some!(sep = line.find(" : "), ());
    let (mut left, right) = line.split_at(sep);
    let right = &right[3..];

    let _ = if let (Some(l), Some(r)) = (left.find('{'), left.rfind('}')) {
        let (_left, tag) = left.split_at(l);
        left = _left.trim();
        Some(&tag[1..(r - l)])
    } else {
        None
    };

    writer.insert(left, right);
}
