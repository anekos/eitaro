
use std::error::{Error as StdError};
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::io::Error as IOError;
use std::str::Utf8Error;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use app_dirs::AppDirsError;
use failure::{Context, Fail, Backtrace};
use kv::{Store as KvStore, Error as KvError};
use regex;
use std::sync::PoisonError;



#[derive(Debug)]
pub struct AppError {
    inner: Context<ErrorKind>,
}

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "Could not get application directory")]
    AppDirs,
    #[fail(display = "Error")]
    Eitaro(&'static str),
    #[fail(display = "Fomat")]
    Format,
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "Database error")]
    Kv,
    #[fail(display = "Failed to lock")]
    Lock,
    #[fail(display = "Readline error")]
    Readline,
    #[fail(display = "Regular expression error")]
    Regex,
    #[fail(display = "Standard error")]
    Standard(String),
    #[fail(display = "UTF8 conversion error")]
    Utf8,
}


impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::ErrorKind::*;

        match self.inner.get_context() {
            Standard(error) => writeln!(f, "{}", error),
            Eitaro(error) => writeln!(f, "{}", error),
            _ => Display::fmt(&self.inner, f),

        }
    }
}

impl Fail for AppError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for AppError {
    fn from(kind: ErrorKind) -> Self {
        AppError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for AppError {
    fn from(inner: Context<ErrorKind>) -> Self {
        AppError { inner }
    }
}


macro_rules! def_from_error {
    ($kind:tt, $type:ty) => {
        impl From<$type> for AppError {
            fn from(error: $type) -> AppError {
                AppError {
                    inner: error.context(ErrorKind::$kind),
                }
            }
        }
    };

    ($kind_type:tt) => {
        def_from_error!($kind_type, $kind_type);
    }
}


def_from_error!(AppDirs, AppDirsError);
def_from_error!(Format, FmtError);
def_from_error!(Io, IOError);
def_from_error!(Kv, KvError);
def_from_error!(Regex, regex::Error);
def_from_error!(Utf8, Utf8Error);
def_from_error!(Readline, rustyline::error::ReadlineError);



impl<'a> From<PoisonError<RwLockWriteGuard<'a, KvStore>>> for AppError {
    fn from(_error: PoisonError<RwLockWriteGuard<'a, KvStore>>) -> AppError {
        AppError {
            inner: Context::from(ErrorKind::Lock),
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, KvStore>>> for AppError {
    fn from(_error: PoisonError<RwLockReadGuard<'a, KvStore>>) -> AppError {
        AppError {
            inner: Context::from(ErrorKind::Lock),
        }
    }
}

impl From<Box<StdError>> for AppError {
    fn from(error: Box<StdError>) -> AppError {
        AppError::from(ErrorKind::Standard(error.description().to_owned()))
    }
}

impl From<String> for AppError {
    fn from(error: String) -> AppError {
        AppError {
            inner: Context::from(ErrorKind::Standard(error)),
        }
    }
}
