
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::sync::PoisonError;

use failure::Fail;

use crate::types::DictionaryFormat;



pub type AppResult<T> = Result<T, AppError>;
pub type AppResultU = AppResult<()>;



#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "Could not get application directory: {}", 0)]
    AppDirs(app_dirs::AppDirsError),
    #[fail(display = "Deserialize error: {}", 0)]
    Bincode(Box<bincode::ErrorKind>),
    #[fail(display = "CSV Error: {}", 0)]
    Csv(csv::Error),
    #[fail(display = "Dictionary format ({:?}) error: {}", 0, 1)]
    DictionaryFormat(DictionaryFormat, &'static str),
    #[fail(display = "Error: {}", 0)]
    Eitaro(&'static str),
    #[fail(display = "Encoding error: {}", 0)]
    Encoding(&'static str),
    #[fail(display = "Format error: {}", 0)]
    Format(std::fmt::Error),
    #[fail(display = "IO error: {}", 0)]
    Io(std::io::Error),
    #[fail(display = "Database error: {}", 0)]
    Kv(kv::Error),
    #[fail(display = "Failed to lock")]
    Lock,
    #[fail(display = "Invalid number: {}", 0)]
    NumberFormat(std::num::ParseIntError),
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parser error: {}", 0)]
    Pom(pom::Error),
    #[fail(display = "Readline error: {}", 0)]
    Readline(rustyline::error::ReadlineError),
    #[fail(display = "Regular expression error: {}", 0)]
    Regex(regex::Error),
    #[fail(display = "Error: {}", 0)]
    Standard(String),
    #[fail(display = "UTF8 conversion error: {}", 0)]
    Utf8(std::str::Utf8Error),
    #[fail(display = "Void")]
    Void,
}


macro_rules! define_error {
    ($source:ty, $kind:ident) => {
        impl From<$source> for AppError {
            fn from(error: $source) -> AppError {
                AppError::$kind(error)
            }
        }
    }
}


define_error!(app_dirs::AppDirsError, AppDirs);
define_error!(kv::Error, Kv);
define_error!(pom::Error, Pom);
define_error!(regex::Error, Regex);
define_error!(rustyline::error::ReadlineError, Readline);
define_error!(std::fmt::Error, Format);
define_error!(std::num::ParseIntError, NumberFormat);
define_error!(std::str::Utf8Error, Utf8);
define_error!(csv::Error, Csv);
define_error!(Box<bincode::ErrorKind>, Bincode);




impl<'a> From<PoisonError<RwLockWriteGuard<'a, kv::Store>>> for AppError {
    fn from(error: PoisonError<RwLockWriteGuard<'a, kv::Store>>) -> AppError {
        AppError::Standard(format!("{}", error))
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, kv::Store>>> for AppError {
    fn from(_error: PoisonError<RwLockReadGuard<'a, kv::Store>>) -> AppError {
        AppError::Lock
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(error: Box<dyn std::error::Error>) -> AppError {
        AppError::Standard(error.description().to_owned())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> AppError {
        if error.kind() == std::io::ErrorKind::BrokenPipe {
            AppError::Void
        } else {
            AppError::Io(error)
        }
    }
}
