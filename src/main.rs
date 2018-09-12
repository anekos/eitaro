
#[macro_use] extern crate if_let_return;
extern crate app_dirs;
extern crate encoding;
extern crate kv;
extern crate nickel;
extern crate percent_encoding;
extern crate rusty_leveldb;

use std::env;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

use app_dirs::{AppInfo, AppDataType};
use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

mod http;
mod loader;
mod store;

use loader::Loader;



const APP_INFO: AppInfo = AppInfo { name: "eitaro", author: "anekos" };


fn get_dictionary_path() -> Result<PathBuf, Box<Error>> {
    let result = app_dirs::get_app_dir(AppDataType::UserCache, &APP_INFO, "dictionary")?;
    if !result.exists() {
        create_dir_all(&result).unwrap();
    }
    Ok(result)
}

fn build_dictionary<T: AsRef<Path>, U: AsRef<Path>>(source_path: &T, dictionary_path: &U) -> Result<(), Box<Error>> {
    eprintln!("Building...");
    let mut buffer = vec![];
    let mut file = File::open(source_path)?;
    eprintln!("Reading...");
    let _ = file.read_to_end(&mut buffer)?;
    eprintln!("Encoding...");
    let decoded = WINDOWS_31J.decode(&buffer, Replace)?;
    eprintln!("Loading...");
    let ldr = loader::eijiro::EijiroLoader::default();
    let _ = ldr.load(&decoded, dictionary_path);
    Ok(())
}

fn _main() -> Result<(), Box<Error>> {
    let mut args = env::args();
    args.next().unwrap();
    let dictionary_path = get_dictionary_path()?;
    if let Some(ref source_path) = args.next() {
        build_dictionary(source_path, &dictionary_path)
    } else {
        http::main(&dictionary_path)
    }
}

fn main() {
    if let Err(err) =  _main() {
        eprintln!("Error: {:?}", err);
        eprintln!("Usage: eitaro <DICTIONARY>");
        exit(1);
    }
}
