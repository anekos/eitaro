
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use errors::{AppError, ErrorKind};

use loader::{eijiro, ejdic, Loader};


#[derive(Clone, Copy)]
pub enum DictionaryFormat {
    Eijiro,
    Ejdic,
}



pub fn build_dictionary<T: AsRef<Path>, U: AsRef<Path>>(source_path: &T, dictionary_path: &U, dictionary_format: DictionaryFormat) -> Result<(), AppError> {
    use self::DictionaryFormat::*;

    let mut file = File::open(source_path)?;

    println!("Loading...");
    let (_, stat) = match dictionary_format {
        Eijiro => eijiro::EijiroLoader::default().load(&mut file, dictionary_path)?,
        Ejdic => ejdic::EjdicLoader::default().load(&mut file, dictionary_path)?,
    };
    println!("Finished: {} words, {} aliases", stat.words, stat.aliases);

    Ok(())
}


impl FromStr for DictionaryFormat {
    type Err = ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::DictionaryFormat::*;

        let result = match s {
            "eijiro" => Eijiro,
            "ejdic" => Ejdic,
            _ => return Err(ErrorKind::Eitaro("Invalid dictionary type"))
        };

        Ok(result)
    }
}
