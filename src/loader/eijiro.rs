
use std::error::Error;
use std::path::Path;

use store::{Dictionary, DictionaryWriter};
use loader::Loader;



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, Box<Error>> {
        let mut result = Dictionary::new(dictionary_path);

        result.writes(move |writer| {
            for line in source.lines() {
                if line.starts_with("■") {
                    load_line(writer, &line[3..]);
                }
            }
        }).unwrap(); // FIXME

        Ok(result)
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) {
    if_let_some!(sep = line.find(" : "), ());
    let (left, right) = line.split_at(sep);
    let right = &right[3..];

    if let (Some(l), Some(r)) = (left.find('{'), left.rfind('}')) {
        let (left, tag) = left.split_at(l);
        let left = left.trim();
        let mut tag = &tag[1..(r - l)];
        if let Some(hyphen) = tag.find('-') {
            tag = &tag[0.. hyphen];
        }
        let right = if tag.chars().next().map(|it| it.is_digit(10)) == Some(false) {
            format!("{{{}}} {}", tag, right)
        } else {
            format!("{}", right)
        };
        writer.insert(left, &right).unwrap(); // FIXME
        return;
    }

    writer.insert(left, right).unwrap(); // FIXME
}
