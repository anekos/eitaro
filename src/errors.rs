
use std::error::{Error as StdError};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IOError;
use std::str::Utf8Error;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use app_dirs::AppDirsError;
use failure::{Context, Fail};
use kv::{Store as KvStore, Error as KvError};
use std::sync::PoisonError;
use readline;



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
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "Database error")]
    Kv,
    #[fail(display = "Failed to lock")]
    Lock,
    #[fail(display = "Readline error")]
    Readline,
    #[fail(display = "Standard error")]
    Standard,
    #[fail(display = "UTF8 conversion error")]
    Utf8,
}


impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.inner, f)
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
def_from_error!(Io, IOError);
def_from_error!(Kv, KvError);
def_from_error!(Readline, readline::Error);
def_from_error!(Utf8, Utf8Error);


impl From<ErrorKind> for AppError {
    fn from(kind: ErrorKind) -> AppError {
        AppError {
            inner: Context::new(kind),
        }
    }
}

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
    fn from(_error: Box<StdError>) -> AppError {
        AppError {
            inner: Context::from(ErrorKind::Standard),
        }
    }
}

impl From<String> for AppError {
    fn from(_error: String) -> AppError {
        AppError {
            inner: Context::from(ErrorKind::Standard),
        }
    }
}
