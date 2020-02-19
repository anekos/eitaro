
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

use structopt::StructOpt;

use crate::errors::{AppError, AppResultU};



#[derive(Debug, Default, Eq, PartialEq, StructOpt)]
pub struct Opt {
    /// Command line arguments for sqlite3
    args: Vec<String>,
}


pub fn shell<T: AsRef<Path>>(opt: Opt, path: &T) -> AppResultU {
    let path = path.as_ref().to_str().ok_or(AppError::Unexpect("Invalid path string"))?;
    Command::new("sqlite3")
        .arg(&path)
        .args(&opt.args)
        .exec();

    Ok(())
}



