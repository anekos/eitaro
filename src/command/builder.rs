
use std::fs::File;
use std::io::Read;
use std::path::Path;

use errors::{AppError, ErrorKind};

use dictionary::Dictionary;
use loader::{eijiro, ejdic, Loader};


#[derive(Clone, Copy)]
pub enum DictionaryFormat {
    Eijiro,
    Ejdic,
}



pub fn build_dictionary<T: AsRef<Path>, U: AsRef<Path>>(files: &[T], dictionary_path: &U) -> Result<(), AppError> {
    use self::DictionaryFormat::*;

    let mut dictionary = Dictionary::new(dictionary_path);

    let stat = dictionary.write(move |mut writer| {
        for file in files {
            println!("[{}]", file.as_ref().to_str().unwrap_or("-"));
            let format = guess(file)?;
            let mut file = File::open(file)?;
            match format {
                Eijiro => eijiro::EijiroLoader::default().load(&mut file, &mut writer)?,
                Ejdic => ejdic::EjdicLoader::default().load(&mut file, &mut writer)?,
            };
        }
        Ok(())
    })?;

    println!("Finished: {} words, {} aliases", stat.words, stat.aliases);

    Ok(())
}

fn guess<T: AsRef<Path>>(source_path: &T) -> Result<DictionaryFormat, AppError> {
    let mut file = File::open(source_path)?;
    let mut head = [0u8;100];
    let size = file.read(&mut head)?;
    let head = &head[0..size];

    // 81a1 == ■
    if 3 <= size && head.starts_with(b"\x81\xa1") {
        return Ok(DictionaryFormat::Eijiro);
    }

    if head.contains(&b'\t') {
        return Ok(DictionaryFormat::Ejdic);
    }

    Err(ErrorKind::Eitaro("Unknown format"))?
}
