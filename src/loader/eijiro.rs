
use std::path::Path;

use errors::AppError;
use loader::Loader;
use store::{Dictionary, DictionaryWriter};



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load<T: AsRef<Path>>(&self, source: &str, dictionary_path: &T) -> Result<Dictionary, AppError> {
        let mut result = Dictionary::new(dictionary_path);

        result.writes(move |writer| {
            for line in source.lines() {
                if line.starts_with("■") {
                    load_line(writer, &line[3..])?;
                }
            }
            Ok(())
        })?;

        Ok(result)
    }
}


fn load_line(writer: &mut DictionaryWriter, line: &str) -> Result<(), AppError> {
    if_let_some!(sep = line.find(" : "), Ok(()));
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
        writer.insert(left, &right)?;
        return Ok(());
    }

    writer.insert(left, right)?;

    Ok(())
}
